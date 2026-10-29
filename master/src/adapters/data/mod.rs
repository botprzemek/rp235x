//! Database storage providers.

mod redb;

pub use redb::{DEVICES_TABLE, GAME_TABLE, ReDBProvider};
