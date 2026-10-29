//! Network protocol, packet serialization, and transport abstractions for RP235x.

#![no_std]

pub mod data;
pub mod event;
pub mod layout;
pub mod node;
pub mod packet;
pub mod transport;

pub use data::{Device, DeviceType, Handshake};
pub use event::{ClientEvent, ServerEvent};
pub use node::{
    Client, ClientHandler, NetError, Node, PacketHandler, ReceivedPacket, Server, ServerHandler,
};
pub use packet::{ClientPacket, Packet, ServerPacket};
pub use transport::{
    PacketReceiver, PacketSender, RawConnection, RawConnection as WebSocketConnection, RawPacket,
    SplitTransport, Transport, Transport as WebSocketTransport,
};
