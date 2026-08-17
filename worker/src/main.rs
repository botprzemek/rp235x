#![no_std]
#![no_main]

mod core0;
mod core1;
mod led;
mod net;
mod shared;

use embassy_rp::multicore::{Stack, spawn_core1};
use {defmt_rtt as _, panic_probe as _};

use core0::Core0;
use core1::Core1;

pub static mut CORE1_STACK: Stack<16384> = Stack::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let peripherals = embassy_rp::init(Default::default());

    let trng_peripherals = net::TrngPeripherals {
        trng: peripherals.TRNG,
    };
    let net_peripherals = net::NetPeripherals {
        pio: peripherals.PIO0,
        dma: peripherals.DMA_CH0,
        pwr: peripherals.PIN_23,
        cs: peripherals.PIN_25,
        dio: peripherals.PIN_24,
        clk: peripherals.PIN_29,
    };

    let led_peripherals = led::LedPeripherals {
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

    spawn_core1(
        peripherals.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || Core1::entry(led_peripherals),
    );

    Core0::entry(net_peripherals, trng_peripherals)
}
