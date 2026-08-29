#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    Start = 0x40,
    Stop = 0x41,
    Reset24 = 0x42,
    Reset14 = 0x43,
    SetTime = 0x44,
}

impl TryFrom<u8> for Event {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x40 => Ok(Event::Start),
            0x41 => Ok(Event::Stop),
            0x42 => Ok(Event::Reset24),
            0x43 => Ok(Event::Reset14),
            0x44 => Ok(Event::SetTime),
            _ => Err(""),
        }
    }
}
