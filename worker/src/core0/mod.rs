//! Core 0 networking, CYW43 driver initialization, and executor entry point.

pub mod cyw43;
pub mod entry;
pub mod net;

pub use entry::Core0;
