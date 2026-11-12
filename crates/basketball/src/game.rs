//! Basketball [`Game`] state machine, clock decomposition, and wire serialization.

use core::time::Duration;

use crate::{Discipline, Quarter, Scorable, Score, Scores, State, Team, layout};

use net::{ClientPacket, ServerPacket};

/// Decomposed regulation clock time in minutes, seconds, and centiseconds.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RegulationTime {
    pub minutes: u8,
    pub seconds: u8,
    pub centiseconds: u8,
}

/// Decomposed shot clock time in seconds and centiseconds.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ClockTime {
    pub seconds: u8,
    pub centiseconds: u8,
}

/// State and score of a basketball game.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Game {
    id: u16,
    state: State,
    discipline: Discipline,
    quarter: Quarter,

    home_score: u16,
    away_score: u16,

    regulation_millis: u32,
    clock_millis: u32,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Discipline::Fiba5v5)
    }
}

impl TryFrom<ClientPacket> for Game {
    type Error = &'static str;

    fn try_from(packet: ClientPacket) -> Result<Self, Self::Error> {
        Self::from_bytes(&packet.data())
    }
}

impl TryFrom<ServerPacket> for Game {
    type Error = &'static str;

    fn try_from(packet: ServerPacket) -> Result<Self, Self::Error> {
        Self::from_bytes(&packet.data())
    }
}

impl Scorable for Game {
    fn add_score(&mut self, team: Team, score: Score) {
        match team {
            Team::Home => self.home_score = self.home_score.saturating_add(score.into()),
            Team::Away => self.away_score = self.away_score.saturating_add(score.into()),
        }
    }

    fn score(&self, team: Team) -> u16 {
        match team {
            Team::Home => self.home_score,
            Team::Away => self.away_score,
        }
    }

    fn scores(&self) -> Scores {
        Scores {
            home: self.home_score,
            away: self.away_score,
        }
    }
}

impl Game {
    /// Creates a new [`Game`] with ID `1` and the given `discipline`.
    #[must_use]
    pub fn new(discipline: Discipline) -> Self {
        Self::new_with_id(1, discipline)
    }

    /// Creates a new [`Game`] with the given `id` and `discipline`.
    #[must_use]
    pub fn new_with_id(id: u16, discipline: Discipline) -> Self {
        let config = discipline.config();

        Self {
            id,
            state: State::Idle,
            discipline,
            quarter: config.initial_quarter,
            home_score: 0,
            away_score: 0,
            regulation_millis: config.regulation_millis,
            clock_millis: config.clock_millis,
        }
    }

    /// Creates a zero-clock idle [`Game`] suitable for static initialization.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            id: 1,
            state: State::Idle,
            discipline: Discipline::Fiba5v5,
            quarter: Quarter::Q1,
            home_score: 0,
            away_score: 0,
            regulation_millis: 0,
            clock_millis: 0,
        }
    }

    /// Returns the game identifier.
    #[must_use]
    pub fn id(&self) -> u16 {
        self.id
    }

    /// Sets the game identifier.
    pub fn set_id(&mut self, id: u16) {
        self.id = id;
    }

    /// Returns the decomposed regulation clock [`RegulationTime`].
    #[must_use]
    pub fn regulation_time(&self) -> RegulationTime {
        let total_seconds = self.regulation_millis / 1000;
        let minutes = (total_seconds / 60) as u8;
        let seconds = (total_seconds % 60) as u8;
        let centiseconds = ((self.regulation_millis % 1000) / 10) as u8;

        RegulationTime {
            minutes,
            seconds,
            centiseconds,
        }
    }

    /// Returns the decomposed shot clock [`ClockTime`].
    #[must_use]
    pub fn clock_time(&self) -> ClockTime {
        let seconds = (self.clock_millis / 1000) as u8;
        let centiseconds = ((self.clock_millis % 1000) / 10) as u8;

        ClockTime {
            seconds,
            centiseconds,
        }
    }

    /// Deserializes a [`Game`] from a wire payload byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut home_score = [0u8; 2];
        let mut away_score = [0u8; 2];
        let mut regulation_millis = [0u8; 4];
        let mut clock_millis = [0u8; 4];
        let mut id_bytes = [0u8; 2];

        home_score.copy_from_slice(&bytes[layout::HOME_SCORE_START..layout::HOME_SCORE_END]);
        away_score.copy_from_slice(&bytes[layout::AWAY_SCORE_START..layout::AWAY_SCORE_END]);

        regulation_millis.copy_from_slice(
            &bytes[layout::REGULATION_MILLIS_START..layout::REGULATION_MILLIS_END],
        );
        clock_millis.copy_from_slice(&bytes[layout::CLOCK_MILLIS_START..layout::CLOCK_MILLIS_END]);
        id_bytes.copy_from_slice(&bytes[layout::ID_START..layout::ID_END]);

        let raw_id = u16::from_be_bytes(id_bytes);
        let id = if raw_id == 0 { 1 } else { raw_id };

        Ok(Self {
            id,
            state: State::try_from(bytes[layout::STATE])?,
            discipline: Discipline::try_from(bytes[layout::DISCIPLINE])?,
            quarter: Quarter::try_from(bytes[layout::QUARTER])?,

            home_score: u16::from_be_bytes(home_score),
            away_score: u16::from_be_bytes(away_score),

            regulation_millis: u32::from_be_bytes(regulation_millis),
            clock_millis: u32::from_be_bytes(clock_millis),
        })
    }

    /// Serializes this [`Game`] into a wire payload array.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; layout::DATA_SIZE] {
        let mut bytes = [0u8; layout::DATA_SIZE];

        bytes[layout::STATE] = self.state as u8;
        bytes[layout::DISCIPLINE] = self.discipline as u8;
        bytes[layout::QUARTER] = self.quarter as u8;

        let home_score_bytes = self.home_score.to_be_bytes();
        let away_score_bytes = self.away_score.to_be_bytes();
        let regulation_millis_bytes = self.regulation_millis.to_be_bytes();
        let clock_millis_bytes = self.clock_millis.to_be_bytes();
        let id_bytes = self.id.to_be_bytes();

        bytes[layout::HOME_SCORE_START..layout::HOME_SCORE_END].copy_from_slice(&home_score_bytes);
        bytes[layout::AWAY_SCORE_START..layout::AWAY_SCORE_END].copy_from_slice(&away_score_bytes);
        bytes[layout::REGULATION_MILLIS_START..layout::REGULATION_MILLIS_END]
            .copy_from_slice(&regulation_millis_bytes);
        bytes[layout::CLOCK_MILLIS_START..layout::CLOCK_MILLIS_END]
            .copy_from_slice(&clock_millis_bytes);
        bytes[layout::ID_START..layout::ID_END].copy_from_slice(&id_bytes);

        bytes
    }

    /// Returns the current [`State`] of the game.
    #[must_use]
    pub fn state(&self) -> State {
        self.state
    }

    /// Sets the [`State`] of the game.
    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    /// Returns the [`Discipline`] of the game.
    #[must_use]
    pub fn discipline(&self) -> Discipline {
        self.discipline
    }

    /// Returns the current [`Quarter`] of the game.
    #[must_use]
    pub fn quarter(&self) -> Quarter {
        self.quarter
    }

    /// Returns the remaining regulation time in milliseconds.
    #[must_use]
    pub fn regulation_millis(&self) -> u32 {
        self.regulation_millis
    }

    /// Returns the remaining shot clock time in milliseconds.
    #[must_use]
    pub fn clock_millis(&self) -> u32 {
        self.clock_millis
    }

    /// Pure presentation helper: decomposes regulation milliseconds into (minutes, seconds, centiseconds).
    #[must_use]
    pub fn display_regulation_time(&self) -> (u8, u8, u8) {
        let total_seconds = self.regulation_millis / 1000;
        let minutes = (total_seconds / 60) as u8;
        let seconds = (total_seconds % 60) as u8;
        let centiseconds = ((self.regulation_millis % 1000) / 10) as u8;
        (minutes, seconds, centiseconds)
    }

    /// Pure presentation helper: decomposes shot clock milliseconds into (seconds, centiseconds).
    #[must_use]
    pub fn display_shot_clock(&self) -> (u8, u8) {
        let seconds = (self.clock_millis / 1000) as u8;
        let centiseconds = ((self.clock_millis % 1000) / 10) as u8;
        (seconds, centiseconds)
    }

    /// Advances the game clock by `delta` if the game is currently [`State::Running`].
    pub fn tick(&mut self, delta: Duration) {
        match self.state {
            State::Idle => self.handle_idle(),
            State::Running => self.handle_running(delta),
            State::Paused => self.handle_paused(),
            State::QuarterEnd => self.handle_quarter_end(),
            State::GameEnd => self.handle_end(),
        }
    }

    /// Advances the game to the next quarter once regulation time reaches zero.
    pub fn next_quarter(&mut self) {
        if self.regulation_millis != 0 {
            return;
        }

        self.state = State::Paused;

        self.quarter = match self.quarter {
            Quarter::Q1 => Quarter::Q2,
            Quarter::Q2 => Quarter::Q3,
            Quarter::Q3 => Quarter::Q4,
            Quarter::Q4 | Quarter::None => {
                self.state = State::GameEnd;
                return;
            }
        };

        let config = self.discipline.config();
        self.regulation_millis = config.regulation_millis;
        self.clock_millis = config.clock_millis;
    }

    fn handle_idle(&mut self) {}

    fn handle_running(&mut self, delta: Duration) {
        let delta_millis = delta.as_millis() as u32;

        self.regulation_millis = self.regulation_millis.saturating_sub(delta_millis);

        if self.regulation_millis == 0 {
            self.next_quarter();
        }
    }

    fn handle_paused(&mut self) {}

    fn handle_quarter_end(&mut self) {}

    fn handle_end(&mut self) {}

    /// Pauses the game clock.
    pub fn pause(&mut self) {
        self.set_state(State::Paused);
    }

    /// Resumes (starts) the game clock.
    pub fn resume(&mut self) {
        self.set_state(State::Running);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_score_and_quarters() {
        let mut game = Game::new(Discipline::Fiba5v5);
        assert_eq!(game.quarter(), Quarter::Q1);
        assert_eq!(game.score(Team::Home), 0);
        assert_eq!(game.score(Team::Away), 0);

        game.add_score(Team::Home, Score::FieldGoal3);
        game.add_score(Team::Away, Score::FieldGoal2);
        assert_eq!(game.score(Team::Home), 3);
        assert_eq!(game.score(Team::Away), 2);

        // Serialization roundtrip
        let bytes = game.to_bytes();
        let restored = Game::from_bytes(&bytes).expect("valid game");
        assert_eq!(restored.score(Team::Home), 3);
        assert_eq!(restored.score(Team::Away), 2);
    }

    #[test]
    fn test_clock_tick_and_quarter_transition() {
        let mut game = Game::new(Discipline::Fiba5v5);
        game.resume();
        assert_eq!(game.state(), State::Running);

        // Tick 1 second
        let initial_millis = game.regulation_millis();
        game.tick(Duration::from_secs(1));
        assert_eq!(game.regulation_millis(), initial_millis - 1000);

        // Fast forward remainder of Q1
        game.tick(Duration::from_millis(game.regulation_millis() as u64));
        assert_eq!(game.quarter(), Quarter::Q2);
        assert_eq!(game.state(), State::Paused);
    }
}
