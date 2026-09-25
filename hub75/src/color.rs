pub type Color = [u8; 3];

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
