use crate::LedOutputs;
use crate::bitplane::BitplaneBuffer;
use crate::output::{OutputPin, PinLevel};

impl<P: OutputPin> LedOutputs<P> {
    #[inline(always)]
    pub fn set_row(&mut self, row: u8) {
        self.a.set_level(PinLevel::from((row & 0x01) != 0));
        self.b.set_level(PinLevel::from((row & 0x02) != 0));
        self.c.set_level(PinLevel::from((row & 0x04) != 0));
        self.d.set_level(PinLevel::from((row & 0x08) != 0));
        self.e.set_low();
    }

    #[inline(always)]
    pub fn clock_out_fast(&mut self, bitplane: &[u8]) {
        for &mask in bitplane {
            self.r1.set_level(PinLevel::from((mask & (1 << 0)) != 0));
            self.g1.set_level(PinLevel::from((mask & (1 << 1)) != 0));
            self.b1.set_level(PinLevel::from((mask & (1 << 2)) != 0));

            self.r2.set_level(PinLevel::from((mask & (1 << 3)) != 0));
            self.g2.set_level(PinLevel::from((mask & (1 << 4)) != 0));
            self.b2.set_level(PinLevel::from((mask & (1 << 5)) != 0));

            self.clk.set_high();
            self.clk.set_low();
        }
    }

    #[inline(always)]
    pub fn clock_out_sio_fast(&mut self, bitplane: &[u8]) {
        const CLEAR_MASK: u32 =
            (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (1 << 8) | (1 << 9) | (1 << 11);
        const CLK_HIGH_BIT: u32 = 1 << 11;

        let sio_out_set = 0xd000_0018 as *mut u32;
        let sio_out_clr = 0xd000_0020 as *mut u32;

        for &mask in bitplane {
            let r1 = ((mask as u32) & 1) << 2;
            let g1 = (((mask as u32) >> 1) & 1) << 3;
            let b1 = (((mask as u32) >> 2) & 1) << 4;
            let r2 = (((mask as u32) >> 3) & 1) << 5;
            let g2 = (((mask as u32) >> 4) & 1) << 8;
            let b2 = (((mask as u32) >> 5) & 1) << 9;

            let set_mask = r1 | g1 | b1 | r2 | g2 | b2;
            let clr_mask = (!set_mask) & (CLEAR_MASK & !CLK_HIGH_BIT);

            unsafe {
                sio_out_clr.write_volatile(clr_mask);
                sio_out_set.write_volatile(set_mask);

                Self::delay(2);

                sio_out_set.write_volatile(CLK_HIGH_BIT);
                Self::delay(2);

                sio_out_clr.write_volatile(CLK_HIGH_BIT);
                Self::delay(1);
            }
        }

        unsafe {
            sio_out_clr.write_volatile(CLK_HIGH_BIT);
        }
    }

    pub fn render_bcm_frame_sio_fast<const WIDTH: usize, const HALF_HEIGHT: usize>(
        &mut self,
        bitplanes: &BitplaneBuffer<WIDTH, HALF_HEIGHT>,
        brightness_percent: u8,
    ) {
        const BASE_CYCLES: u32 = 20;
        let brightness_factor = u32::from(brightness_percent.min(100));

        for bit in 0..8 {
            let full_cycles = BASE_CYCLES << bit;
            let on_cycles = (full_cycles * brightness_factor) / 100;
            let off_cycles = full_cycles.saturating_sub(on_cycles);

            for row in 0..HALF_HEIGHT {
                self.oe.set_high();

                self.clock_out_sio_fast(bitplanes.row_slice(row, bit));
                self.set_row(row as u8);

                Self::delay(4);
                self.lat.set_high();
                Self::delay(8);
                self.lat.set_low();
                Self::delay(4);

                if on_cycles > 0 {
                    self.oe.set_low();
                    Self::delay(on_cycles);
                    self.oe.set_high();
                }

                if off_cycles > 0 {
                    Self::delay(off_cycles);
                }
            }
        }

        self.oe.set_high();
        self.lat.set_low();
    }

    pub fn render_bcm_frame_sync<const WIDTH: usize, const HALF_HEIGHT: usize>(
        &mut self,
        bitplanes: &BitplaneBuffer<WIDTH, HALF_HEIGHT>,
    ) {
        self.render_bcm_frame_sync_with_brightness(bitplanes, 100);
    }

    pub fn render_bcm_frame_sync_with_brightness<const WIDTH: usize, const HALF_HEIGHT: usize>(
        &mut self,
        bitplanes: &BitplaneBuffer<WIDTH, HALF_HEIGHT>,
        brightness_percent: u8,
    ) {
        const BASE_CYCLES: u32 = 20;
        let brightness_factor = u32::from(brightness_percent.min(100));

        for bit in 0..8 {
            let full_cycles = BASE_CYCLES << bit;
            let on_cycles = (full_cycles * brightness_factor) / 100;
            let off_cycles = full_cycles.saturating_sub(on_cycles);

            for row in 0..HALF_HEIGHT {
                self.oe.set_high();

                self.clock_out_fast(bitplanes.row_slice(row, bit));
                self.set_row(row as u8);

                Self::delay(4);
                self.lat.set_high();
                Self::delay(8);
                self.lat.set_low();
                Self::delay(4);

                if on_cycles > 0 {
                    self.oe.set_low();
                    Self::delay(on_cycles);
                    self.oe.set_high();
                }

                if off_cycles > 0 {
                    Self::delay(off_cycles);
                }
            }
        }

        self.oe.set_high();
        self.lat.set_low();
    }

    fn delay(_cycles: u32) {
        #[cfg(feature = "rp235x")]
        cortex_m::asm::delay(_cycles);
    }
}
