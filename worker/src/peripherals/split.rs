//! Partitioning of RP235x hardware peripherals into functional groups.

use embassy_rp::peripherals::CORE1;
use embassy_rp::{Peri, Peripherals};
use hub75::LedPeripherals;

use crate::peripherals::{NetPeripherals, TrngPeripherals};

/// Partitioned hardware peripherals for Core 0 and Core 1.
pub struct BoardPeripherals {
    pub trng: TrngPeripherals,
    pub net: NetPeripherals,
    pub led: LedPeripherals,
    pub core1: Peri<'static, CORE1>,
}

/// Splits the raw `embassy_rp::Peripherals` into [`BoardPeripherals`].
#[must_use]
pub fn split(peripherals: Peripherals) -> BoardPeripherals {
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

    let led = LedPeripherals::new(
        peripherals.PIN_2,
        peripherals.PIN_3,
        peripherals.PIN_4,
        peripherals.PIN_5,
        peripherals.PIN_8,
        peripherals.PIN_9,
        peripherals.PIN_10,
        peripherals.PIN_16,
        peripherals.PIN_18,
        peripherals.PIN_20,
        peripherals.PIN_22,
        peripherals.PIN_11,
        peripherals.PIN_12,
        peripherals.PIN_13,
    );

    BoardPeripherals {
        trng,
        net,
        led,
        core1: peripherals.CORE1,
    }
}

