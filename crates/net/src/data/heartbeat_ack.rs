//! Heartbeat acknowledgment payload.

use crate::layout::DATA_SIZE;

/// Payload sent by the master in [`crate::ServerEvent::HeartbeatAck`].
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HeartbeatAck {
    latest_game_seq: u16,
    flags: u8,
}

impl HeartbeatAck {
    /// Wire payload length in bytes.
    pub const PAYLOAD_LEN: u8 = 3;

    /// Creates a new [`HeartbeatAck`].
    #[must_use]
    pub const fn new(latest_game_seq: u16, flags: u8) -> Self {
        Self {
            latest_game_seq,
            flags,
        }
    }

    /// Sequence ID of the latest game state update.
    #[must_use]
    pub const fn latest_game_seq(&self) -> u16 {
        self.latest_game_seq
    }

    /// Flags / status bits.
    #[must_use]
    pub const fn flags(&self) -> u8 {
        self.flags
    }

    /// Serializes this [`HeartbeatAck`] into a wire payload buffer.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; DATA_SIZE] {
        let mut buffer = [0u8; DATA_SIZE];
        buffer[0..2].copy_from_slice(&self.latest_game_seq.to_be_bytes());
        buffer[2] = self.flags;
        buffer
    }

    /// Deserializes a [`HeartbeatAck`] from a raw payload slice.
    pub fn from_bytes(buffer: &[u8; DATA_SIZE]) -> Result<Self, &'static str> {
        let seq = u16::from_be_bytes([buffer[0], buffer[1]]);
        let flags = buffer[2];
        Ok(Self {
            latest_game_seq: seq,
            flags,
        })
    }
}
