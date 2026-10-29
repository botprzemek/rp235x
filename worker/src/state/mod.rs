//! Worker firmware state machine and UDP synchronization handlers.

pub mod handler;
mod machine;

pub use machine::{GAME_STATE, Input, Machine};
