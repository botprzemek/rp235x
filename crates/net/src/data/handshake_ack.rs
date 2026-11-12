//! Handshake acknowledgment payload.

use crate::layout::DATA_SIZE;

/// Payload sent by the master in [`crate::ServerEvent::HandshakeAck`].
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HandshakeAck {
    assigned_game_id: u16,
    flags: u8,
}

impl HandshakeAck {
    /// Wire payload length in bytes.
    pub const PAYLOAD_LEN: u8 = 3;

    /// Creates a new [`HandshakeAck`].
    #[must_use]
    pub const fn new(assigned_game_id: u16, flags: u8) -> Self {
        Self {
            assigned_game_id,
            flags,
        }
    }

    /// Returns the game ID assigned by Master.
    #[must_use]
    pub const fn assigned_game_id(&self) -> u16 {
        self.assigned_game_id
    }

    /// Returns the flags/status code.
    #[must_use]
    pub const fn flags(&self) -> u8 {
        self.flags
    }

    /// Serializes this [`HandshakeAck`] into a wire payload buffer.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; DATA_SIZE] {
        let mut buffer = [0u8; DATA_SIZE];
        buffer[0..2].copy_from_slice(&self.assigned_game_id.to_be_bytes());
        buffer[2] = self.flags;
        buffer
    }

    /// Deserializes a [`HandshakeAck`] from a raw payload slice.
    pub fn from_bytes(buffer: &[u8; DATA_SIZE]) -> Result<Self, &'static str> {
        let game_id = u16::from_be_bytes([buffer[0], buffer[1]]);
        let flags = buffer[2];
        Ok(Self {
            assigned_game_id: game_id,
            flags,
        })
    }
}
