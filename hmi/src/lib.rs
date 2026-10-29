//! Web HMI components for the basketball scoreboard, HUB75 LED matrix simulator, and WebSocket dashboard.

pub mod components;
pub mod matrix;
pub mod scoreboard;
pub mod websocket_test;

pub use scoreboard::Scoreboard;
pub use websocket_test::App;
