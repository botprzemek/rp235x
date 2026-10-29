//! CYW43439 Wi-Fi chip initialization and PIO SPI runner task.

use core::hint;
use core::mem;

use cyw43::{A4, Aligned, Control, NetDriver, SpiBus, State};
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use embassy_rp::dma::Channel;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals;
use embassy_rp::pio::Pio;
use static_cell::StaticCell;

use crate::interrupts::Irqs;
use crate::peripherals::NetPeripherals;

type Runner<'driver> =
    cyw43::Runner<'driver, SpiBus<Output<'driver>, PioSpi<'driver, peripherals::PIO0, 0>>>;

/// Initializer for the CYW43 wireless driver, control handle, and background runner.
pub struct Cyw43Driver {
    pub net_driver: NetDriver<'static>,
    pub control: Control<'static>,
    pub runner: Runner<'static>,
}

/// CYW43 hardware initialization helper.
pub struct Cyw43;

static WIFI_STATE: StaticCell<State> = StaticCell::new();

const FIRMWARE_MASK: usize = 0x5a5a_5a5a;
const FIRMWARE_ADDR: usize = 0x1020_0000 ^ FIRMWARE_MASK;
const FIRMWARE_LEN: usize = 231_077;

const NVRAM_MASK: usize = 0x5a5a_5a5a;
const NVRAM_ADDR: usize = 0x1025_0000 ^ NVRAM_MASK;
const NVRAM_LEN: usize = 742;

impl Cyw43 {
    /// Initializes the CYW43 chip over PIO SPI using firmware and NVRAM stored in flash.
    pub async fn init(peripherals: NetPeripherals) -> Cyw43Driver {
        let state = WIFI_STATE.init(State::new());

        let power_pin = Output::new(peripherals.pwr, Level::High);
        let chip_select_pin = Output::new(peripherals.cs, Level::High);
        let mut pio = Pio::new(peripherals.pio, Irqs);
        let dma = Channel::new(peripherals.dma, Irqs);

        let spi = PioSpi::new(
            &mut pio.common,
            pio.sm0,
            DEFAULT_CLOCK_DIVIDER,
            pio.irq0,
            chip_select_pin,
            peripherals.dio,
            peripherals.clk,
            dma,
        );

        let firmware = Self::load_firmware();
        let nvram = Self::load_nvram();

        let (net_driver, control, runner) =
            cyw43::new(state, power_pin, spi, firmware, nvram).await;

        Cyw43Driver {
            net_driver,
            control,
            runner,
        }
    }

    #[inline(always)]
    fn load_firmware() -> &'static Aligned<A4, [u8; FIRMWARE_LEN]> {
        let address = FIRMWARE_ADDR ^ FIRMWARE_MASK;

        assert!(
            address.is_multiple_of(mem::align_of::<Aligned<A4, [u8; FIRMWARE_LEN]>>()),
            "Memory alignment violation detected at hardware boundary for CYW43 firmware at {address:#x}"
        );

        let pointer = hint::black_box(address as *const Aligned<A4, [u8; FIRMWARE_LEN]>);
        // SAFETY: `address` is 4-byte aligned (checked above) and points to the CYW43 firmware blob
        // flashed at `0x1020_0000` with length `FIRMWARE_LEN`.
        // Justification: Zero-copy access to memory-mapped XIP flash on RP235x.
        unsafe { &*pointer }
    }

    #[inline(always)]
    fn load_nvram() -> &'static Aligned<A4, [u8; NVRAM_LEN]> {
        let address = NVRAM_ADDR ^ NVRAM_MASK;

        assert!(
            address.is_multiple_of(mem::align_of::<Aligned<A4, [u8; NVRAM_LEN]>>()),
            "Memory alignment violation detected at hardware boundary for CYW43 NVRAM at {address:#x}"
        );

        let pointer = hint::black_box(address as *const Aligned<A4, [u8; NVRAM_LEN]>);
        // SAFETY: `address` is 4-byte aligned (checked above) and points to the CYW43 NVRAM blob
        // flashed at `0x1025_0000` with length `NVRAM_LEN`.
        // Justification: Zero-copy access to memory-mapped XIP flash on RP235x.
        unsafe { &*pointer }
    }
}

/// Background Embassy task driving the CYW43 PIO SPI bus.
#[embassy_executor::task]
pub async fn cyw43_task(runner: Runner<'static>) -> ! {
    runner.run().await
}
