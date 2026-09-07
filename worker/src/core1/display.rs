pub type Color = [u8; 3];
pub type FrameBuffer = [[Color; 64]; 32];

#[derive(Clone, Copy)]
pub struct BitplaneBuffer {
    pub data: [[[u8; 64]; 8]; 16],
}

impl BitplaneBuffer {
    pub const EMPTY: Self = Self {
        data: [[[0; 64]; 8]; 16],
    };
}

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

pub static IBM_FONT_7X9: [[u8; 9]; 10] = [
    // '0'
    [
        0b01111100, 0b11000110, 0b11001110, 0b11011110, 0b11110110, 0b11100110, 0b11000110,
        0b11000110, 0b01111100,
    ],
    // '1'
    [
        0b00011000, 0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
        0b00011000, 0b01111110,
    ],
    // '2'
    [
        0b01111100, 0b11000110, 0b00000110, 0b00011100, 0b00110000, 0b01100000, 0b11000000,
        0b11000000, 0b11111110,
    ],
    // '3'
    [
        0b01111100, 0b11000110, 0b00000110, 0b00111100, 0b00111100, 0b00000110, 0b00000110,
        0b11000110, 0b01111100,
    ],
    // '4'
    [
        0b00001100, 0b00011100, 0b00110100, 0b01100100, 0b11000100, 0b11111110, 0b00000100,
        0b00000100, 0b00000100,
    ],
    // '5'
    [
        0b11111110, 0b11000000, 0b11000000, 0b11111100, 0b00000110, 0b00000110, 0b00000110,
        0b11000110, 0b01111100,
    ],
    // '6'
    [
        0b00111100, 0b01100000, 0b11000000, 0b11000000, 0b11111100, 0b11000110, 0b11000110,
        0b11000110, 0b01111100,
    ],
    // '7'
    [
        0b11111110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b01100000, 0b01100000,
        0b01100000, 0b01100000,
    ],
    // '8'
    [
        0b01111100, 0b11000110, 0b11000110, 0b01111100, 0b01111100, 0b11000110, 0b11000110,
        0b11000110, 0b01111100,
    ],
    // '9'
    [
        0b01111100, 0b11000110, 0b11000110, 0b11000110, 0b01111110, 0b00000110, 0b00000110,
        0b11000110, 0b01111100,
    ],
];

#[inline]
pub fn get_ibm_pixel(digit: u8, pixel_x: u32, pixel_y: u32) -> bool {
    if digit > 9 || pixel_x >= 7 || pixel_y >= 9 {
        return false;
    }

    let row_data = IBM_FONT_7X9[digit as usize][pixel_y as usize];

    (row_data & (1 << (7 - pixel_x))) != 0
}

pub struct Display {
    frame_buffer: FrameBuffer,
    bitplane_buffer: BitplaneBuffer,
}

impl Display {
    pub const fn new() -> Self {
        Self {
            frame_buffer: [[[0, 0, 0]; 64]; 32],
            bitplane_buffer: BitplaneBuffer::EMPTY,
        }
    }

    #[inline(always)]
    fn hue_to_rgb(&self, hue: u16) -> Color {
        let h = (hue % 360) as u32;
        let x = ((1..=255).contains(&((h % 60) * 255 / 60)) as u32 * ((h % 60) * 255 / 60)) as u8;

        match h / 60 {
            0 => [255, x, 0],
            1 => [255 - x, 255, 0],
            2 => [0, 255, x],
            3 => [0, 255 - x, 255],
            4 => [x, 0, 255],
            _ => [255, 0, 255 - x],
        }
    }

    pub fn draw_animated_text(
        &mut self,
        home_str: &str,
        guest_str: &str,
        clock_str: &str,
        phase_hue: u16,
    ) {
        self.frame_buffer = [[[0, 0, 0]; 64]; 32];

        let block_width: i32 = 3 * 8;
        let gap_px: i32 = 17;
        let total_width = (block_width * 2) + gap_px;

        let r1_start_x: i32 = (64 - total_width) / 2;
        let r1_start_y: u32 = 0;

        let clock_width: i32 = clock_str.len() as i32 * 8;
        let r2_start_x: i32 = (64 - clock_width) / 2;
        let r2_start_y: u32 = 23;

        for y in 0..32 {
            for x in 0..64 {
                let hue = (phase_hue + ((x as u32 * 67) / 64) as u16) % 360;
                let color_at_x = self.hue_to_rgb(hue);
                if y >= r1_start_y && y < r1_start_y + 9 && x >= r1_start_x {
                    let local_x = x - r1_start_x;

                    if local_x < block_width {
                        let char_idx = (local_x / 8) as usize;
                        let pixel_x = (local_x % 8) as u32;
                        let pixel_y = y - r1_start_y;

                        if char_idx < home_str.len() && pixel_x < 7 {
                            let c = home_str.as_bytes()[char_idx];
                            if c.is_ascii_digit() && get_ibm_pixel(c - b'0', pixel_x, pixel_y) {
                                self.frame_buffer[y as usize][x as usize] = color_at_x;
                            }
                        }
                    } else if local_x >= block_width + gap_px {
                        let right_local_x = local_x - (block_width + gap_px);
                        let char_idx = (right_local_x / 8) as usize;
                        let pixel_x = (right_local_x % 8) as u32;
                        let pixel_y = y - r1_start_y;

                        if char_idx < guest_str.len() && pixel_x < 7 {
                            let c = guest_str.as_bytes()[char_idx];
                            if c.is_ascii_digit() && get_ibm_pixel(c - b'0', pixel_x, pixel_y) {
                                self.frame_buffer[y as usize][x as usize] = color_at_x;
                            }
                        }
                    }
                }
                if y >= r2_start_y && y < r2_start_y + 9 && x >= r2_start_x {
                    let local_x = x - r2_start_x;
                    let char_idx = (local_x / 8) as usize;
                    let pixel_x = (local_x % 8) as u32;
                    let pixel_y = y - r2_start_y;

                    if char_idx < clock_str.len() && pixel_x < 7 {
                        let c = clock_str.as_bytes()[char_idx];
                        let pixel_on = if c == b':' {
                            (3..=6).contains(&pixel_y) && (pixel_x == 2 || pixel_x == 3)
                        } else if c.is_ascii_digit() {
                            get_ibm_pixel(c - b'0', pixel_x, pixel_y)
                        } else {
                            false
                        };

                        if pixel_on {
                            self.frame_buffer[y as usize][x as usize] = color_at_x;
                        }
                    }
                }
            }
        }

        self.prepare_bitplanes();
    }

    fn prepare_bitplanes(&mut self) {
        for row in 0..16 {
            let top_row = &self.frame_buffer[row];
            let bot_row = &self.frame_buffer[row + 16];

            for col in 0..64 {
                let r1 = GAMMA_TABLE[top_row[col][0] as usize];
                let g1 = GAMMA_TABLE[top_row[col][1] as usize];
                let b1 = GAMMA_TABLE[top_row[col][2] as usize];

                let r2 = GAMMA_TABLE[bot_row[col][0] as usize];
                let g2 = GAMMA_TABLE[bot_row[col][1] as usize];
                let b2 = GAMMA_TABLE[bot_row[col][2] as usize];

                for bit in 0..8 {
                    let mask = ((r1 >> bit) & 1)
                        | (((g1 >> bit) & 1) << 1)
                        | (((b1 >> bit) & 1) << 2)
                        | (((r2 >> bit) & 1) << 3)
                        | (((g2 >> bit) & 1) << 4)
                        | (((b2 >> bit) & 1) << 5);

                    self.bitplane_buffer.data[row][bit as usize][col] = mask;
                }
            }
        }
    }

    pub fn bitplanes(&self) -> &BitplaneBuffer {
        &self.bitplane_buffer
    }
}
