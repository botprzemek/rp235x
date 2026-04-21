// #![no_std]
// #![no_main]
// #![allow(async_fn_in_trait)]

// mod irqs;
// mod net;
// mod usb;

// use defmt::unwrap;
// use embassy_executor::Executor;
// use embassy_rp::multicore::{Stack, spawn_core1};
// use static_cell::StaticCell;
// use {defmt_rtt as _, panic_probe as _};

// static mut CORE1_STACK: Stack<8192> = Stack::new();
// static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
// static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

// #[cortex_m_rt::entry]
// fn main() -> ! {
//     let peripherals = embassy_rp::init(Default::default());
//     let usb_peripherals = usb::UsbPeripherals {
//         usb: peripherals.USB,
//     };
//     let net_peripherals = net::NetPeripherals {
//         trng: peripherals.TRNG,
//         pio: peripherals.PIO0,
//         dma: peripherals.DMA_CH0,
//         pwr: peripherals.PIN_23,
//         cs: peripherals.PIN_25,
//         dio: peripherals.PIN_24,
//         clk: peripherals.PIN_29,
//     };

//     spawn_core1(
//         peripherals.CORE1,
//         unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
//         move || {
//             let executor1 = EXECUTOR1.init(Executor::new());
//             executor1.run(|spawner| spawner.spawn(unwrap!(core1_task(spawner, net_peripherals))));
//         },
//     );

//     let executor0 = EXECUTOR0.init(Executor::new());
//     executor0.run(|spawner| spawner.spawn(unwrap!(core0_task(spawner, usb_peripherals))));
// }

// #[embassy_executor::task]
// async fn core0_task(spawner: embassy_executor::Spawner, usb_periperherals: usb::UsbPeripherals) {
//     usb::Usb::init(spawner, usb_periperherals);
// }

// #[embassy_executor::task]
// async fn core1_task(spawner: embassy_executor::Spawner, net_peripherals: net::NetPeripherals) {
//     embassy_time::Timer::after_millis(7000).await;
//     net::Net::init(spawner, net_peripherals).await;
// }

// // #![no_std]
// // #![no_main]

// // use cyw43::aligned_bytes;
// // use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
// // use defmt::*;
// // use embassy_executor::Executor;
// // use embassy_executor::Spawner;
// // use embassy_rp::gpio::{Level, Output};
// // use embassy_rp::multicore::{Stack, spawn_core1};
// // use embassy_rp::peripherals;
// // use embassy_rp::peripherals::{DMA_CH0, PIO0};
// // use embassy_rp::pio::{InterruptHandler, Pio};
// // use embassy_rp::{bind_interrupts, dma};
// // use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
// // use embassy_sync::channel::Channel;
// // use embassy_time::Timer;
// // use static_cell::StaticCell;
// // use {defmt_rtt as _, panic_probe as _};
// // use {defmt_rtt as _, panic_probe as _};

// // static mut CORE1_STACK: Stack<8192> = Stack::new();
// // static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
// // static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
// // static CHANNEL: Channel<CriticalSectionRawMutex, LedState, 1> = Channel::new();

// // static FW: &cyw43::Aligned<cyw43::A4, [u8]> = aligned_bytes!("bin/cyw43-firmware/43439A0.bin");
// // static CLM: &cyw43::Aligned<cyw43::A4, [u8]> = aligned_bytes!("bin/cyw43-firmware/43439A0_clm.bin");
// // static NVRAM: &cyw43::Aligned<cyw43::A4, [u8]> =
// //     aligned_bytes!("bin/cyw43-firmware/nvram_rp2040.bin");

// // enum LedState {
// //     On,
// //     Off,
// // }

// // pub struct NetPeripherals {
// //     pub trng: embassy_rp::Peri<'static, peripherals::TRNG>,
// //     pub pio: embassy_rp::Peri<'static, peripherals::PIO0>,
// //     pub dma: embassy_rp::Peri<'static, peripherals::DMA_CH0>,
// //     pub pwr: embassy_rp::Peri<'static, peripherals::PIN_23>,
// //     pub cs: embassy_rp::Peri<'static, peripherals::PIN_25>,
// //     pub dio: embassy_rp::Peri<'static, peripherals::PIN_24>,
// //     pub clk: embassy_rp::Peri<'static, peripherals::PIN_29>,
// // }

// // bind_interrupts!(struct Irqs1 {
// //     PIO0_IRQ_0 => InterruptHandler<PIO0>;
// //     DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
// // });

// // #[embassy_executor::task]
// // async fn wifi_task(
// //     runner: cyw43::Runner<
// //         'static,
// //         cyw43::SpiBus<
// //             embassy_rp::gpio::Output<'static>,
// //             cyw43_pio::PioSpi<'static, peripherals::PIO0, 0>,
// //         >,
// //     >,
// // ) -> ! {
// //     runner.run().await
// // }

// // #[cortex_m_rt::entry]
// // fn main() -> ! {
// //     let peripherals = embassy_rp::init(Default::default());
// //     let net_peripherals = NetPeripherals {
// //         trng: peripherals.TRNG,
// //         pio: peripherals.PIO0,
// //         dma: peripherals.DMA_CH0,
// //         pwr: peripherals.PIN_23,
// //         cs: peripherals.PIN_25,
// //         dio: peripherals.PIN_24,
// //         clk: peripherals.PIN_29,
// //     };

// //     let executor0 = EXECUTOR0.init(Executor::new());

// //     spawn_core1(
// //         peripherals.CORE1,
// //         unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
// //         move || {
// //             let executor1 = EXECUTOR1.init(Executor::new());
// //             executor1.run(|spawner| spawner.spawn(unwrap!(core1_task(spawner, net_peripherals))));
// //         },
// //     );

// //     executor0.run(|spawner| spawner.spawn(unwrap!(core0_task())));
// // }

// // #[embassy_executor::task]
// // async fn core0_task() {
// //     loop {
// //         CHANNEL.send(LedState::On).await;
// //         Timer::after_millis(100).await;
// //         CHANNEL.send(LedState::Off).await;
// //         Timer::after_millis(400).await;
// //     }
// // }

// // #[embassy_executor::task]
// // async fn core1_task(spawner: Spawner, net_peripherals: NetPeripherals) {
// //     Timer::after_secs(5).await;

// //     let pwr = Output::new(net_peripherals.pwr, Level::Low);
// //     let cs = Output::new(net_peripherals.cs, Level::High);
// //     let mut pio = Pio::new(net_peripherals.pio, Irqs1);
// //     let spi = PioSpi::new(
// //         &mut pio.common,
// //         pio.sm0,
// //         RM2_CLOCK_DIVIDER,
// //         pio.irq0,
// //         cs,
// //         net_peripherals.dio,
// //         net_peripherals.clk,
// //         dma::Channel::new(net_peripherals.dma, Irqs1),
// //     );

// //     static STATE: StaticCell<cyw43::State> = StaticCell::new();
// //     let state = STATE.init(cyw43::State::new());
// //     let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, FW, NVRAM).await;
// //     spawner.spawn(unwrap!(wifi_task(runner)));

// //     control.init(CLM).await;
// //     control
// //         .set_power_management(cyw43::PowerManagementMode::PowerSave)
// //         .await;

// //     loop {
// //         match CHANNEL.receive().await {
// //             LedState::On => control.gpio_set(0, true).await,
// //             LedState::Off => control.gpio_set(0, false).await,
// //         }
// //     }
// // }
// x

//! This example shows how to use USB (Universal Serial Bus) in the RP2040 chip as well as how to create multiple usb classes for one device
//!
//! This creates a USB serial port that echos. It will also print out logging information on a separate serial device

#![no_std]
#![no_main]

use defmt::{info, panic, unwrap};
use embassy_executor::Executor;
use embassy_futures::join::join;
use embassy_rp::Peri;
use embassy_rp::bind_interrupts;
use embassy_rp::multicore::{Stack, spawn_core1};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

static mut CORE1_STACK: Stack<8192> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

bind_interrupts!(struct Irqs0 {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| spawner.spawn(unwrap!(core1_task())));
        },
    );

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| spawner.spawn(unwrap!(core0_task(p.USB))));
}

#[embassy_executor::task]
async fn core1_task() {
    loop {
        info!("from core1!");
        Timer::after_millis(1).await;
    }
}

#[embassy_executor::task]
async fn core0_task(usb: Peri<'static, USB>) {
    info!("Hello there!");

    // Create the driver, from the HAL.
    let driver = Driver::new(usb, Irqs0);

    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();
    let mut logger_state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Create a class for the logger
    let logger_class = CdcAcmClass::new(&mut builder, &mut logger_state, 64);

    // Creates the logger and returns the logger future
    // Note: You'll need to use log::info! afterwards instead of info! for this to work (this also applies to all the other log::* macros)
    let log_fut = embassy_usb_logger::with_class!(1024, log::LevelFilter::Info, logger_class);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            class.wait_connection().await;
            log::info!("Connected");
            let _ = echo(&mut class).await;
            log::info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, join(echo_fut, log_fut)).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}
