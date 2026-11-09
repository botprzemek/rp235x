//! Dual-core RP235x firmware library / core modules.

#![cfg_attr(not(feature = "linux"), no_std)]

pub mod core1;
#[cfg(feature = "linux")]
pub mod sim;
#[cfg(feature = "rp235x")]
pub mod core0;
#[cfg(feature = "rp235x")]
pub mod interrupts;
#[cfg(feature = "rp235x")]
pub mod peripherals;
#[cfg(feature = "rp235x")]
pub mod state;
