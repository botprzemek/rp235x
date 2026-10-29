//! RGB color constants and color-space helper functions.

/// 24-bit RGB color triple `[red, green, blue]`.
pub type Color = [u8; 3];

/// Pure red RGB color.
pub const RED: Color = [255, 0, 0];

/// Pure green RGB color.
pub const GREEN: Color = [0, 255, 0];

/// Pure blue RGB color.
pub const BLUE: Color = [0, 0, 255];

/// Pure white RGB color.
pub const WHITE: Color = [255, 255, 255];

/// Pure black (off) RGB color.
pub const BLACK: Color = [0, 0, 0];

/// Scales RGB channels proportionally to `brightness_percent` (`0..=100`).
#[must_use]
#[inline(always)]
pub fn with_brightness(color: Color, brightness_percent: u8) -> Color {
    let factor = u16::from(brightness_percent.min(100));
    [
        ((u16::from(color[0]) * factor) / 100) as u8,
        ((u16::from(color[1]) * factor) / 100) as u8,
        ((u16::from(color[2]) * factor) / 100) as u8,
    ]
}

/// Converts a hue angle in degrees (`0..360`) into a saturated RGB [`Color`].
#[must_use]
#[inline(always)]
pub fn hue_to_rgb(hue: u16) -> Color {
    let hue_normalized = hue % 360;
    let sector = (hue_normalized % 60) * 255 / 60;
    let step = sector as u8;

    match hue_normalized / 60 {
        0 => [255, step, 0],
        1 => [255 - step, 255, 0],
        2 => [0, 255, step],
        3 => [0, 255 - step, 255],
        4 => [step, 0, 255],
        _ => [255, 0, 255 - step],
    }
}
