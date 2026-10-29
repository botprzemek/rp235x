//! [`Handshake`] wire payload structure and serialization.

use crate::data::handshake::layout;

/// Binary handshake payload containing protocol version, device ID, and flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Handshake {
    protocol_version: u8,
    device_id: u32,
    flags: u8,
}

impl Handshake {
    /// Creates a new [`Handshake`] payload.
    #[must_use]
    pub fn new(protocol_version: u8, device_id: u32, flags: u8) -> Self {
        Self {
            protocol_version,
            device_id,
            flags,
        }
    }

    /// Serializes this [`Handshake`] into a 6-byte array.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; layout::DATA_SIZE] {
        let mut bytes = [0u8; layout::DATA_SIZE];

        bytes[layout::PROTOCOL_VERSION] = self.protocol_version;

        let device_id_bytes = self.device_id.to_be_bytes();
        bytes[layout::DEVICE_ID_START..layout::DEVICE_ID_END].copy_from_slice(&device_id_bytes);

        bytes[layout::FLAGS] = self.flags;

        bytes
    }

    /// Deserializes a [`Handshake`] from a 6-byte array.
    pub fn from_bytes(buffer: &[u8; layout::DATA_SIZE]) -> Result<Self, &'static str> {
        let mut device_id_bytes = [0u8; 4];
        device_id_bytes.copy_from_slice(&buffer[layout::DEVICE_ID_START..layout::DEVICE_ID_END]);

        Ok(Self {
            protocol_version: buffer[layout::PROTOCOL_VERSION],
            device_id: u32::from_be_bytes(device_id_bytes),
            flags: buffer[layout::FLAGS],
        })
    }
}
