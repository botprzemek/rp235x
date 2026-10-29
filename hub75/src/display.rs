//! Framebuffer, text drawing, and gamma-corrected bitplane preparation for HUB75 panels.

use crate::bitplane::BitplaneBuffer;
use crate::color::Color;
use crate::font;

/// 8-bit gamma correction lookup table for LED perceptual linearity.
pub static GAMMA_TABLE: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7,
    8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18,
    18, 19, 19, 20, 20, 21, 21, 22, 23, 23, 24, 24, 25, 26, 26, 27, 28, 28, 29, 30, 30, 31, 32, 32,
    33, 34, 35, 35, 36, 37, 38, 38, 39, 40, 41, 42, 42, 43, 44, 45, 46, 47, 47, 48, 49, 50, 51, 52,
    53, 54, 55, 56, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 73, 74, 75, 76,
    77, 78, 79, 81, 82, 83, 84, 85, 87, 88, 89, 90, 91, 93, 94, 95, 97, 98, 99, 101, 102, 103, 105,
    106, 107, 109, 110, 112, 113, 114, 116, 117, 119, 120, 122, 123, 125, 126, 128, 130, 131, 133,
    134, 136, 138, 139, 141, 142, 144, 146, 148, 149, 151, 153, 155, 156, 158, 160, 162, 164, 166,
    167, 169, 171, 173, 175, 177, 179, 181, 183, 185, 187, 189, 191, 193, 195, 197, 199, 201, 203,
    205, 207, 210, 212, 214, 216, 218, 221, 223, 225, 228, 230, 235, 242, 255,
];

/// RGB framebuffer paired with a [`BitplaneBuffer`] and global brightness setting.
#[non_exhaustive]
pub struct DisplayBuffer<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize> {
    pub frame_buffer: [[Color; WIDTH]; HEIGHT],
    pub bitplanes: BitplaneBuffer<WIDTH, HALF_HEIGHT>,
    pub brightness_percent: u8,
}

impl<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize> Default
    for DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize>
    DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>
{
    /// Creates a new cleared [`DisplayBuffer`] at 100% brightness.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            frame_buffer: [[[0, 0, 0]; WIDTH]; HEIGHT],
            bitplanes: BitplaneBuffer::new(),
            brightness_percent: 100,
        }
    }

    /// Sets the display brightness percentage (`0..=100`).
    #[inline(always)]
    pub fn set_brightness(&mut self, percent: u8) {
        self.brightness_percent = percent.min(100);
    }

    /// Returns the display brightness percentage (`0..=100`).
    #[must_use]
    #[inline(always)]
    pub fn brightness(&self) -> u8 {
        self.brightness_percent
    }

    /// Clears all pixels in the framebuffer to black.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.frame_buffer = [[[0, 0, 0]; WIDTH]; HEIGHT];
    }

    /// Sets the pixel at `(x, y)` to `color` if within bounds.
    #[inline(always)]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x < WIDTH && y < HEIGHT {
            self.frame_buffer[y][x] = color;
        }
    }

    /// Renders an ASCII `text` string starting at `(start_x, start_y)` using `color`.
    pub fn draw_text(&mut self, text: &str, start_x: i32, start_y: i32, color: Color) {
        for (character_index, character) in text.bytes().enumerate() {
            let character_x = start_x + (character_index as i32 * 8);
            if character_x + 7 < 0 || character_x >= WIDTH as i32 {
                continue;
            }

            for pixel_y in 0..9 {
                let current_y = start_y + pixel_y;
                if current_y < 0 || current_y >= HEIGHT as i32 {
                    continue;
                }

                for pixel_x in 0..7 {
                    let target_x = character_x + pixel_x;
                    if target_x < 0 || target_x >= WIDTH as i32 {
                        continue;
                    }

                    let is_pixel_set =
                        font::is_ascii_pixel_set(character, pixel_x as usize, pixel_y as usize);
                    if !is_pixel_set {
                        continue;
                    }

                    self.set_pixel(target_x as usize, current_y as usize, color);
                }
            }
        }
    }

    /// Converts the RGB `frame_buffer` into BCM bitplanes with brightness scaling and gamma correction.
    pub fn prepare_bitplanes(&mut self) {
        let brightness = u16::from(self.brightness_percent);
        for row in 0..HALF_HEIGHT {
            let top_row = &self.frame_buffer[row];
            let bottom_row = &self.frame_buffer[row + HALF_HEIGHT];

            for column in 0..WIDTH {
                let red_1_in = if brightness == 100 {
                    top_row[column][0]
                } else {
                    ((u16::from(top_row[column][0]) * brightness) / 100) as u8
                };
                let green_1_in = if brightness == 100 {
                    top_row[column][1]
                } else {
                    ((u16::from(top_row[column][1]) * brightness) / 100) as u8
                };
                let blue_1_in = if brightness == 100 {
                    top_row[column][2]
                } else {
                    ((u16::from(top_row[column][2]) * brightness) / 100) as u8
                };

                let red_2_in = if brightness == 100 {
                    bottom_row[column][0]
                } else {
                    ((u16::from(bottom_row[column][0]) * brightness) / 100) as u8
                };
                let green_2_in = if brightness == 100 {
                    bottom_row[column][1]
                } else {
                    ((u16::from(bottom_row[column][1]) * brightness) / 100) as u8
                };
                let blue_2_in = if brightness == 100 {
                    bottom_row[column][2]
                } else {
                    ((u16::from(bottom_row[column][2]) * brightness) / 100) as u8
                };

                let red_1 = GAMMA_TABLE[red_1_in as usize];
                let green_1 = GAMMA_TABLE[green_1_in as usize];
                let blue_1 = GAMMA_TABLE[blue_1_in as usize];

                let red_2 = GAMMA_TABLE[red_2_in as usize];
                let green_2 = GAMMA_TABLE[green_2_in as usize];
                let blue_2 = GAMMA_TABLE[blue_2_in as usize];

                for bit in 0..8 {
                    let mask = ((red_1 >> bit) & 1)
                        | (((green_1 >> bit) & 1) << 1)
                        | (((blue_1 >> bit) & 1) << 2)
                        | (((red_2 >> bit) & 1) << 3)
                        | (((green_2 >> bit) & 1) << 4)
                        | (((blue_2 >> bit) & 1) << 5);

                    self.bitplanes.data[row][bit as usize][column] = mask;
                }
            }
        }
    }

    /// Returns a reference to the prepared [`BitplaneBuffer`].
    #[must_use]
    pub fn bitplanes(&self) -> &BitplaneBuffer<WIDTH, HALF_HEIGHT> {
        &self.bitplanes
    }
}
