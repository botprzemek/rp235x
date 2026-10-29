//! Basketball discipline definitions and initial clock/quarter parameters.

use crate::Quarter;

/// Initial game parameters determined by a [`Discipline`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisciplineConfig {
    pub initial_quarter: Quarter,
    pub regulation_millis: u32,
    pub clock_millis: u32,
}

/// Basketball discipline rule set.
#[non_exhaustive]
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
    /// Returns the [`DisciplineConfig`] for this discipline.
    #[must_use]
    pub fn config(&self) -> DisciplineConfig {
        match self {
            Self::Fiba5vs5 => DisciplineConfig {
                initial_quarter: Quarter::Q1,
                regulation_millis: 720_000,
                clock_millis: 24_000,
            },
            Self::Fiba3x3 => DisciplineConfig {
                initial_quarter: Quarter::None,
                regulation_millis: 600_000,
                clock_millis: 24_000,
            },
        }
    }
}
