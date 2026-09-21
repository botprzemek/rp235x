#![no_std]

use embassy_rp::gpio::{Level, Output};
use embassy_rp::{Peri, peripherals};

pub type Glyph = [u8; 9];

pub struct LedPeripherals {
    pub r1: Peri<'static, peripherals::PIN_2>,
    pub g1: Peri<'static, peripherals::PIN_3>,
    pub b1: Peri<'static, peripherals::PIN_4>,
    pub r2: Peri<'static, peripherals::PIN_5>,
    pub g2: Peri<'static, peripherals::PIN_8>,
    pub b2: Peri<'static, peripherals::PIN_9>,

    pub a: Peri<'static, peripherals::PIN_10>,
    pub b: Peri<'static, peripherals::PIN_16>,
    pub c: Peri<'static, peripherals::PIN_18>,
    pub d: Peri<'static, peripherals::PIN_20>,
    pub e: Peri<'static, peripherals::PIN_22>,

    pub clk: Peri<'static, peripherals::PIN_11>,
    pub lat: Peri<'static, peripherals::PIN_12>,
    pub oe: Peri<'static, peripherals::PIN_13>,
}

pub struct LedOutputs {
    pub r1: Output<'static>,
    pub g1: Output<'static>,
    pub b1: Output<'static>,
    pub r2: Output<'static>,
    pub g2: Output<'static>,
    pub b2: Output<'static>,

    pub a: Output<'static>,
    pub b: Output<'static>,
    pub c: Output<'static>,
    pub d: Output<'static>,
    pub _e: Output<'static>,

    pub clk: Output<'static>,
    pub lat: Output<'static>,
    pub oe: Output<'static>,
}

impl From<LedPeripherals> for LedOutputs {
    fn from(peripherals: LedPeripherals) -> Self {
        Self {
            r1: Output::new(peripherals.r1, Level::Low),
            g1: Output::new(peripherals.g1, Level::Low),
            b1: Output::new(peripherals.b1, Level::Low),
            r2: Output::new(peripherals.r2, Level::Low),
            g2: Output::new(peripherals.g2, Level::Low),
            b2: Output::new(peripherals.b2, Level::Low),

            a: Output::new(peripherals.a, Level::Low),
            b: Output::new(peripherals.b, Level::Low),
            c: Output::new(peripherals.c, Level::Low),
            d: Output::new(peripherals.d, Level::Low),
            _e: Output::new(peripherals.e, Level::Low),

            clk: Output::new(peripherals.clk, Level::Low),
            lat: Output::new(peripherals.lat, Level::Low),
            oe: Output::new(peripherals.oe, Level::Low),
        }
    }
}

impl LedOutputs {
    #[inline(always)]
    fn set_row(&mut self, row: u8) {
        self.a.set_level(if (row & 0x01) != 0 {
            Level::High
        } else {
            Level::Low
        });
        self.b.set_level(if (row & 0x02) != 0 {
            Level::High
        } else {
            Level::Low
        });
        self.c.set_level(if (row & 0x04) != 0 {
            Level::High
        } else {
            Level::Low
        });
        self.d.set_level(if (row & 0x08) != 0 {
            Level::High
        } else {
            Level::Low
        });
        self._e.set_level(Level::Low);
    }

    #[inline(always)]
    fn clock_out_fast(&mut self, bitplane: &[u8]) {
        for &mask in bitplane.iter() {
            self.r1.set_level(if (mask & (1 << 0)) != 0 {
                Level::High
            } else {
                Level::Low
            });
            self.g1.set_level(if (mask & (1 << 1)) != 0 {
                Level::High
            } else {
                Level::Low
            });
            self.b1.set_level(if (mask & (1 << 2)) != 0 {
                Level::High
            } else {
                Level::Low
            });

            self.r2.set_level(if (mask & (1 << 3)) != 0 {
                Level::High
            } else {
                Level::Low
            });
            self.g2.set_level(if (mask & (1 << 4)) != 0 {
                Level::High
            } else {
                Level::Low
            });
            self.b2.set_level(if (mask & (1 << 5)) != 0 {
                Level::High
            } else {
                Level::Low
            });

            self.clk.set_level(Level::High);
            self.clk.set_level(Level::Low);
        }
    }

    pub fn render_bcm_frame_sync<const WIDTH: usize, const HALF_HEIGHT: usize>(
        &mut self,
        bitplanes: &BitplaneBuffer<WIDTH, HALF_HEIGHT>,
    ) {
        const BASE_CYCLES: u32 = 50;

        for bit in 0..8 {
            let cycles = BASE_CYCLES << bit;

            for row in 0..HALF_HEIGHT {
                self.oe.set_level(Level::High);
                self.clock_out_fast(bitplanes.row_bitplane_slice(row, bit));
                self.set_row(row as u8);

                self.lat.set_level(Level::High);
                cortex_m::asm::delay(10);
                self.lat.set_level(Level::Low);

                self.oe.set_level(Level::Low);
                cortex_m::asm::delay(cycles);
                self.oe.set_level(Level::High);
            }
        }
    }
}

pub type Color = [u8; 3];

/// Generic Bitplane representation taking WIDTH and HALF_HEIGHT as standalone const generics
pub struct BitplaneBuffer<const WIDTH: usize, const HALF_HEIGHT: usize> {
    pub data: [[[u8; WIDTH]; 8]; HALF_HEIGHT],
}

impl<const WIDTH: usize, const HALF_HEIGHT: usize> BitplaneBuffer<WIDTH, HALF_HEIGHT> {
    pub const fn new() -> Self {
        Self {
            data: [[[0; WIDTH]; 8]; HALF_HEIGHT],
        }
    }

    #[inline(always)]
    pub fn row_bitplane_slice(&self, row: usize, bit: usize) -> &[u8] {
        &self.data[row][bit]
    }
}

/// Generic Layout and Content Engine taking WIDTH, HEIGHT, and HALF_HEIGHT as standalone generics
pub struct DisplayBuffer<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize> {
    pub frame_buffer: [[Color; WIDTH]; HEIGHT],
    pub bitplanes: BitplaneBuffer<WIDTH, HALF_HEIGHT>,
}

impl<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize>
    DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>
{
    pub const fn new() -> Self {
        Self {
            frame_buffer: [[[0, 0, 0]; WIDTH]; HEIGHT],
            bitplanes: BitplaneBuffer::new(),
        }
    }

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

            for py in 0..9 {
                let current_y = start_y + py;
                if current_y < 0 || current_y >= HEIGHT as i32 {
                    continue;
                }

                for px in 0..7 {
                    let target_x = char_x + px;
                    if target_x >= 0 && target_x < WIDTH as i32 {
                        if get_ascii_pixel(c, px as u32, py as u32) {
                            self.set_pixel(target_x as usize, current_y as usize, color);
                        }
                    }
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

    /// Returns a reference to the inner bitplanes (used by LedOutputs)
    pub fn bitplanes(&self) -> &BitplaneBuffer<WIDTH, HALF_HEIGHT> {
        &self.bitplanes
    }

    /// High-level convenience method to draw your scoreboard layout using a color hue cycle
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

// Color conversion helpers & fonts
#[inline(always)]
pub fn hue_to_rgb(hue: u16) -> Color {
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

pub static FONT_DIGITS: [Glyph; 10] = [
    [
        0b01111100, 0b11000110, 0b11001110, 0b11011110, 0b11110110, 0b11100110, 0b11000110,
        0b11000110, 0b01111100,
    ],
    [
        0b00011000, 0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
        0b00011000, 0b01111110,
    ],
    [
        0b01111100, 0b11000110, 0b00000110, 0b00011100, 0b00110000, 0b01100000, 0b11000000,
        0b11000000, 0b11111110,
    ],
    [
        0b01111100, 0b11000110, 0b00000110, 0b00111100, 0b00111100, 0b00000110, 0b00000110,
        0b11000110, 0b01111100,
    ],
    [
        0b00001100, 0b00011100, 0b00110100, 0b01100100, 0b11000100, 0b11111110, 0b00000100,
        0b00000100, 0b00000100,
    ],
    [
        0b11111110, 0b11000000, 0b11000000, 0b11111100, 0b00000110, 0b00000110, 0b00000110,
        0b11000110, 0b01111100,
    ],
    [
        0b00111100, 0b01100000, 0b11000000, 0b11000000, 0b11111100, 0b11000110, 0b11000110,
        0b11000110, 0b01111100,
    ],
    [
        0b11111110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b01100000, 0b01100000,
        0b01100000, 0b01100000,
    ],
    [
        0b01111100, 0b11000110, 0b11000110, 0b01111100, 0b01111100, 0b11000110, 0b11000110,
        0b11000110, 0b01111100,
    ],
    [
        0b01111100, 0b11000110, 0b11000110, 0b11000110, 0b01111110, 0b00000110, 0b00000110,
        0b11000110, 0b01111100,
    ],
];

pub static FONT_UPPERCASE: [Glyph; 26] = [
    // A
    [
        0b00111000, 0b01101100, 0b11000110, 0b11000110, 0b11111110, 0b11000110, 0b11000110,
        0b11000110, 0b11000110,
    ],
    // B
    [
        0b11111100, 0b11000110, 0b11000110, 0b11111100, 0b11000110, 0b11000110, 0b11000110,
        0b11000110, 0b11111100,
    ],
    // C
    [
        0b01111100, 0b11000110, 0b11000000, 0b11000000, 0b11000000, 0b11000000, 0b11000000,
        0b11000110, 0b01111100,
    ],
    // ... (Fill in D through P as needed, or use a complete generator script)
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [
        0b01111100, 0b11000110, 0b11000110, 0b11000110, 0b11000110, 0b11000110, 0b11010110,
        0b11001110, 0b01110110,
    ],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
    [0; 9],
];

#[inline]
pub fn get_ascii_pixel(mut c: u8, pixel_x: u32, pixel_y: u32) -> bool {
    if pixel_x >= 7 || pixel_y >= 9 {
        return false;
    }

    if (b'a'..=b'z').contains(&c) {
        c -= 32;
    }

    let row_data = if (b'0'..=b'9').contains(&c) {
        FONT_DIGITS[(c - b'0') as usize][pixel_y as usize]
    } else if (b'A'..=b'Z').contains(&c) {
        FONT_UPPERCASE[(c - b'A') as usize][pixel_y as usize]
    } else if c == b':' {
        let py = pixel_y;
        let px = pixel_x;
        return (3..=6).contains(&py) && (px == 2 || px == 3);
    } else {
        0
    };

    (row_data & (1 << (7 - pixel_x))) != 0
}
