use crate::Quarter;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Discipline {
    Fiba5vs5 = 0x00,
    Fiba3x3 = 0x01,
}

impl TryFrom<u8> for Discipline {
    type Error = &'static str;
    fn try_from(discipline: u8) -> Result<Self, Self::Error> {
        match discipline {
            0x00 => Ok(Self::Fiba5vs5),
            0x01 => Ok(Self::Fiba3x3),
            _ => Err(""),
        }
    }
}

impl Discipline {
    pub fn map(&self) -> (Quarter, u32, u32) {
        match self {
            Self::Fiba5vs5 => (Quarter::Q1, 720000, 24000),
            Self::Fiba3x3 => (Quarter::None, 600000, 24000),
        }
    }
}
