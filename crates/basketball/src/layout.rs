//! Byte offsets and sizes for the [`crate::Game`] wire payload.

/// Total payload size in bytes.
pub(crate) const DATA_SIZE: usize = net::layout::DATA_SIZE;

/// Byte index of the [`crate::State`] discriminant.
pub(crate) const STATE: usize = 0x00;

/// Byte index of the [`crate::Discipline`] discriminant.
pub(crate) const DISCIPLINE: usize = 0x01;

/// Byte index of the [`crate::Quarter`] discriminant.
pub(crate) const QUARTER: usize = 0x02;

/// Start byte index (inclusive) of the home score.
pub(crate) const HOME_SCORE_START: usize = 0x03;

/// End byte index (exclusive) of the home score.
pub(crate) const HOME_SCORE_END: usize = 0x05;

/// Start byte index (inclusive) of the away score.
pub(crate) const AWAY_SCORE_START: usize = 0x05;

/// End byte index (exclusive) of the away score.
pub(crate) const AWAY_SCORE_END: usize = 0x07;

/// Start byte index (inclusive) of the regulation clock in milliseconds.
pub(crate) const REGULATION_MILLIS_START: usize = 0x07;

/// End byte index (exclusive) of the regulation clock in milliseconds.
pub(crate) const REGULATION_MILLIS_END: usize = 0x0b;

/// Start byte index (inclusive) of the shot clock in milliseconds.
pub(crate) const CLOCK_MILLIS_START: usize = 0x0b;

/// End byte index (exclusive) of the shot clock in milliseconds.
pub(crate) const CLOCK_MILLIS_END: usize = 0x0f;

/// Start byte index (inclusive) of the game ID.
pub(crate) const ID_START: usize = 0x0f;

/// End byte index (exclusive) of the game ID.
pub(crate) const ID_END: usize = 0x11;

/// Start byte index (inclusive) of the dynamic session authentication token.
pub(crate) const SESSION_TOKEN_START: usize = 0x11;

/// End byte index (exclusive) of the dynamic session authentication token.
pub(crate) const SESSION_TOKEN_END: usize = 0x15;
