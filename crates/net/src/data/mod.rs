//! Payload data models for device registration and handshakes.

pub mod config_packet;
pub mod config_payload;
pub mod device;
pub mod handshake;
pub mod handshake_ack;
pub mod heartbeat;
pub mod heartbeat_ack;

pub use config_packet::DeviceConfigPacket;
pub use config_payload::DeviceConfigPayload;
pub use device::{Device, DeviceType};
pub use handshake::Handshake;
pub use handshake_ack::HandshakeAck;
pub use heartbeat::Heartbeat;
pub use heartbeat_ack::HeartbeatAck;
