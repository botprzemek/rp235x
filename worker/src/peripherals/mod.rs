mod led;
mod net;
mod trng;

use embassy_rp::{Peri, Peripherals, peripherals::CORE1};
pub use led::LedPeripherals;
pub use net::NetPeripherals;
pub use trng::TrngPeripherals;

pub fn split(
    peripherals: Peripherals,
) -> (
    TrngPeripherals,
    NetPeripherals,
    LedPeripherals,
    Peri<'static, CORE1>,
) {
    let trng = TrngPeripherals {
        trng: peripherals.TRNG,
    };

    let net = NetPeripherals {
        pio: peripherals.PIO0,
        dma: peripherals.DMA_CH0,
        pwr: peripherals.PIN_23,
        cs: peripherals.PIN_25,
        dio: peripherals.PIN_24,
        clk: peripherals.PIN_29,
    };

    let led = LedPeripherals {
        r1: peripherals.PIN_2,
        g1: peripherals.PIN_3,
        b1: peripherals.PIN_4,
        r2: peripherals.PIN_5,
        g2: peripherals.PIN_8,
        b2: peripherals.PIN_9,
        a: peripherals.PIN_10,
        b: peripherals.PIN_16,
        c: peripherals.PIN_18,
        d: peripherals.PIN_20,
        e: peripherals.PIN_22,
        clk: peripherals.PIN_11,
        lat: peripherals.PIN_12,
        oe: peripherals.PIN_13,
    };

    (trng, net, led, peripherals.CORE1)
}
