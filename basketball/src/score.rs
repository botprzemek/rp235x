//! Basketball scoring values and [`Scorable`] trait.

use crate::Team;

/// Points awarded for a single scoring play.
#[non_exhaustive]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(into = "u8"))]
#[cfg_attr(feature = "serde", serde(try_from = "u8"))]
pub enum Score {
    FreeThrow = 1,
    FieldGoal2 = 2,
    FieldGoal3 = 3,
}

/// Home and away team scores.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Scores {
    pub home: u16,
    pub away: u16,
}

/// Interface for entities that track team scores.
pub trait Scorable {
    /// Adds `score` points to `team`.
    fn add_score(&mut self, team: Team, score: Score);

    /// Returns the current score of `team`.
    fn score(&self, team: Team) -> u16;

    /// Returns both home and away [`Scores`].
    fn scores(&self) -> Scores;
}

impl From<Score> for u8 {
    fn from(score: Score) -> Self {
        score as u8
    }
}

impl From<Score> for u16 {
    fn from(score: Score) -> Self {
        score as u16
    }
}

impl TryFrom<u8> for Score {
    type Error = &'static str;

    fn try_from(score: u8) -> Result<Self, Self::Error> {
        match score {
            1 => Ok(Self::FreeThrow),
            2 => Ok(Self::FieldGoal2),
            3 => Ok(Self::FieldGoal3),
            _ => Err(""),
        }
    }
}
