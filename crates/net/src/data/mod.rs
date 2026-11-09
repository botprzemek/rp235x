//! Payload data models for device registration and handshakes.

pub mod config_packet;
pub mod config_payload;
pub mod device;
pub mod handshake;

pub use config_packet::DeviceConfigPacket;
pub use config_payload::DeviceConfigPayload;
pub use device::{Device, DeviceType};
pub use handshake::Handshake;
