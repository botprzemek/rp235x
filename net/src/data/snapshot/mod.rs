use crate::ServerPacket;

pub mod layout;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Team {
    Home,
    Away,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum State {
    Idle = 0x00,
    Running = 0x01,
    Paused = 0x02,
    QuarterEnd = 0x03,
    SnapshotEnd = 0x04,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Discipline {
    FIBA5V5 = 0x00,
    FIBA3X3 = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Quarter {
    None = 0x00,
    Q1 = 0x01,
    Q2 = 0x02,
    Q3 = 0x03,
    Q4 = 0x04,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Snapshot {
    state: State,
    discipline: Discipline,
    quarter: Quarter,

    home_score: u16,
    away_score: u16,

    regulation_millis: u32,
    clock_millis: u32,
}

impl TryFrom<u8> for State {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Idle),
            0x01 => Ok(Self::Running),
            0x02 => Ok(Self::Paused),
            0x03 => Ok(Self::QuarterEnd),
            0x04 => Ok(Self::SnapshotEnd),
            _ => Err(""),
        }
    }
}

impl TryFrom<u8> for Discipline {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::FIBA5V5),
            0x01 => Ok(Self::FIBA3X3),
            _ => Err(""),
        }
    }
}

impl TryFrom<u8> for Quarter {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::None),
            0x01 => Ok(Self::Q1),
            0x02 => Ok(Self::Q2),
            0x03 => Ok(Self::Q3),
            0x04 => Ok(Self::Q4),
            _ => Err(""),
        }
    }
}

impl TryFrom<ServerPacket> for Snapshot {
    type Error = &'static str;
    fn try_from(value: ServerPacket) -> Result<Self, Self::Error> {
        let mut home_score = [0u8; 2];
        let mut away_score = [0u8; 2];
        let mut regulation_millis = [0u8; 4];
        let mut clock_millis = [0u8; 4];

        home_score.copy_from_slice(&value.data()[layout::HOME_SCORE_START..layout::HOME_SCORE_END]);
        away_score.copy_from_slice(&value.data()[layout::AWAY_SCORE_START..layout::AWAY_SCORE_END]);
        regulation_millis.copy_from_slice(
            &value.data()[layout::REGULATION_MILLIS_START..layout::REGULATION_MILLIS_END],
        );
        clock_millis
            .copy_from_slice(&value.data()[layout::CLOCK_MILLIS_START..layout::CLOCK_MILLIS_END]);

        Ok(Self {
            state: State::try_from(value.data()[layout::STATE])?,
            discipline: Discipline::try_from(value.data()[layout::DISCIPLINE])?,
            quarter: Quarter::try_from(value.data()[layout::QUARTER])?,

            home_score: u16::from_be_bytes(home_score),
            away_score: u16::from_be_bytes(away_score),

            regulation_millis: u32::from_be_bytes(regulation_millis),

            clock_millis: u32::from_be_bytes(clock_millis),
        })
    }
}

impl Discipline {
    fn map(&self) -> (Quarter, u32, u32) {
        match self {
            Self::FIBA5V5 => (Quarter::Q1, 720000, 24000),
            Self::FIBA3X3 => (Quarter::None, 600000, 24000),
        }
    }
}

impl Snapshot {
    pub fn new(discipline: Discipline) -> Self {
        let (quarter, regulation_millis, clock_millis) = discipline.map();

        Self {
            state: State::Idle,
            discipline,
            quarter,
            home_score: 0,
            away_score: 0,
            regulation_millis,
            clock_millis,
        }
    }

    pub const fn empty() -> Self {
        Self {
            state: State::Idle,
            discipline: Discipline::FIBA5V5,
            quarter: Quarter::Q1,
            home_score: 0,
            away_score: 0,
            regulation_millis: 0,
            clock_millis: 0,
        }
    }

    pub fn get_regulation_time(&self) -> (u8, u8, u8) {
        let total_secs = self.regulation_millis / 1000;
        let minutes = (total_secs / 60) as u8;
        let seconds = (total_secs % 60) as u8;
        let centiseconds = ((self.regulation_millis % 1000) / 10) as u8;

        (minutes, seconds, centiseconds)
    }

    pub fn get_clock_time(&self) -> (u8, u8) {
        let seconds = (self.clock_millis / 1000) as u8;
        let centiseconds = ((self.clock_millis % 1000) / 10) as u8;

        (seconds, centiseconds)
    }

    pub fn to_bytes(&self) -> [u8; layout::DATA_SIZE] {
        let mut bytes = [0u8; layout::DATA_SIZE];

        bytes[layout::STATE] = self.state as u8;
        bytes[layout::DISCIPLINE] = self.discipline as u8;
        bytes[layout::QUARTER] = self.quarter as u8;

        let home_score_bytes = self.home_score.to_be_bytes();
        let away_score_bytes = self.away_score.to_be_bytes();
        let regulation_millis_bytes = self.regulation_millis.to_be_bytes();
        let clock_millis_bytes = self.clock_millis.to_be_bytes();

        bytes[layout::HOME_SCORE_START..layout::HOME_SCORE_END].copy_from_slice(&home_score_bytes);
        bytes[layout::AWAY_SCORE_START..layout::AWAY_SCORE_END].copy_from_slice(&away_score_bytes);
        bytes[layout::REGULATION_MILLIS_START..layout::REGULATION_MILLIS_END]
            .copy_from_slice(&regulation_millis_bytes);
        bytes[layout::CLOCK_MILLIS_START..layout::CLOCK_MILLIS_END]
            .copy_from_slice(&clock_millis_bytes);

        bytes
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state
    }

    pub fn discipline(&self) -> Discipline {
        self.discipline
    }

    pub fn quarter(&self) -> Quarter {
        self.quarter
    }

    pub fn home_score(&self) -> u16 {
        self.home_score
    }

    pub fn set_home_score(&mut self, home_score: u16) {
        self.home_score = home_score
    }

    pub fn away_score(&self) -> u16 {
        self.away_score
    }

    pub fn set_away_score(&mut self, away_score: u16) {
        self.away_score = away_score
    }

    pub fn regulation_millis(&self) -> u32 {
        self.regulation_millis
    }

    pub fn clock_millis(&self) -> u32 {
        self.clock_millis
    }

    pub fn transition_to(&mut self, state: State) {
        self.set_state(state);
    }

    pub fn make_field_goal(&mut self, team: Team, points: u16) {
        match team {
            Team::Home => self.set_home_score(self.home_score().saturating_add(points)),
            Team::Away => self.set_away_score(self.away_score().saturating_add(points)),
        }
    }

    pub fn tick(&mut self, tick_rate: core::time::Duration) {
        match self.state {
            State::Idle => self.handle_idle(),
            State::Running => self.handle_running(tick_rate),
            State::Paused => self.handle_paused(),
            State::QuarterEnd => self.handle_quarter_end(),
            State::SnapshotEnd => self.handle_end(),
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
            _ => {
                self.state = State::SnapshotEnd;
                return;
            }
        };

        self.regulation_millis = self.discipline.map().1;
        self.clock_millis = self.discipline.map().2;
    }

    fn handle_idle(&mut self) {}

    fn handle_running(&mut self, tick_rate: core::time::Duration) {
        let tick_millis = tick_rate.as_millis() as u32;

        if self.regulation_millis == 0 {
            return self.next_quarter();
        };

        if self.regulation_millis >= tick_millis {
            self.regulation_millis -= tick_millis;
        } else {
            self.regulation_millis = 0;
        }
    }

    fn handle_paused(&mut self) {}

    fn handle_quarter_end(&mut self) {}

    fn handle_end(&mut self) {}
}
