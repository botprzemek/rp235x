use crate::interrupts::Irqs;
use cyw43::{Control, JoinOptions, NetDriver, SpiBus, State};
use cyw43_pio::PioSpi;
use embassy_rp::{
    dma::Channel,
    gpio::{Level, Output},
    peripherals,
    pio::Pio,
};
use static_cell::StaticCell;

use crate::peripherals::NetPeripherals;

static FW_ADDR: usize = 0x10200000;
static FW_LEN: usize = 231077;

static NVRAM_ADDR: usize = 0x10250000;
static NVRAM_LEN: usize = 742;

static CLM_ADDR: usize = 0x10240000;
static CLM_LEN: usize = 984;

static WIFI_STATE: StaticCell<State> = StaticCell::new();

type Runner<'a> = cyw43::Runner<'a, SpiBus<Output<'a>, PioSpi<'a, peripherals::PIO0, 0>>>;

struct Cyw43;

impl Cyw43 {
    pub async fn init(
        peripherals: NetPeripherals,
    ) -> (NetDriver<'static>, Control<'static>, Runner<'static>) {
        let state = WIFI_STATE.init(State::new());
        let pwr = Output::new(peripherals.pwr, Level::High);
        let cs = Output::new(peripherals.cs, Level::High);
        let mut pio = Pio::new(peripherals.pio, Irqs);
        let dma_channel = Channel::new(peripherals.dma, Irqs);
        let spi = PioSpi::new(
            &mut pio.common,
            pio.sm0,
            cyw43_pio::DEFAULT_CLOCK_DIVIDER,
            pio.irq0,
            cs,
            peripherals.dio,
            peripherals.clk,
            dma_channel,
        );

        let fw_ptr = core::hint::black_box(FW_ADDR as *const u8);
        let firmware_slice: &[u8] = unsafe { core::slice::from_raw_parts(fw_ptr, FW_LEN) };
        let firmware_aligned: &cyw43::Aligned<cyw43::A4, [u8]> =
            unsafe { core::mem::transmute(firmware_slice) };

        let nvram_ptr = core::hint::black_box(NVRAM_ADDR as *const u8);
        let nvram_slice: &[u8] = unsafe { core::slice::from_raw_parts(nvram_ptr, NVRAM_LEN) };
        let nvram_aligned: &cyw43::Aligned<cyw43::A4, [u8]> =
            unsafe { core::mem::transmute(nvram_slice) };

        cyw43::new(state, pwr, spi, firmware_aligned, nvram_aligned).await
    }
}

#[embassy_executor::task]
pub async fn task(runner: Runner<'static>) -> ! {
    runner.run().await
}
