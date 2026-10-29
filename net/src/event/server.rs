//! Server-to-client event types.

/// Event code sent by the server in a [`crate::ServerPacket`].
#[non_exhaustive]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ServerEvent {
    HandshakeAck = 0x81,

    GameUpdate = 0xc0,
    GameListUpdate = 0xc1,

    DeviceUpdate = 0xd0,
    DeviceDisconnect = 0xd1,
}

impl TryFrom<u8> for ServerEvent {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x81 => Ok(Self::HandshakeAck),

            0xc0 => Ok(Self::GameUpdate),
            0xc1 => Ok(Self::GameListUpdate),

            0xd0 => Ok(Self::DeviceUpdate),
            0xd1 => Ok(Self::DeviceDisconnect),
            _ => Err(""),
        }
    }
}
