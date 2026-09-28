#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Team {
    Home = 0x00,
    Away = 0x01,
}

impl TryFrom<u8> for Team {
    type Error = &'static str;
    fn try_from(team: u8) -> Result<Self, Self::Error> {
        match team {
            0x00 => Ok(Self::Home),
            0x01 => Ok(Self::Away),
            _ => Err(""),
        }
    }
}
