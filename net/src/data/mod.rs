//! Payload data models for device registration and handshakes.

pub mod device;
pub mod handshake;

pub use device::{Device, DeviceType};
pub use handshake::Handshake;
