use crate::Team;

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

pub trait Scorable {
    fn score(&mut self, team: Team, score: Score);
    fn get_score(&self, team: Team) -> u16;
    fn get_scores(&self) -> (u16, u16);
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
            1 => Ok(Score::FreeThrow),
            2 => Ok(Score::FieldGoal2),
            3 => Ok(Score::FieldGoal3),
            _ => {
                return Err("");
            }
        }
    }
}
