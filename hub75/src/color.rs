pub type Color = [u8; 3];

pub const RED: Color = [255, 0, 0];
pub const GREEN: Color = [0, 255, 0];
pub const BLUE: Color = [0, 0, 255];
pub const WHITE: Color = [255, 255, 255];
pub const BLACK: Color = [0, 0, 0];

/// Skaluje składowe koloru RGB proporcjonalnie do zadanej jasności w procentach (0..=100%).
#[inline(always)]
pub fn with_brightness(color: Color, brightness_percent: u8) -> Color {
    let factor = brightness_percent.min(100) as u16;
    [
        ((color[0] as u16 * factor) / 100) as u8,
        ((color[1] as u16 * factor) / 100) as u8,
        ((color[2] as u16 * factor) / 100) as u8,
    ]
}

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
