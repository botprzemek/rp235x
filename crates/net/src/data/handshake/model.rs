use super::layout;
use crate::layout::DATA_SIZE;

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
    #[must_use]
    pub fn new(protocol_version: u8, device_id: u32, flags: u8) -> Self {
        Self {
            protocol_version,
            device_id,
            flags,
        }
    }

    #[must_use]
    pub fn to_bytes(&self) -> [u8; layout::DATA_SIZE] {
        let mut bytes = [0u8; layout::DATA_SIZE];
        bytes[layout::PROTOCOL_VERSION] = self.protocol_version;
        let device_id_bytes = self.device_id.to_be_bytes();
        bytes[layout::DEVICE_ID_START..layout::DEVICE_ID_END].copy_from_slice(&device_id_bytes);
        bytes[layout::FLAGS] = self.flags;
        bytes
    }

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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HandshakeAck {
    assigned_game_id: u16,
    flags: u8,
}

impl HandshakeAck {
    pub const PAYLOAD_LEN: u8 = 3;

    #[must_use]
    pub const fn new(assigned_game_id: u16, flags: u8) -> Self {
        Self {
            assigned_game_id,
            flags,
        }
    }

    #[must_use]
    pub const fn assigned_game_id(&self) -> u16 {
        self.assigned_game_id
    }

    #[must_use]
    pub const fn flags(&self) -> u8 {
        self.flags
    }

    #[must_use]
    pub fn to_bytes(&self) -> [u8; DATA_SIZE] {
        let mut buffer = [0u8; DATA_SIZE];
        buffer[0..2].copy_from_slice(&self.assigned_game_id.to_be_bytes());
        buffer[2] = self.flags;
        buffer
    }

    pub fn from_bytes(buffer: &[u8; DATA_SIZE]) -> Result<Self, &'static str> {
        let game_id = u16::from_be_bytes([buffer[0], buffer[1]]);
        let flags = buffer[2];
        Ok(Self {
            assigned_game_id: game_id,
            flags,
        })
    }
}
