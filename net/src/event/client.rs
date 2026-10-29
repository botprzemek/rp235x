//! Client-to-server event types.

/// Event code sent by a client in a [`crate::ClientPacket`].
#[non_exhaustive]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClientEvent {
    HandshakeRequest = 0x01,
    HandshakeHeartbeat = 0x02,

    GameStart = 0x10,
    GameResume = 0x11,
    GamePause = 0x12,
    GameEnd = 0x13,
    GameReset = 0x14,
    GameSelect = 0x15,
    GameCreate = 0x16,

    GameScoreHomeFT = 0x20,
    GameScoreHome2FG = 0x21,
    GameScoreHome3FG = 0x22,

    GameScoreAwayFT = 0x30,
    GameScoreAway2FG = 0x31,
    GameScoreAway3FG = 0x32,

    DeviceDisconnect = 0x40,
    DeviceAssignGame = 0x41,
}

impl TryFrom<u8> for ClientEvent {
    type Error = &'static str;

    fn try_from(event: u8) -> Result<Self, Self::Error> {
        match event {
            0x01 => Ok(Self::HandshakeRequest),
            0x02 => Ok(Self::HandshakeHeartbeat),

            0x10 => Ok(Self::GameStart),
            0x11 => Ok(Self::GameResume),
            0x12 => Ok(Self::GamePause),
            0x13 => Ok(Self::GameEnd),
            0x14 => Ok(Self::GameReset),
            0x15 => Ok(Self::GameSelect),
            0x16 => Ok(Self::GameCreate),

            0x20 => Ok(Self::GameScoreHomeFT),
            0x21 => Ok(Self::GameScoreHome2FG),
            0x22 => Ok(Self::GameScoreHome3FG),

            0x30 => Ok(Self::GameScoreAwayFT),
            0x31 => Ok(Self::GameScoreAway2FG),
            0x32 => Ok(Self::GameScoreAway3FG),

            0x40 => Ok(Self::DeviceDisconnect),
            0x41 => Ok(Self::DeviceAssignGame),

            _ => Err(""),
        }
    }
}
