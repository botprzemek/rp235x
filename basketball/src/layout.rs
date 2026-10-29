//! Byte offsets and sizes for the [`crate::Game`] wire payload.

/// Total payload size in bytes.
pub const DATA_SIZE: usize = net::layout::DATA_SIZE;

/// Byte index of the [`crate::State`] discriminant.
pub const STATE: usize = 0x00;

/// Byte index of the [`crate::Discipline`] discriminant.
pub const DISCIPLINE: usize = 0x01;

/// Byte index of the [`crate::Quarter`] discriminant.
pub const QUARTER: usize = 0x02;

/// Start byte index (inclusive) of the home score.
pub const HOME_SCORE_START: usize = 0x03;

/// End byte index (exclusive) of the home score.
pub const HOME_SCORE_END: usize = 0x05;

/// Start byte index (inclusive) of the away score.
pub const AWAY_SCORE_START: usize = 0x05;

/// End byte index (exclusive) of the away score.
pub const AWAY_SCORE_END: usize = 0x07;

/// Start byte index (inclusive) of the regulation clock in milliseconds.
pub const REGULATION_MILLIS_START: usize = 0x07;

/// End byte index (exclusive) of the regulation clock in milliseconds.
pub const REGULATION_MILLIS_END: usize = 0x0b;

/// Start byte index (inclusive) of the shot clock in milliseconds.
pub const CLOCK_MILLIS_START: usize = 0x0b;

/// End byte index (exclusive) of the shot clock in milliseconds.
pub const CLOCK_MILLIS_END: usize = 0x0f;

/// Start byte index (inclusive) of the game ID.
pub const ID_START: usize = 0x0f;

/// End byte index (exclusive) of the game ID.
pub const ID_END: usize = 0x11;
