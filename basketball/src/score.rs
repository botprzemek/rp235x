use crate::Team;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Score {
    FreeThrow = 1,
    FieldGoal2 = 2,
    FieldGoal3 = 3,
}

pub trait Scorable {
    fn score(&mut self, team: Team, score: Score);
    fn get_score(&self, team: Team) -> u16;
}

impl From<Score> for u16 {
    fn from(score: Score) -> Self {
        score as u16
    }
}
