//! Dual-core RP235x firmware entry point for the wireless HUB75 scoreboard worker.

#![no_std]
#![no_main]

use core::ptr;

use worker::core0::Core0;
use worker::core1::Core1;
use worker::peripherals;

use defmt_rtt as _;
use embassy_rp::multicore::{self, Stack};
use panic_probe as _;

/// Execution stack allocated for Core 1.
pub static mut CORE1_STACK: Stack<16384> = Stack::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let raw_peripherals = embassy_rp::init(Default::default());
    defmt::info!("=== RP235x Worker starting ===");
    let board_peripherals = peripherals::split(raw_peripherals);

    // SAFETY: `CORE1_STACK` is only passed once here to `spawn_core1` before Core 1 starts executing,
    // and is never accessed again from Core 0.
    // Justification: Required by `embassy_rp::multicore::spawn_core1` to provide a static stack for Core 1.
    let core1_stack = unsafe { &mut *ptr::addr_of_mut!(CORE1_STACK) };

    multicore::spawn_core1(board_peripherals.core1, core1_stack, move || {
        Core1::entry(board_peripherals.led)
    });

    Core0::entry(
        board_peripherals.net,
        board_peripherals.trng,
        board_peripherals.watchdog,
        board_peripherals.flash,
    )
}
