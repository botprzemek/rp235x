#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClientEvent {
    HandshakeRequest = 0x01,
    Heartbeat = 0x02,

    Start = 0x40,
    Stop = 0x41,
    SetHomeScore = 0x42,
    SetAwayScore = 0x43,
}

impl TryFrom<u8> for ClientEvent {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(ClientEvent::HandshakeRequest),
            0x02 => Ok(ClientEvent::Heartbeat),
            0x40 => Ok(ClientEvent::Start),
            0x41 => Ok(ClientEvent::Stop),
            0x42 => Ok(ClientEvent::SetHomeScore),
            0x43 => Ok(ClientEvent::SetAwayScore),
            _ => Err(""),
        }
    }
}
