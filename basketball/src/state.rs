#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum State {
    Idle = 0x00,
    Running = 0x01,
    Paused = 0x02,
    QuarterEnd = 0x03,
    GameEnd = 0x04,
}

impl TryFrom<u8> for State {
    type Error = &'static str;
    fn try_from(state: u8) -> Result<Self, Self::Error> {
        match state {
            0x00 => Ok(Self::Idle),
            0x01 => Ok(Self::Running),
            0x02 => Ok(Self::Paused),
            0x03 => Ok(Self::QuarterEnd),
            0x04 => Ok(Self::GameEnd),
            _ => Err(""),
        }
    }
}
