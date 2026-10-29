//! HUB75 RGB LED matrix display buffer, bitplane modulation, and RP235x GPIO driver.

#![no_std]

pub mod bitplane;
pub mod color;
pub mod display;
pub mod font;
mod led;
pub mod output;

#[cfg(feature = "rp235x")]
pub mod rp235x;

pub use bitplane::BitplaneBuffer;
pub use color::{BLACK, BLUE, Color, GREEN, RED, WHITE, hue_to_rgb, with_brightness};
pub use display::{DisplayBuffer, GAMMA_TABLE};
pub use font::{FONT_DIGITS, FONT_UPPERCASE, Glyph, is_ascii_pixel_set};
pub use output::{LedOutputs, OutputPin, PinLevel};

#[cfg(feature = "rp235x")]
pub use rp235x::LedPeripherals;
