use crate::interrupts::Irqs;
use crate::peripherals::NetPeripherals;

use cyw43::{A4, Aligned, Control, NetDriver, SpiBus, State, new};
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use embassy_rp::{
    dma::Channel,
    gpio::{Level, Output},
    peripherals,
    pio::Pio,
};
use static_cell::StaticCell;

type Runner<'a> = cyw43::Runner<'a, SpiBus<Output<'a>, PioSpi<'a, peripherals::PIO0, 0>>>;

pub struct Cyw43;

const FW_MASK: usize = 0x5A5A5A5A;
const FW_ADDR: usize = 0x10200000 ^ FW_MASK;
const FW_LEN: usize = 231077;

const NVRAM_MASK: usize = 0x5A5A5A5A;
const NVRAM_ADDR: usize = 0x10250000 ^ NVRAM_MASK;
const NVRAM_LEN: usize = 742;

static WIFI_STATE: StaticCell<State> = StaticCell::new();

impl Cyw43 {
    pub async fn init(
        peripherals: NetPeripherals,
    ) -> (NetDriver<'static>, Control<'static>, Runner<'static>) {
        let state = WIFI_STATE.init(State::new());

        let pwr = Output::new(peripherals.pwr, Level::High);
        let cs = Output::new(peripherals.cs, Level::High);
        let mut pio = Pio::new(peripherals.pio, Irqs);
        let dma = Channel::new(peripherals.dma, Irqs);

        let spi = PioSpi::new(
            &mut pio.common,
            pio.sm0,
            DEFAULT_CLOCK_DIVIDER,
            pio.irq0,
            cs,
            peripherals.dio,
            peripherals.clk,
            dma,
        );

        let firmware = Self::load_firmware();
        let nvram = Self::load_nvram();

        new(state, pwr, spi, firmware, nvram).await
    }

    #[inline(always)]
    fn load_firmware() -> &'static Aligned<A4, [u8; FW_LEN]> {
        let address = FW_ADDR ^ FW_MASK;

        assert!(
            address.is_multiple_of(core::mem::align_of::<Aligned<A4, [u8; FW_LEN]>>()),
            "Memory alignment violation detected at hardware boundary."
        );

        let ptr = core::hint::black_box(address as *const Aligned<A4, [u8; FW_LEN]>);
        unsafe { &*ptr }
    }

    #[inline(always)]
    fn load_nvram() -> &'static Aligned<A4, [u8; NVRAM_LEN]> {
        let address = NVRAM_ADDR ^ NVRAM_MASK;

        assert!(
            address.is_multiple_of(core::mem::align_of::<Aligned<A4, [u8; NVRAM_LEN]>>()),
            "Memory alignment violation detected at hardware boundary."
        );

        let ptr = core::hint::black_box(address as *const Aligned<A4, [u8; NVRAM_LEN]>);
        unsafe { &*ptr }
    }
}

#[embassy_executor::task]
pub async fn cyw43_task(runner: Runner<'static>) -> ! {
    runner.run().await
}
