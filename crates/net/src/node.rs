use core::future::Future;
use core::marker::PhantomData;
use core::net::SocketAddr;

use crate::packet::{ClientPacket, Packet, ServerPacket};
use crate::transport::{PacketReceiver, PacketSender};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NetError {
    Disconnected,
    InvalidPacket(&'static str),
}

pub trait ClientHandler: Send {
    fn handle_packet(
        &mut self,
        packet: ServerPacket,
        address: Option<SocketAddr>,
    ) -> impl Future<Output = Option<ClientPacket>> + Send;
}

pub trait ServerHandler: Send {
    fn handle_packet(
        &mut self,
        packet: ClientPacket,
        address: Option<SocketAddr>,
    ) -> impl Future<Output = Option<ServerPacket>> + Send;
}

pub trait PacketHandler<In: Packet, Out: Packet>: Send {
    fn handle_packet(
        &mut self,
        packet: In,
        address: Option<SocketAddr>,
    ) -> impl Future<Output = Option<Out>> + Send;
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

pub struct Node<T, H, In, Out> {
    transport: T,
    handler: H,
    _marker: PhantomData<(In, Out)>,
}

pub type Client<T, H> = Node<T, H, ServerPacket, ClientPacket>;
pub type Server<T, H> = Node<T, H, ClientPacket, ServerPacket>;

impl<T, H, In, Out> Node<T, H, In, Out> {
    #[must_use]
    pub fn new(transport: T, handler: H) -> Self {
        Self {
            transport,
            handler,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    #[must_use]
    pub fn handler(&self) -> &H {
        &self.handler
    }

    pub fn handler_mut(&mut self) -> &mut H {
        &mut self.handler
    }
}

impl<T, H, In, Out> Node<T, H, In, Out>
where
    T: PacketSender,
    Out: Packet,
{
    pub async fn send(&mut self, packet: &Out) {
        self.transport.send_packet(&packet.to_bytes()).await;
    }

    pub async fn send_to(&mut self, packet: &Out, address: Option<SocketAddr>) {
        self.transport.send_packet_to(&packet.to_bytes(), address).await;
    }
}

impl<T, H, In, Out> Node<T, H, In, Out>
where
    T: PacketReceiver + PacketSender,
    H: PacketHandler<In, Out>,
    In: Packet,
    Out: Packet,
{
    pub async fn poll(&mut self) -> Result<In, NetError> {
        let raw = self
            .transport
            .receive_packet()
            .await
            .map_err(|_| NetError::Disconnected)?;

        let packet = In::from_bytes(raw.bytes).map_err(NetError::InvalidPacket)?;

        if let Some(response) = self.handler.handle_packet(packet, raw.address).await {
            self.send_to(&response, raw.address).await;
        }

        Ok(packet)
    }
}
