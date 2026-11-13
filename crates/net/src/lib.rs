#![no_std]

mod data;
pub mod event;
pub mod game;
pub mod layout;
mod node;
mod packet;
pub mod protocol;
pub mod session;
mod transport;

#[macro_use]
pub mod macros;

pub use data::{
    Device, DeviceConfig, DeviceConfig as DeviceConfigPacket, DeviceType, Handshake, HandshakeAck,
    Heartbeat, HeartbeatAck,
};
pub use event::{ClientEvent, ServerEvent};
pub use node::{Client, ClientHandler, NetError, Node, PacketHandler, Server, ServerHandler};
pub use packet::{ClientPacket, Packet, ServerPacket, crc16_ccitt};
pub use protocol::{ClientReaderProtocol, ClientWriterProtocol, ServerReaderProtocol};
pub use transport::{
    PacketReceiver, PacketSender, RawConnection, RawPacket, SplitTransport, Transport,
};
