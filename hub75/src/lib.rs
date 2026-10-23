#![no_std]

mod bitplane;
mod color;
mod display;
mod font;
mod led;
mod output;

#[cfg(feature = "rp235x")]
mod rp235x;

pub use bitplane::BitplaneBuffer;
pub use color::{Color, hue_to_rgb};
pub use display::{DisplayBuffer, GAMMA_TABLE};
pub use font::{FONT_DIGITS, FONT_UPPERCASE, Glyph, get_ascii_pixel};
pub use output::{LedOutputs, OutputPin};

#[cfg(feature = "rp235x")]
pub use rp235x::LedPeripherals;
