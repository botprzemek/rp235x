#![no_std]

pub mod data;
pub mod event;
pub mod layout;
pub mod packet;
pub mod transport;

pub use data::{Device, DeviceType};
pub use event::{ClientEvent, ServerEvent};
pub use packet::{ClientPacket, Packet, ServerPacket};
pub use transport::{
    PacketReceiver, PacketSender, RawConnection, RawConnection as WebSocketConnection,
    SplitTransport, Transport, Transport as WebSocketTransport,
};

extern crate alloc;
use core::future::Future;
use core::marker::PhantomData;
use core::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NetError {
    Disconnected,
    InvalidPacket(&'static str),
}

pub trait PacketHandler<In: Packet, Out: Packet>: Send {
    fn handle_packet(
        &mut self,
        packet: In,
        address: Option<SocketAddr>,
    ) -> impl Future<Output = Option<Out>> + Send;
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
    pub fn new(transport: T, handler: H) -> Self {
        Self {
            transport,
            handler,
            _marker: PhantomData,
        }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    pub fn handler(&self) -> &H {
        &self.handler
    }

    pub fn handler_mut(&mut self) -> &mut H {
        &mut self.handler
    }

    pub fn into_parts(self) -> (T, H) {
        (self.transport, self.handler)
    }
}

impl<T, H, In, Out> Node<T, H, In, Out>
where
    T: PacketSender,
    Out: Packet,
{
    pub async fn send(&mut self, packet: &Out) {
        let bytes = packet.to_bytes();
        self.transport.send_packet(&bytes).await;
    }

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
    pub async fn receive(&mut self) -> Result<(In, Option<SocketAddr>), NetError> {
        let (raw, address) = self
            .transport
            .receive_packet()
            .await
            .map_err(|_| NetError::Disconnected)?;
        let packet = In::from_bytes(raw).map_err(NetError::InvalidPacket)?;
        Ok((packet, address))
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
        let (packet, address) = self.receive().await?;

        if let Some(response) = self.handler.handle_packet(packet, address).await {
            self.send_to(&response, address).await;
        }

        Ok(packet)
    }

    pub async fn run(&mut self) -> Result<(), NetError> {
        loop {
            match self.poll().await {
                Ok(_) | Err(NetError::InvalidPacket(_)) => continue,
                Err(NetError::Disconnected) => return Err(NetError::Disconnected),
            }
        }
    }
}
