use crate::peripherals::LedPeripherals;
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
