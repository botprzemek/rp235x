use core::future::Future;
use core::net::SocketAddr;

use crate::layout::PACKET_SIZE;

pub trait PacketSender: Send {
    fn send_packet(&mut self, data: &[u8; PACKET_SIZE]) -> impl Future<Output = ()> + Send;

    fn send_packet_to(
        &mut self,
        data: &[u8; PACKET_SIZE],
        _address: Option<SocketAddr>,
    ) -> impl Future<Output = ()> + Send {
        self.send_packet(data)
    }
}

pub trait PacketReceiver: Send {
    fn receive_packet(
        &mut self,
    ) -> impl Future<Output = Result<([u8; PACKET_SIZE], Option<SocketAddr>), &'static str>> + Send;
}

pub trait RawConnection: Send {
    fn send(
        &mut self,
        data: &[u8; PACKET_SIZE],
    ) -> impl Future<Output = Result<(), ()>> + Send;

    fn receive(
        &mut self,
        buffer: &mut [u8; PACKET_SIZE],
    ) -> impl Future<Output = Result<usize, ()>> + Send;
}

pub struct Transport<C> {
    connection: C,
    address: Option<SocketAddr>,
}

impl<C> Transport<C> {
    pub fn new(connection: C, address: Option<SocketAddr>) -> Self {
        Self {
            connection,
            address,
        }
    }

    pub fn connection(&self) -> &C {
        &self.connection
    }

    pub fn connection_mut(&mut self) -> &mut C {
        &mut self.connection
    }

    pub fn address(&self) -> Option<SocketAddr> {
        self.address
    }

    pub fn into_inner(self) -> C {
        self.connection
    }
}

impl<C> PacketSender for Transport<C>
where
    C: RawConnection,
{
    async fn send_packet(&mut self, data: &[u8; PACKET_SIZE]) {
        let _ = self.connection.send(data).await;
    }
}

impl<C> PacketReceiver for Transport<C>
where
    C: RawConnection,
{
    async fn receive_packet(
        &mut self,
    ) -> Result<([u8; PACKET_SIZE], Option<SocketAddr>), &'static str> {
        let mut buffer = [0u8; PACKET_SIZE];

        loop {
            let response_size = self
                .connection
                .receive(&mut buffer)
                .await
                .map_err(|_| "disconnected")?;

            if response_size != PACKET_SIZE {
                continue;
            }

            return Ok((buffer, self.address));
        }
    }
}

pub struct SplitTransport<S, R> {
    sender: S,
    receiver: R,
}

impl<S, R> SplitTransport<S, R> {
    pub fn new(sender: S, receiver: R) -> Self {
        Self { sender, receiver }
    }

    pub fn sender(&self) -> &S {
        &self.sender
    }

    pub fn sender_mut(&mut self) -> &mut S {
        &mut self.sender
    }

    pub fn receiver(&self) -> &R {
        &self.receiver
    }

    pub fn receiver_mut(&mut self) -> &mut R {
        &mut self.receiver
    }

    pub fn into_parts(self) -> (S, R) {
        (self.sender, self.receiver)
    }
}

impl<S, R> PacketSender for SplitTransport<S, R>
where
    S: PacketSender,
    R: Send,
{
    async fn send_packet(&mut self, data: &[u8; PACKET_SIZE]) {
        self.sender.send_packet(data).await;
    }

    async fn send_packet_to(&mut self, data: &[u8; PACKET_SIZE], address: Option<SocketAddr>) {
        self.sender.send_packet_to(data, address).await;
    }
}

impl<S, R> PacketReceiver for SplitTransport<S, R>
where
    S: Send,
    R: PacketReceiver,
{
    async fn receive_packet(
        &mut self,
    ) -> Result<([u8; PACKET_SIZE], Option<SocketAddr>), &'static str> {
        self.receiver.receive_packet().await
    }
}
