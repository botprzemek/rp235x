//! Network protocol, packet serialization, and transport abstractions

#![no_std]

pub mod config_machine;
pub mod data;
pub mod event;
pub mod layout;
pub mod node;
pub mod packet;
pub mod session;
pub mod transport;

#[macro_use]
pub mod macros;

pub use config_machine::{ConfigMachine, ConfigStorage, config_state};
pub use session::{ClientSession, ServerSession, client_state, server_state};

pub use data::{
    Device, DeviceConfigPacket, DeviceConfigPayload, DeviceType, Handshake, HandshakeAck,
    Heartbeat, HeartbeatAck,
};
pub use event::{ClientEvent, ServerEvent};
pub use node::{
    Client, ClientHandler, NetError, Node, PacketHandler, ReceivedPacket, Server, ServerHandler,
};
pub use packet::{ClientPacket, Packet, ServerPacket};
pub use transport::{
    PacketReceiver, PacketSender, RawConnection, RawConnection as WebSocketConnection, RawPacket,
    SplitTransport, Transport, Transport as WebSocketTransport,
};
