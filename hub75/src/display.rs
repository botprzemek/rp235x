use crate::bitplane::BitplaneBuffer;
use crate::color::Color;
use crate::font::get_ascii_pixel;

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

pub struct DisplayBuffer<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize> {
    pub frame_buffer: [[Color; WIDTH]; HEIGHT],
    pub bitplanes: BitplaneBuffer<WIDTH, HALF_HEIGHT>,
}

impl<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize> Default
    for DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>
{
    fn default() -> Self {
        Self {
            frame_buffer: [[[0, 0, 0]; WIDTH]; HEIGHT],
            bitplanes: BitplaneBuffer::new(),
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize>
    DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>
{
    #[inline(always)]
    pub fn clear(&mut self) {
        self.frame_buffer = [[[0, 0, 0]; WIDTH]; HEIGHT];
    }

    #[inline(always)]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x < WIDTH && y < HEIGHT {
            self.frame_buffer[y][x] = color;
        }
    }

    pub fn draw_text(&mut self, text: &str, start_x: i32, start_y: i32, color: Color) {
        for (char_idx, c) in text.bytes().enumerate() {
            let char_x = start_x + (char_idx as i32 * 8);
            if char_x + 7 < 0 || char_x >= WIDTH as i32 {
                continue;
            }

            for pixel_y in 0..9 {
                let current_y = start_y + pixel_y;
                if current_y < 0 || current_y >= HEIGHT as i32 {
                    continue;
                }

                for pixel_x in 0..7 {
                    let target_x = char_x + pixel_x;
                    if target_x < 0 || target_x >= WIDTH as i32 {
                        continue;
                    }

                    let is_pixel_set = get_ascii_pixel(c, pixel_x as usize, pixel_y as usize);
                    if !is_pixel_set {
                        continue;
                    }

                    self.set_pixel(target_x as usize, current_y as usize, color);
                }
            }
        }
    }

    pub fn prepare_bitplanes(&mut self) {
        for row in 0..HALF_HEIGHT {
            let top_row = &self.frame_buffer[row];
            let bot_row = &self.frame_buffer[row + HALF_HEIGHT];

            for col in 0..WIDTH {
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

                    self.bitplanes.data[row][bit as usize][col] = mask;
                }
            }
        }
    }

    pub fn bitplanes(&self) -> &BitplaneBuffer<WIDTH, HALF_HEIGHT> {
        &self.bitplanes
    }

    pub fn draw_animated_text(
        &mut self,
        color: Color,
        home: &str,
        guest: &str,
        quarter: &str,
        clock: &str,
    ) {
        self.clear();

        let home_x = 3;
        let guest_x = (WIDTH as i32) - 26;
        let quarter_x = 3;
        let clock_x = 22;

        self.draw_text(home, home_x, 3, color);
        self.draw_text(guest, guest_x, 3, color);
        self.draw_text(quarter, quarter_x, 20, color);
        self.draw_text(clock, clock_x, 20, color);
        self.prepare_bitplanes();
    }
}
