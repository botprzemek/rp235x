//! Client and server network node abstractions and packet handlers.

use core::future::Future;
use core::marker::PhantomData;
use core::net::SocketAddr;

use crate::event::ServerEvent;
use crate::packet::{ClientPacket, Packet, ServerPacket};
use crate::transport::{PacketReceiver, PacketSender};

/// Errors produced by a [`Node`] when receiving or decoding packets.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NetError {
    Disconnected,
    InvalidPacket(&'static str),
}

/// Decoded packet together with its optional sender socket address.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReceivedPacket<P> {
    pub packet: P,
    pub address: Option<SocketAddr>,
}

/// Asynchronous handler that processes an inbound packet and optionally produces a reply.
pub trait PacketHandler<In: Packet, Out: Packet>: Send {
    /// Handles an incoming `packet` from `address` and returns an optional response packet.
    fn handle_packet(
        &mut self,
        packet: In,
        address: Option<SocketAddr>,
    ) -> impl Future<Output = Option<Out>> + Send;
}

/// Asynchronous handler for client-side nodes receiving [`ServerPacket`]s.
pub trait ClientHandler: Send {
    /// Handles an incoming [`ServerPacket`] and returns an optional [`ClientPacket`] response.
    fn handle_packet(
        &mut self,
        packet: ServerPacket,
        address: Option<SocketAddr>,
    ) -> impl Future<Output = Option<ClientPacket>> + Send;
}

/// Asynchronous handler for server-side nodes receiving [`ClientPacket`]s.
pub trait ServerHandler: Send {
    /// Handles an incoming [`ClientPacket`] and returns an optional [`ServerPacket`] response.
    fn handle_packet(
        &mut self,
        packet: ClientPacket,
        address: Option<SocketAddr>,
    ) -> impl Future<Output = Option<ServerPacket>> + Send;
}

impl<H: ClientHandler> PacketHandler<ServerPacket, ClientPacket> for H {
    async fn handle_packet(
        &mut self,
        packet: ServerPacket,
        address: Option<SocketAddr>,
    ) -> Option<ClientPacket> {
        ClientHandler::handle_packet(self, packet, address).await
    }
}

impl<H: ServerHandler> PacketHandler<ClientPacket, ServerPacket> for H {
    async fn handle_packet(
        &mut self,
        packet: ClientPacket,
        address: Option<SocketAddr>,
    ) -> Option<ServerPacket> {
        ServerHandler::handle_packet(self, packet, address).await
    }
}

/// Protocol endpoint combining a packet transport and a packet handler.
pub struct Node<T, H, In, Out> {
    transport: T,
    handler: H,
    _marker: PhantomData<(In, Out)>,
}

/// Client-side [`Node`] receiving [`ServerPacket`]s and sending [`ClientPacket`]s.
pub type Client<T, H> = Node<T, H, ServerPacket, ClientPacket>;

/// Server-side [`Node`] receiving [`ClientPacket`]s and sending [`ServerPacket`]s.
pub type Server<T, H> = Node<T, H, ClientPacket, ServerPacket>;

impl<T, H, In, Out> Node<T, H, In, Out> {
    /// Creates a new [`Node`] from the given `transport` and `handler`.
    #[must_use]
    pub fn new(transport: T, handler: H) -> Self {
        Self {
            transport,
            handler,
            _marker: PhantomData,
        }
    }

    /// Returns a shared reference to the underlying transport.
    #[must_use]
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Returns a mutable reference to the underlying transport.
    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Returns a shared reference to the packet handler.
    #[must_use]
    pub fn handler(&self) -> &H {
        &self.handler
    }

    /// Returns a mutable reference to the packet handler.
    pub fn handler_mut(&mut self) -> &mut H {
        &mut self.handler
    }
}

impl<T, H, In, Out> Node<T, H, In, Out>
where
    T: PacketSender,
    Out: Packet,
{
    /// Serializes and sends `packet` through the underlying transport.
    pub async fn send(&mut self, packet: &Out) {
        let bytes = packet.to_bytes();
        self.transport.send_packet(&bytes).await;
    }

    /// Serializes and sends `packet` to `address` through the underlying transport.
    pub async fn send_to(&mut self, packet: &Out, address: Option<SocketAddr>) {
        let bytes = packet.to_bytes();
        self.transport.send_packet_to(&bytes, address).await;
    }
}

impl<T, H, In, Out> Node<T, H, In, Out>
where
    T: PacketReceiver,
    In: Packet,
{
    /// Receives and decodes a single packet from the underlying transport.
    pub async fn receive(&mut self) -> Result<ReceivedPacket<In>, NetError> {
        let raw_packet = self
            .transport
            .receive_packet()
            .await
            .map_err(|_| NetError::Disconnected)?;
        let packet = In::from_bytes(raw_packet.bytes).map_err(NetError::InvalidPacket)?;
        Ok(ReceivedPacket {
            packet,
            address: raw_packet.address,
        })
    }
}

impl<T, H, In, Out> Node<T, H, In, Out>
where
    T: PacketReceiver + PacketSender,
    H: PacketHandler<In, Out>,
    In: Packet,
    Out: Packet,
{
    /// Receives a single packet, dispatches it to the handler, and sends any reply.
    pub async fn poll(&mut self) -> Result<In, NetError> {
        let received = self.receive().await?;

        if let Some(response) = self
            .handler
            .handle_packet(received.packet, received.address)
            .await
        {
            self.send_to(&response, received.address).await;
        }

        Ok(received.packet)
    }

    /// Continuously polls and processes packets until the transport disconnects.
    pub async fn run(&mut self) -> Result<(), NetError> {
        loop {
            match self.poll().await {
                Ok(_) | Err(NetError::InvalidPacket(_)) => continue,
                Err(NetError::Disconnected) => return Err(NetError::Disconnected),
            }
        }
    }
}

#[allow(dead_code)]
const _: Option<ServerEvent> = None;
