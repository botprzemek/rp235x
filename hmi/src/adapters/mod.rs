//! Secondary adapters for WebSocket transport and Tauri desktop shell commands.

pub mod tauri;
pub mod ws;

pub use tauri::*;
pub use ws::*;

