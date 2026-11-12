pub(crate) const DATA_SIZE: usize = net::layout::DATA_SIZE;

pub(crate) const STATE: usize = 0x00;

pub(crate) const DISCIPLINE: usize = 0x01;

pub(crate) const QUARTER: usize = 0x02;

pub(crate) const HOME_SCORE_START: usize = 0x03;

pub(crate) const HOME_SCORE_END: usize = 0x05;

pub(crate) const AWAY_SCORE_START: usize = 0x05;

pub(crate) const AWAY_SCORE_END: usize = 0x07;

pub(crate) const REGULATION_MILLIS_START: usize = 0x07;

pub(crate) const REGULATION_MILLIS_END: usize = 0x0b;

pub(crate) const CLOCK_MILLIS_START: usize = 0x0b;

pub(crate) const CLOCK_MILLIS_END: usize = 0x0f;

pub(crate) const ID_START: usize = 0x0f;

pub(crate) const ID_END: usize = 0x11;
