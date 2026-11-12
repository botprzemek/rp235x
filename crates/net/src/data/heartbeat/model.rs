use crate::layout::DATA_SIZE;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Heartbeat {
    flags: u8,
}

impl Heartbeat {
    pub const PAYLOAD_LEN: u8 = 1;

    #[must_use]
    pub const fn new(flags: u8) -> Self {
        Self { flags }
    }

    #[must_use]
    pub const fn flags(&self) -> u8 {
        self.flags
    }

    #[must_use]
    pub fn to_bytes(&self) -> [u8; DATA_SIZE] {
        let mut buffer = [0u8; DATA_SIZE];
        buffer[0] = self.flags;
        buffer
    }

    pub fn from_bytes(buffer: &[u8; DATA_SIZE]) -> Result<Self, &'static str> {
        Ok(Self { flags: buffer[0] })
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HeartbeatAck {
    latest_game_seq: u16,
    flags: u8,
}

impl HeartbeatAck {
    pub const PAYLOAD_LEN: u8 = 3;

    #[must_use]
    pub const fn new(latest_game_seq: u16, flags: u8) -> Self {
        Self {
            latest_game_seq,
            flags,
        }
    }

    #[must_use]
    pub const fn latest_game_seq(&self) -> u16 {
        self.latest_game_seq
    }

    #[must_use]
    pub const fn flags(&self) -> u8 {
        self.flags
    }

    #[must_use]
    pub fn to_bytes(&self) -> [u8; DATA_SIZE] {
        let mut buffer = [0u8; DATA_SIZE];
        buffer[0..2].copy_from_slice(&self.latest_game_seq.to_be_bytes());
        buffer[2] = self.flags;
        buffer
    }

    pub fn from_bytes(buffer: &[u8; DATA_SIZE]) -> Result<Self, &'static str> {
        let seq = u16::from_be_bytes([buffer[0], buffer[1]]);
        let flags = buffer[2];
        Ok(Self {
            latest_game_seq: seq,
            flags,
        })
    }
}
