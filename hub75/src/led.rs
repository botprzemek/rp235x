use crate::{BitplaneBuffer, LedOutputs, OutputPin};

impl<Output: OutputPin> LedOutputs<Output> {
    #[inline(always)]
    pub fn set_row(&mut self, row: u8) {
        self.a.set_level((row & 0x01) != 0);
        self.b.set_level((row & 0x02) != 0);
        self.c.set_level((row & 0x04) != 0);
        self.d.set_level((row & 0x08) != 0);
        self._e.set_low();
    }

    #[inline(always)]
    pub fn clock_out_fast(&mut self, bitplane: &[u8]) {
        for &mask in bitplane.iter() {
            self.r1.set_level((mask & (1 << 0)) != 0);
            self.g1.set_level((mask & (1 << 1)) != 0);
            self.b1.set_level((mask & (1 << 2)) != 0);

            self.r2.set_level((mask & (1 << 3)) != 0);
            self.g2.set_level((mask & (1 << 4)) != 0);
            self.b2.set_level((mask & (1 << 5)) != 0);

            self.clk.set_high();
            self.clk.set_low();
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
                self.oe.set_high();
                self.clock_out_fast(bitplanes.row_slice(row, bit));
                self.set_row(row as u8);

                self.lat.set_high();
                Self::delay(10);
                self.lat.set_low();

                self.oe.set_low();
                Self::delay(cycles);
                self.oe.set_high();
            }
        }
    }

    fn delay(_cycles: u32) {
        #[cfg(feature = "rp235x")]
        cortex_m::asm::delay(_cycles);
    }
}
