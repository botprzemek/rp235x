//! Core 1 HUB75 LED matrix rendering loop and layout formatting.

pub mod entry;
mod layout;
#[cfg(not(feature = "rp235x"))]
pub mod sim;

pub use entry::Core1;
pub use layout::ScoreboardLayout;
#[cfg(not(feature = "rp235x"))]
pub use sim::LinuxScoreboardSimulator;

