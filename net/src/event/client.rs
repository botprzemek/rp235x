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

    GameScoreHomeFT = 0x20,
    GameScoreHome2FG = 0x21,
    GameScoreHome3FG = 0x22,

    GameScoreAwayFT = 0x30,
    GameScoreAway2FG = 0x31,
    GameScoreAway3FG = 0x32,
}

impl TryFrom<u8> for ClientEvent {
    type Error = &'static str;
    fn try_from(event: u8) -> Result<Self, Self::Error> {
        match event {
            0x01 => Ok(ClientEvent::HandshakeRequest),
            0x02 => Ok(ClientEvent::HandshakeHeartbeat),

            0x10 => Ok(ClientEvent::GameStart),
            0x11 => Ok(ClientEvent::GameResume),
            0x12 => Ok(ClientEvent::GamePause),
            0x13 => Ok(ClientEvent::GameEnd),
            0x14 => Ok(ClientEvent::GameReset),

            0x20 => Ok(ClientEvent::GameScoreHomeFT),
            0x21 => Ok(ClientEvent::GameScoreHome2FG),
            0x22 => Ok(ClientEvent::GameScoreHome3FG),

            0x30 => Ok(ClientEvent::GameScoreAwayFT),
            0x31 => Ok(ClientEvent::GameScoreAway2FG),
            0x32 => Ok(ClientEvent::GameScoreAway3FG),

            _ => Err(""),
        }
    }
}
