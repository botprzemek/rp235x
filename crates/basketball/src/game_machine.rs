use core::marker::PhantomData;
use core::time::Duration;

use crate::{Discipline, Game, Quarter, Score, ScoreRead, ScoreWrite, Scores, State, Team};

pub mod state_machine {
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
pub struct GameMachine<S> {
    game: Game,
    _marker: PhantomData<S>,
}

impl Default for GameMachine<state_machine::Idle> {
    fn default() -> Self {
        Self::new(Discipline::Fiba5v5)
    }
}

impl GameMachine<state_machine::Idle> {
    #[must_use]
    pub fn new(discipline: Discipline) -> Self {
        Self {
            game: Game::new(discipline),
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn start(mut self) -> GameMachine<state_machine::Running> {
        self.game.set_state(State::Running);
        GameMachine {
            game: self.game,
            _marker: PhantomData,
        }
    }
}

impl GameMachine<state_machine::Running> {
    #[must_use]
    pub fn pause(mut self) -> GameMachine<state_machine::Paused> {
        self.game.set_state(State::Paused);
        GameMachine {
            game: self.game,
            _marker: PhantomData,
        }
    }

    pub fn tick(
        mut self,
        delta: Duration,
    ) -> Result<GameMachine<state_machine::Running>, GameMachine<state_machine::QuarterBreak>> {
        let delta_ms = delta.as_millis() as u32;
        let current_rem = self.game.regulation_millis();

        if delta_ms >= current_rem {
            self.game.tick(Duration::from_millis(current_rem as u64));
            self.game.set_state(State::QuarterEnd);
            Err(GameMachine {
                game: self.game,
                _marker: PhantomData,
            })
        } else {
            self.game.tick(delta);
            Ok(self)
        }
    }
}

impl GameMachine<state_machine::Paused> {
    #[must_use]
    pub fn resume(mut self) -> GameMachine<state_machine::Running> {
        self.game.set_state(State::Running);
        GameMachine {
            game: self.game,
            _marker: PhantomData,
        }
    }
}

impl GameMachine<state_machine::QuarterBreak> {
    pub fn advance_quarter(
        mut self,
    ) -> Result<GameMachine<state_machine::Paused>, GameMachine<state_machine::Finished>> {
        self.game.next_quarter();
        if self.game.state() == State::GameEnd || self.game.quarter() == Quarter::None {
            Err(GameMachine {
                game: self.game,
                _marker: PhantomData,
            })
        } else {
            Ok(GameMachine {
                game: self.game,
                _marker: PhantomData,
            })
        }
    }
}

pub trait ModifiableScore {
    fn add_score(&mut self, team: Team, score: Score);
}

impl<S> ModifiableScore for GameMachine<S>
where
    S: CanModifyScore,
{
    fn add_score(&mut self, team: Team, score: Score) {
        self.game.add_score(team, score);
    }
}

pub trait CanModifyScore {}
impl CanModifyScore for state_machine::Idle {}
impl CanModifyScore for state_machine::Running {}
impl CanModifyScore for state_machine::Paused {}
impl CanModifyScore for state_machine::QuarterBreak {}

impl<S> ScoreRead for GameMachine<S> {
    fn score(&self, team: Team) -> u16 {
        self.game.score(team)
    }

    fn scores(&self) -> Scores {
        self.game.scores()
    }
}

impl<S: CanModifyScore> ScoreWrite for GameMachine<S> {
    fn add_score(&mut self, team: Team, score: Score) {
        self.game.add_score(team, score);
    }
}

impl<S> GameMachine<S> {
    #[must_use]
    pub const fn game(&self) -> &Game {
        &self.game
    }

    #[must_use]
    pub fn into_game(self) -> Game {
        self.game
    }
}
