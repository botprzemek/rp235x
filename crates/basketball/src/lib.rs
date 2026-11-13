#![no_std]

mod discipline;
mod game;
mod game_machine;
mod layout;
mod quarter;
mod score;
mod state;
mod team;

pub use discipline::{Discipline, DisciplineConfig};
pub use game::{ClockTime, Game, RegulationTime};
pub use game_machine::{GameMachine, state_machine};
pub use quarter::Quarter;
pub use score::{Scorable, Score, ScoreRead, ScoreWrite, Scores};
pub use state::State;
pub use team::Team;
