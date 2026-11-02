//! Core 0 networking, CYW43 driver initialization, and executor entry point.

pub mod cyw43;
pub mod dhcp;
pub mod entry;
pub mod net;
pub mod udp_handler;

pub use entry::Core0;
