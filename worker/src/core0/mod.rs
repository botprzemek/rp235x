//! Core 0 networking, CYW43 driver initialization, and executor entry point.

#[cfg(feature = "rp235x")]
pub mod cyw43;
#[cfg(feature = "rp235x")]
pub mod dhcp;
pub mod entry;
#[cfg(feature = "rp235x")]
pub mod net;
#[cfg(feature = "rp235x")]
pub mod udp_handler;

pub use entry::Core0;
