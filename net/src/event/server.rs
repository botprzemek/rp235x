#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    StateSnapshot = 0xF0,
}

impl TryFrom<u8> for Event {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xF0 => Ok(Event::StateSnapshot),
            _ => Err(""),
        }
    }
}
