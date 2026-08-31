use core::time::Duration;

pub mod layout;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    Idle = 0x00,
    Running = 0x01,
    Paused = 0x02,
    QuarterEnd = 0x03,
    SnapshotEnd = 0x04,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Discipline {
    FIBA5V5 = 0x00,
    FIBA3X3 = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Quarter {
    None = 0x00,
    Q1 = 0x01,
    Q2 = 0x02,
    Q3 = 0x03,
    Q4 = 0x04,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Snapshot {
    pub state: State,
    pub discipline: Discipline,
    pub quarter: Quarter,

    pub home_score: u16,
    pub away_score: u16,

    pub regulation_millis: u32,
    pub clock_millis: u32,
}

impl TryFrom<u8> for State {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(State::Idle),
            0x01 => Ok(State::Running),
            0x02 => Ok(State::Paused),
            0x03 => Ok(State::QuarterEnd),
            0x04 => Ok(State::SnapshotEnd),
            _ => Err("state"),
        }
    }
}

impl TryFrom<u8> for Discipline {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Discipline::FIBA5V5),
            0x01 => Ok(Discipline::FIBA3X3),
            _ => Err(""),
        }
    }
}

impl TryFrom<u8> for Quarter {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Quarter::None),
            0x01 => Ok(Quarter::Q1),
            0x02 => Ok(Quarter::Q2),
            0x03 => Ok(Quarter::Q3),
            0x04 => Ok(Quarter::Q4),
            _ => Err(""),
        }
    }
}

impl Snapshot {
    pub fn new(discipline: Discipline) -> Self {
        let (quarter, regulation_millis, clock_millis) = Self::map_discipline(discipline);

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
    pub fn from_bytes(buffer: &[u8; layout::DATA_SIZE]) -> Result<Self, &'static str> {
        let mut home_score = [0u8; 2];
        let mut away_score = [0u8; 2];
        let mut regulation_millis = [0u8; 4];
        let mut clock_millis = [0u8; 4];

        home_score.copy_from_slice(&buffer[layout::HOME_SCORE_START..layout::HOME_SCORE_END]);
        away_score.copy_from_slice(&buffer[layout::AWAY_SCORE_START..layout::AWAY_SCORE_END]);
        regulation_millis.copy_from_slice(
            &buffer[layout::REGULATION_MILLIS_START..layout::REGULATION_MILLIS_END],
        );
        clock_millis.copy_from_slice(&buffer[layout::CLOCK_MILLIS_START..layout::CLOCK_MILLIS_END]);

        Ok(Self {
            state: State::try_from(buffer[layout::STATE])?,
            discipline: Discipline::try_from(buffer[layout::DISCIPLINE])?,
            quarter: Quarter::try_from(buffer[layout::QUARTER])?,

            home_score: u16::from_be_bytes(home_score),
            away_score: u16::from_be_bytes(away_score),

            regulation_millis: u32::from_be_bytes(regulation_millis),

            clock_millis: u32::from_be_bytes(clock_millis),
        })
    }

    pub fn tick(&mut self, tick_rate: Duration) {
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

        self.regulation_millis = Self::map_discipline(self.discipline).1;
        self.clock_millis = Self::map_discipline(self.discipline).2;
    }

    fn handle_idle(&mut self) {}

    fn handle_running(&mut self, tick_rate: Duration) {
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

    fn map_discipline(discipline: Discipline) -> (Quarter, u32, u32) {
        match discipline {
            Discipline::FIBA5V5 => (Quarter::Q1, 720000, 24000),
            Discipline::FIBA3X3 => (Quarter::None, 600000, 24000),
        }
    }
}
