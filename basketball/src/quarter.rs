#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Quarter {
    None = 0x00,
    Q1 = 0x01,
    Q2 = 0x02,
    Q3 = 0x03,
    Q4 = 0x04,
}

impl TryFrom<u8> for Quarter {
    type Error = &'static str;
    fn try_from(quarter: u8) -> Result<Self, Self::Error> {
        match quarter {
            0x00 => Ok(Self::None),
            0x01 => Ok(Self::Q1),
            0x02 => Ok(Self::Q2),
            0x03 => Ok(Self::Q3),
            0x04 => Ok(Self::Q4),
            _ => Err(""),
        }
    }
}
