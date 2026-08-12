use embassy_rp::Peri;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals;

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

pub const IBM_FONT_7X9: [[u8; 9]; 10] = [
    // '0'
    [
        0b01111100, 0b11000110, 0b11001110, 0b11011110, 0b11110110, 0b11100110, 0b11000110,
        0b11000110, 0b01111100,
    ],
    // '1'
    [
        0b00011000, 0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
        0b01111110, 0b00000000,
    ],
    // '2'
    [
        0b01111100, 0b11000110, 0b00000110, 0b00011100, 0b00110000, 0b01100000, 0b11000000,
        0b11111110, 0b00000000,
    ],
    // '3'
    [
        0b01111100, 0b11000110, 0b00000110, 0b00111100, 0b00000110, 0b00000110, 0b11000110,
        0b01111100, 0b00000000,
    ],
    // '4'
    [
        0b00001100, 0b00011100, 0b00110100, 0b01100100, 0b11000100, 0b11111110, 0b00000100,
        0b00000100, 0b00000000,
    ],
    // '5'
    [
        0b11111110, 0b11000000, 0b11000000, 0b11111100, 0b00000110, 0b00000110, 0b11000110,
        0b01111100, 0b00000000,
    ],
    // '6'
    [
        0b00111100, 0b01100000, 0b11000000, 0b11111100, 0b11000110, 0b11000110, 0b11000110,
        0b01111100, 0b00000000,
    ],
    // '7'
    [
        0b11111110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b01100000, 0b01100000,
        0b01100000, 0b00000000,
    ],
    // '8'
    [
        0b01111100, 0b11000110, 0b11000110, 0b01111100, 0b11000110, 0b11000110, 0b11000110,
        0b01111100, 0b00000000,
    ],
    // '9'
    [
        0b01111100, 0b11000110, 0b11000110, 0b11000110, 0b01111110, 0b00000110, 0b01100110,
        0b00111100, 0b00000000,
    ],
];

static CHARS: [[u8; 5]; 11] = [
    // 0
    [0b01111100, 0b10001010, 0b10010010, 0b10100010, 0b01111100],
    // 1
    [0b00000010, 0b01000010, 0b11111110, 0b00000010, 0b00000010],
    // 2
    [0b01000110, 0b10001010, 0b10010010, 0b10010010, 0b01100010],
    // 3
    [0b01000100, 0b10000010, 0b10010010, 0b10010010, 0b01101100],
    // 4
    [0b00011000, 0b00101000, 0b01001000, 0b10001000, 0b11111110],
    // 5
    [0b11100100, 0b10100010, 0b10100010, 0b10100010, 0b10011100],
    // 6
    [0b00111100, 0b01010010, 0b10010010, 0b10010010, 0b00001100],
    // 7
    [0b11000000, 0b10000000, 0b10001110, 0b10010000, 0b11100000],
    // 8
    [0b01101100, 0b10010010, 0b10010010, 0b10010010, 0b01101100],
    // 9
    [0b01100000, 0b10010010, 0b10010010, 0b10010100, 0b01111000],
    // :
    [0b00000000, 0b00000000, 0b01101100, 0b00000000, 0b00000000],
];

pub fn get_char_pixel(char: usize, x: u32, y: u32) -> bool {
    if x >= 5 || y >= 8 || char > 10 {
        return false;
    }

    let column_data = CHARS[char][x as usize];

    (column_data & (0x80 >> y)) != 0
}

#[inline]
pub fn get_ibm_pixel(digit: u8, pixel_x: u32, pixel_y: u32) -> bool {
    if digit > 9 || pixel_x >= 7 || pixel_y >= 9 {
        return false;
    }

    let row_data = IBM_FONT_7X9[digit as usize][pixel_y as usize];

    (row_data & (1 << (7 - pixel_x))) != 0
}
