//! Type-state basketball match wrapper.
//!
//! Provides compile-time enforcement of match phases and invariants:
//! - Scores cannot be added when a match is `Finished`.
//! - Clock cannot be ticked when a match is `Paused` or `Idle`.
//! - Transitions between quarters require explicit advancing through `QuarterBreak`.

use core::marker::PhantomData;
use core::time::Duration;

use crate::{Discipline, Game, Quarter, Scorable, Score, State, Team};

/// Match state marker types.
pub mod match_state {
    /// Match is ready to begin but clock is stopped.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Idle;

    /// Match clock is actively running.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Running;

    /// Match clock is temporarily paused (timeout, foul, whistle).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Paused;

    /// Current quarter ended; awaiting start of next quarter or game end.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct QuarterBreak;

    /// Match has completed regulation / final overtime. No further actions permitted.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Finished;
}

/// Generic compile-time type-state basketball match wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Match<S> {
    game: Game,
    _marker: PhantomData<S>,
}

impl Default for Match<match_state::Idle> {
    fn default() -> Self {
        Self::new(Discipline::Fiba5v5)
    }
}

impl Match<match_state::Idle> {
    /// Creates a new match in [`match_state::Idle`].
    #[must_use]
    pub fn new(discipline: Discipline) -> Self {
        Self {
            game: Game::new(discipline),
            _marker: PhantomData,
        }
    }

    /// Starts or resumes the match clock, transitioning into [`match_state::Running`].
    #[must_use]
    pub fn start(mut self) -> Match<match_state::Running> {
        self.game.set_state(State::Running);
        Match {
            game: self.game,
            _marker: PhantomData,
        }
    }
}

impl Match<match_state::Running> {
    /// Pauses the match clock (e.g. whistle, timeout), transitioning into [`match_state::Paused`].
    #[must_use]
    pub fn pause(mut self) -> Match<match_state::Paused> {
        self.game.set_state(State::Paused);
        Match {
            game: self.game,
            _marker: PhantomData,
        }
    }

    /// Ticks match clock by `delta`. If quarter reaches zero, transitions into [`match_state::QuarterBreak`].
    pub fn tick(
        mut self,
        delta: Duration,
    ) -> Result<Match<match_state::Running>, Match<match_state::QuarterBreak>> {
        let delta_ms = delta.as_millis() as u32;
        let current_rem = self.game.regulation_millis();

        if delta_ms >= current_rem {
            self.game.tick(Duration::from_millis(current_rem as u64));
            self.game.set_state(State::QuarterEnd);
            Err(Match {
                game: self.game,
                _marker: PhantomData,
            })
        } else {
            self.game.tick(delta);
            Ok(self)
        }
    }
}

impl Match<match_state::Paused> {
    /// Resumes the match clock, transitioning into [`match_state::Running`].
    #[must_use]
    pub fn resume(mut self) -> Match<match_state::Running> {
        self.game.set_state(State::Running);
        Match {
            game: self.game,
            _marker: PhantomData,
        }
    }
}

impl Match<match_state::QuarterBreak> {
    /// Advances game to the next quarter or concludes match if final quarter ended.
    pub fn advance_quarter(
        mut self,
    ) -> Result<Match<match_state::Paused>, Match<match_state::Finished>> {
        self.game.next_quarter();
        if self.game.state() == State::GameEnd || self.game.quarter() == Quarter::None {
            Err(Match {
                game: self.game,
                _marker: PhantomData,
            })
        } else {
            Ok(Match {
                game: self.game,
                _marker: PhantomData,
            })
        }
    }
}

/// Trait for states in which score modifications are permitted (Idle, Running, Paused, QuarterBreak).
/// Explicitly NOT implemented for [`match_state::Finished`].
pub trait ModifiableScore {
    /// Adds points to `team`.
    fn add_score(&mut self, team: Team, score: Score);
}

impl<S> ModifiableScore for Match<S>
where
    S: CanModifyScore,
{
    fn add_score(&mut self, team: Team, score: Score) {
        self.game.add_score(team, score);
    }
}

/// Seal trait marking states where scoring is allowed.
pub trait CanModifyScore {}
impl CanModifyScore for match_state::Idle {}
impl CanModifyScore for match_state::Running {}
impl CanModifyScore for match_state::Paused {}
impl CanModifyScore for match_state::QuarterBreak {}

impl<S> Match<S> {
    /// Accesses the underlying [`Game`] snapshot.
    #[must_use]
    pub const fn game(&self) -> &Game {
        &self.game
    }

    /// Converts back to a raw [`Game`] object.
    #[must_use]
    pub fn into_game(self) -> Game {
        self.game
    }
}

#[cfg(test)]
#[path = "match_machine_tests.rs"]
mod match_machine_tests;
