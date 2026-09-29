#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ServerEvent {
    HandshakeAck = 0x81,

    GameUpdate = 0xC0,
}

impl TryFrom<u8> for ServerEvent {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x81 => Ok(ServerEvent::HandshakeAck),

            0xC0 => Ok(ServerEvent::GameUpdate),
            _ => Err(""),
        }
    }
}
