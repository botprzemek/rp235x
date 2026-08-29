#![no_std]
#![no_main]

mod core0;
mod core1;
mod interrupts;
mod peripherals;
mod state;

use core0::Core0;
use core1::Core1;
use embassy_rp::multicore::{Stack, spawn_core1};
use {defmt_rtt as _, panic_probe as _};

pub static mut CORE1_STACK: Stack<16384> = Stack::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let peripherals = embassy_rp::init(Default::default());
    let (trng, net, led, core1) = peripherals::split(peripherals);

    spawn_core1(
        core1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || Core1::entry(led),
    );

    Core0::entry(net, trng)
}
