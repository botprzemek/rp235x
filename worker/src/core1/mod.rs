//! Core 1 HUB75 LED matrix rendering loop and layout formatting.

#[cfg(feature = "rp235x")]
mod entry;
mod layout;
#[cfg(feature = "linux")]
pub mod sim;

#[cfg(feature = "rp235x")]
pub use entry::Core1;
pub use layout::ScoreboardLayout;
#[cfg(feature = "linux")]
pub use sim::LinuxScoreboardSimulator;

