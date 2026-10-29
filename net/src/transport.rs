//! Packet sender, receiver, and transport abstractions.

use core::future::Future;
use core::net::SocketAddr;

use crate::layout::PACKET_SIZE;

/// Raw wire packet buffer accompanied by an optional peer address.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawPacket {
    pub bytes: [u8; PACKET_SIZE],
    pub address: Option<SocketAddr>,
}

impl RawPacket {
    /// Creates a new [`RawPacket`] from `bytes` and an optional peer `address`.
    #[must_use]
    pub const fn new(bytes: [u8; PACKET_SIZE], address: Option<SocketAddr>) -> Self {
        Self { bytes, address }
    }
}

/// Asynchronous sink for 64-byte wire packets.
pub trait PacketSender: Send {
    /// Sends a single 64-byte packet.
    fn send_packet(&mut self, data: &[u8; PACKET_SIZE]) -> impl Future<Output = ()> + Send;

    /// Sends a single 64-byte packet to the specified `address`.
    fn send_packet_to(
        &mut self,
        data: &[u8; PACKET_SIZE],
        _address: Option<SocketAddr>,
    ) -> impl Future<Output = ()> + Send {
        self.send_packet(data)
    }
}

/// Asynchronous source of 64-byte wire packets.
pub trait PacketReceiver: Send {
    /// Receives the next 64-byte packet from the transport.
    fn receive_packet(&mut self) -> impl Future<Output = Result<RawPacket, &'static str>> + Send;
}

/// Low-level byte connection capable of sending and receiving fixed-size packet buffers.
pub trait RawConnection: Send {
    /// Sends a 64-byte buffer over the connection.
    fn send(&mut self, data: &[u8; PACKET_SIZE]) -> impl Future<Output = Result<(), ()>> + Send;

    /// Receives bytes into `buffer` and returns the number of bytes read.
    fn receive(
        &mut self,
        buffer: &mut [u8; PACKET_SIZE],
    ) -> impl Future<Output = Result<usize, ()>> + Send;
}

/// Packet transport wrapping a [`RawConnection`] and an optional peer [`SocketAddr`].
pub struct Transport<C> {
    connection: C,
    address: Option<SocketAddr>,
}

impl<C> Transport<C> {
    /// Wraps `connection` and `address` into a [`Transport`].
    #[must_use]
    pub fn new(connection: C, address: Option<SocketAddr>) -> Self {
        Self {
            connection,
            address,
        }
    }

    /// Returns a shared reference to the underlying connection.
    #[must_use]
    pub fn connection(&self) -> &C {
        &self.connection
    }

    /// Returns a mutable reference to the underlying connection.
    pub fn connection_mut(&mut self) -> &mut C {
        &mut self.connection
    }

    /// Returns the peer socket address associated with this transport, if any.
    #[must_use]
    pub fn address(&self) -> Option<SocketAddr> {
        self.address
    }

    /// Consumes the transport and returns the underlying connection.
    #[must_use]
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
    async fn receive_packet(&mut self) -> Result<RawPacket, &'static str> {
        let mut buffer = [0u8; PACKET_SIZE];

        loop {
            let response_size = self
                .connection
                .receive(&mut buffer)
                .await
                .map_err(|()| "disconnected")?;

            if response_size != PACKET_SIZE {
                continue;
            }

            return Ok(RawPacket {
                bytes: buffer,
                address: self.address,
            });
        }
    }
}

/// Transport composed of independent sender and receiver halves.
pub struct SplitTransport<S, R> {
    sender: S,
    receiver: R,
}

impl<S, R> SplitTransport<S, R> {
    /// Combines `sender` and `receiver` into a [`SplitTransport`].
    #[must_use]
    pub fn new(sender: S, receiver: R) -> Self {
        Self { sender, receiver }
    }

    /// Returns a shared reference to the sender half.
    #[must_use]
    pub fn sender(&self) -> &S {
        &self.sender
    }

    /// Returns a mutable reference to the sender half.
    pub fn sender_mut(&mut self) -> &mut S {
        &mut self.sender
    }

    /// Returns a shared reference to the receiver half.
    #[must_use]
    pub fn receiver(&self) -> &R {
        &self.receiver
    }

    /// Returns a mutable reference to the receiver half.
    pub fn receiver_mut(&mut self) -> &mut R {
        &mut self.receiver
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
    async fn receive_packet(&mut self) -> Result<RawPacket, &'static str> {
        self.receiver.receive_packet().await
    }
}
