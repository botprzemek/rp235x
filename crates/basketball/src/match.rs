use core::marker::PhantomData;
use core::time::Duration;

use crate::{Discipline, Game, Quarter, Scorable, Score, State, Team};

pub mod match_state {

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Idle;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Running;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Paused;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct QuarterBreak;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Finished;
}

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

    #[must_use]
    pub fn new(discipline: Discipline) -> Self {
        Self {
            game: Game::new(discipline),
            _marker: PhantomData,
        }
    }

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

    #[must_use]
    pub fn pause(mut self) -> Match<match_state::Paused> {
        self.game.set_state(State::Paused);
        Match {
            game: self.game,
            _marker: PhantomData,
        }
    }

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

pub trait ModifiableScore {

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

pub trait CanModifyScore {}
impl CanModifyScore for match_state::Idle {}
impl CanModifyScore for match_state::Running {}
impl CanModifyScore for match_state::Paused {}
impl CanModifyScore for match_state::QuarterBreak {}

impl<S> Match<S> {

    #[must_use]
    pub const fn game(&self) -> &Game {
        &self.game
    }

    #[must_use]
    pub fn into_game(self) -> Game {
        self.game
    }
}
