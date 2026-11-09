//! Dual-core RP235x / Linux firmware library / core modules.

#![cfg_attr(feature = "rp235x", no_std)]

pub mod core0;
pub mod core1;
#[cfg(not(feature = "rp235x"))]
pub mod sim;
pub mod state;

#[cfg(feature = "rp235x")]
pub mod interrupts;
#[cfg(feature = "rp235x")]
pub mod peripherals;
