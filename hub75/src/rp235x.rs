//! RP235x GPIO peripheral bindings for [`LedOutputs`].

use embassy_rp::gpio::{Level, Output};
use embassy_rp::{Peri, peripherals};

use crate::{LedOutputs, OutputPin};

/// RP235x GPIO pins assigned to the HUB75 panel interface.
#[non_exhaustive]
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

impl LedPeripherals {
    /// Bundles the RP235x GPIO pins into [`LedPeripherals`].
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        r1: Peri<'static, peripherals::PIN_2>,
        g1: Peri<'static, peripherals::PIN_3>,
        b1: Peri<'static, peripherals::PIN_4>,
        r2: Peri<'static, peripherals::PIN_5>,
        g2: Peri<'static, peripherals::PIN_8>,
        b2: Peri<'static, peripherals::PIN_9>,
        a: Peri<'static, peripherals::PIN_10>,
        b: Peri<'static, peripherals::PIN_16>,
        c: Peri<'static, peripherals::PIN_18>,
        d: Peri<'static, peripherals::PIN_20>,
        e: Peri<'static, peripherals::PIN_22>,
        clk: Peri<'static, peripherals::PIN_11>,
        lat: Peri<'static, peripherals::PIN_12>,
        oe: Peri<'static, peripherals::PIN_13>,
    ) -> Self {
        Self {
            r1,
            g1,
            b1,
            r2,
            g2,
            b2,
            a,
            b,
            c,
            d,
            e,
            clk,
            lat,
            oe,
        }
    }
}

impl OutputPin for Output<'_> {
    fn set_high(&mut self) {
        Self::set_high(self);
    }

    fn set_low(&mut self) {
        Self::set_low(self);
    }
}

impl LedOutputs<Output<'static>> {
    /// Initializes [`LedOutputs`] from RP235x [`LedPeripherals`].
    #[must_use]
    pub fn from_peripherals(peripherals: LedPeripherals) -> Self {
        Self::from(peripherals)
    }
}

impl From<LedPeripherals> for LedOutputs<Output<'static>> {
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
            e: Output::new(peripherals.e, Level::Low),

            clk: Output::new(peripherals.clk, Level::Low),
            lat: Output::new(peripherals.lat, Level::Low),
            oe: Output::new(peripherals.oe, Level::Low),
        }
    }
}
