#![no_std]

pub const DATA_SIZE: usize = 22;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    Idle = 0x00,
    Running = 0x01,
    Paused = 0x02,
    QuarterEnd = 0x03,
    GameEnd = 0x04,
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
pub struct Game {
    pub state: State,
    pub discipline: Discipline,
    pub quarter: Quarter,

    pub home_score: u16,
    pub away_score: u16,

    pub millis: u32,

    pub _pad: [u8; 11],
}

impl TryFrom<u8> for State {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(State::Idle),
            0x01 => Ok(State::Running),
            0x02 => Ok(State::Paused),
            0x03 => Ok(State::QuarterEnd),
            0x04 => Ok(State::GameEnd),
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
            _ => Err("type"),
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
            _ => Err("quarter"),
        }
    }
}

fn map_discipline_millis(discipline: Discipline) -> (Quarter, u32) {
    match discipline {
        Discipline::FIBA5V5 => (Quarter::Q1, 720000),
        Discipline::FIBA3X3 => (Quarter::None, 600000),
    }
}

impl Game {
    pub fn new(discipline: Discipline) -> Self {
        let (quarter, millis) = map_discipline_millis(discipline);

        Self {
            state: State::Running,
            discipline,
            quarter,
            home_score: 0,
            away_score: 0,
            millis,
            _pad: [0; 11],
        }
    }

    pub fn get_formatted_time(&self) -> (u8, u8, u8) {
        let total_secs = self.millis / 1000;
        let minutes = (total_secs / 60) as u8;
        let seconds = (total_secs % 60) as u8;
        let centiseconds = ((self.millis % 1000) / 10) as u8;
        (minutes, seconds, centiseconds)
    }

    pub fn to_bytes(&self) -> [u8; DATA_SIZE] {
        let mut bytes = [0u8; DATA_SIZE];

        bytes[0] = self.state as u8;
        bytes[1] = self.discipline as u8;
        bytes[2] = self.quarter as u8;

        let home_score_bytes = self.home_score.to_be_bytes();
        let away_score_bytes = self.away_score.to_be_bytes();
        let millis_bytes = self.millis.to_be_bytes();

        bytes[3..5].copy_from_slice(&home_score_bytes);
        bytes[5..7].copy_from_slice(&away_score_bytes);
        bytes[7..11].copy_from_slice(&millis_bytes);
        bytes[11..DATA_SIZE].copy_from_slice(&self._pad);

        bytes
    }

    pub fn from_payload(data: &[u8; DATA_SIZE]) -> Result<Self, &'static str> {
        let mut home_score = [0u8; 2];
        let mut away_score = [0u8; 2];
        let mut millis = [0u8; 4];
        let mut _pad = [0u8; 11];

        home_score.copy_from_slice(&data[3..5]);
        away_score.copy_from_slice(&data[5..7]);
        millis.copy_from_slice(&data[7..11]);
        _pad.copy_from_slice(&data[11..DATA_SIZE]);

        Ok(Self {
            state: State::try_from(data[0])?,
            discipline: Discipline::try_from(data[1])?,
            quarter: Quarter::try_from(data[2])?,

            home_score: u16::from_be_bytes(home_score),
            away_score: u16::from_be_bytes(away_score),

            millis: u32::from_be_bytes(millis),

            _pad,
        })
    }

    fn idle_handler(&mut self) {}

    fn running_handler(&mut self, tick_rate: u32) {
        if self.millis == 0 {
            return self.next_quarter();
        };

        if self.millis >= tick_rate {
            self.millis -= tick_rate;
        } else {
            self.millis = 0;
        }
    }

    fn paused_handler(&mut self) {}

    fn quarter_end_handler(&mut self) {}

    fn game_end_handler(&mut self) {}

    pub fn tick(&mut self, tick_rate: u32) {
        match self.state {
            State::Idle => self.idle_handler(),
            State::Running => self.running_handler(tick_rate),
            State::Paused => self.paused_handler(),
            State::QuarterEnd => self.quarter_end_handler(),
            State::GameEnd => self.game_end_handler(),
        }
    }

    pub fn next_quarter(&mut self) {
        if self.millis != 0 {
            return;
        }

        self.state = State::Paused;

        self.quarter = match self.quarter {
            Quarter::Q1 => Quarter::Q2,
            Quarter::Q2 => Quarter::Q3,
            Quarter::Q3 => Quarter::Q4,
            _ => return,
        };

        self.millis = map_discipline_millis(self.discipline).1;
    }
}
