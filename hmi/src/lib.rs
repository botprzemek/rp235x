//! Clean Leptos-based macOS HMI application for basketball scoreboard, HUB75 matrix simulator, and WebSocket dashboard.

pub mod adapters;
pub mod app;
pub mod core;
pub mod ui;

pub use app::App;
pub use ui::matrix::MatrixSimulator;
pub use ui::scoreboard::Scoreboard;
