use crate::{core1::display::BitplaneBuffer, peripherals::LedPeripherals};
use embassy_rp::gpio::{Level, Output};

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
    fn clock_out_fast(&mut self, bitplane: &[u8; 64]) {
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

    pub fn render_bcm_frame_sync(&mut self, bitplane_buffer: &BitplaneBuffer) {
        const BASE_CYCLES: u32 = 50;

        for bit in 0..8 {
            let cycles = BASE_CYCLES << bit;

            for row in 0..16 {
                self.oe.set_level(Level::High);
                self.clock_out_fast(&bitplane_buffer.data[row][bit]);
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
