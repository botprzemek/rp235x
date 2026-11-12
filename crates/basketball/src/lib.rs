#![no_std]

mod discipline;
mod game;
mod layout;
mod r#match;
mod quarter;
mod score;
mod state;
mod team;

pub use discipline::{Discipline, DisciplineConfig};
pub use game::{ClockTime, Game, RegulationTime};
pub use r#match::{Match, ModifiableScore, match_state};
pub use quarter::Quarter;
pub use score::{Scorable, Score, Scores};
pub use state::State;
pub use team::Team;
