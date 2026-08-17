use embassy_rp::{Peri, peripherals};

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
