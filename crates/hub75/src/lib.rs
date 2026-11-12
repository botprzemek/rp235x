//! HUB75 RGB LED matrix display buffer, bitplane modulation, and RP235x GPIO driver.

#![no_std]

mod bitplane;
mod color;
mod display;
mod font;
mod led;
mod output;

#[cfg(feature = "rp235x")]
pub mod rp235x;

pub use color::{BLACK, BLUE, CORAL_RED, Color, GREEN, RED, WHITE, hue_to_rgb, with_brightness};
pub use display::DisplayBuffer;
pub use output::{LedOutputs, OutputPin, PinLevel};

#[cfg(feature = "rp235x")]
pub use rp235x::LedPeripherals;
