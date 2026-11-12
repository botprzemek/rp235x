//! Heartbeat payload model and wire layout.

use crate::layout::DATA_SIZE;

/// Lightweight heartbeat payload sent periodically by active workers.
///
/// Session authentication is carried in the frame header (`session_token`).
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Heartbeat {
    flags: u8,
}

impl Heartbeat {
    /// Wire size of the heartbeat payload in bytes.
    pub const PAYLOAD_LEN: u8 = 1;

    /// Creates a new [`Heartbeat`] with optional `flags`.
    #[must_use]
    pub const fn new(flags: u8) -> Self {
        Self { flags }
    }

    /// Returns the heartbeat flags/status code.
    #[must_use]
    pub const fn flags(&self) -> u8 {
        self.flags
    }

    /// Serializes this [`Heartbeat`] into a wire payload buffer.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; DATA_SIZE] {
        let mut buffer = [0u8; DATA_SIZE];
        buffer[0] = self.flags;
        buffer
    }

    /// Deserializes a [`Heartbeat`] from a raw payload slice.
    pub fn from_bytes(buffer: &[u8; DATA_SIZE]) -> Result<Self, &'static str> {
        Ok(Self { flags: buffer[0] })
    }
}
