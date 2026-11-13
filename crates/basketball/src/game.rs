use core::time::Duration;

use crate::{Discipline, Quarter, Score, ScoreRead, ScoreWrite, Scores, State, Team, layout};

use net::{ClientPacket, ServerPacket};

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RegulationTime {
    pub minutes: u8,
    pub seconds: u8,
    pub centiseconds: u8,
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ClockTime {
    pub seconds: u8,
    pub centiseconds: u8,
}

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

impl ScoreWrite for Game {
    fn add_score(&mut self, team: Team, score: Score) {
        match team {
            Team::Home => self.home_score = self.home_score.saturating_add(score.into()),
            Team::Away => self.away_score = self.away_score.saturating_add(score.into()),
        }
    }
}

impl ScoreRead for Game {
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

    #[must_use]
    pub fn new(discipline: Discipline) -> Self {
        Self::new_with_id(1, discipline)
    }

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

    #[must_use]
    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn set_id(&mut self, id: u16) {
        self.id = id;
    }

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

    #[must_use]
    pub fn clock_time(&self) -> ClockTime {
        let seconds = (self.clock_millis / 1000) as u8;
        let centiseconds = ((self.clock_millis % 1000) / 10) as u8;

        ClockTime {
            seconds,
            centiseconds,
        }
    }

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

    #[must_use]
    pub fn state(&self) -> State {
        self.state
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    #[must_use]
    pub fn discipline(&self) -> Discipline {
        self.discipline
    }

    #[must_use]
    pub fn quarter(&self) -> Quarter {
        self.quarter
    }

    #[must_use]
    pub fn regulation_millis(&self) -> u32 {
        self.regulation_millis
    }

    #[must_use]
    pub fn clock_millis(&self) -> u32 {
        self.clock_millis
    }

    #[must_use]
    pub fn display_regulation_time(&self) -> (u8, u8, u8) {
        let total_seconds = self.regulation_millis / 1000;
        let minutes = (total_seconds / 60) as u8;
        let seconds = (total_seconds % 60) as u8;
        let centiseconds = ((self.regulation_millis % 1000) / 10) as u8;
        (minutes, seconds, centiseconds)
    }

    #[must_use]
    pub fn display_shot_clock(&self) -> (u8, u8) {
        let seconds = (self.clock_millis / 1000) as u8;
        let centiseconds = ((self.clock_millis % 1000) / 10) as u8;
        (seconds, centiseconds)
    }

    pub fn tick(&mut self, delta: Duration) {
        match self.state {
            State::Idle => self.handle_idle(),
            State::Running => self.handle_running(delta),
            State::Paused => self.handle_paused(),
            State::QuarterEnd => self.handle_quarter_end(),
            State::GameEnd => self.handle_end(),
        }
    }

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

    pub fn pause(&mut self) {
        self.set_state(State::Paused);
    }

    pub fn resume(&mut self) {
        self.set_state(State::Running);
    }
}
