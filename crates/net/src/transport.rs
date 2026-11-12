use core::future::Future;
use core::net::SocketAddr;

use crate::layout::PACKET_SIZE;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawPacket {
    pub bytes: [u8; PACKET_SIZE],
    pub address: Option<SocketAddr>,
}

impl RawPacket {
    #[must_use]
    pub const fn new(bytes: [u8; PACKET_SIZE], address: Option<SocketAddr>) -> Self {
        Self { bytes, address }
    }
}

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
    fn receive_packet(&mut self) -> impl Future<Output = Result<RawPacket, &'static str>> + Send;
}

pub trait RawConnection: Send {
    fn send(&mut self, data: &[u8; PACKET_SIZE]) -> impl Future<Output = Result<(), ()>> + Send;

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
    #[must_use]
    pub fn new(connection: C, address: Option<SocketAddr>) -> Self {
        Self {
            connection,
            address,
        }
    }

    #[must_use]
    pub fn connection(&self) -> &C {
        &self.connection
    }

    pub fn connection_mut(&mut self) -> &mut C {
        &mut self.connection
    }

    #[must_use]
    pub fn address(&self) -> Option<SocketAddr> {
        self.address
    }

    #[must_use]
    pub fn into_inner(self) -> C {
        self.connection
    }
}

impl<C: RawConnection> PacketSender for Transport<C> {
    async fn send_packet(&mut self, data: &[u8; PACKET_SIZE]) {
        let _ = self.connection.send(data).await;
    }
}

impl<C: RawConnection> PacketReceiver for Transport<C> {
    async fn receive_packet(&mut self) -> Result<RawPacket, &'static str> {
        let mut buffer = [0u8; PACKET_SIZE];
        loop {
            let size = self
                .connection
                .receive(&mut buffer)
                .await
                .map_err(|()| "disconnected")?;

            if size == PACKET_SIZE {
                return Ok(RawPacket {
                    bytes: buffer,
                    address: self.address,
                });
            }
        }
    }
}

pub struct SplitTransport<S, R> {
    sender: S,
    receiver: R,
}

impl<S, R> SplitTransport<S, R> {
    #[must_use]
    pub fn new(sender: S, receiver: R) -> Self {
        Self { sender, receiver }
    }

    #[must_use]
    pub fn sender(&self) -> &S {
        &self.sender
    }

    pub fn sender_mut(&mut self) -> &mut S {
        &mut self.sender
    }

    #[must_use]
    pub fn receiver(&self) -> &R {
        &self.receiver
    }

    pub fn receiver_mut(&mut self) -> &mut R {
        &mut self.receiver
    }
}

impl<S: PacketSender, R: Send> PacketSender for SplitTransport<S, R> {
    async fn send_packet(&mut self, data: &[u8; PACKET_SIZE]) {
        self.sender.send_packet(data).await;
    }

    async fn send_packet_to(&mut self, data: &[u8; PACKET_SIZE], address: Option<SocketAddr>) {
        self.sender.send_packet_to(data, address).await;
    }
}

impl<S: Send, R: PacketReceiver> PacketReceiver for SplitTransport<S, R> {
    async fn receive_packet(&mut self) -> Result<RawPacket, &'static str> {
        self.receiver.receive_packet().await
    }
}
