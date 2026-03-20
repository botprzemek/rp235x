#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

mod net;

use crate::net::{http_task, ssdp_task, udp_task};
use cyw43::{JoinOptions, aligned_bytes};
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Config, DhcpConfig, StackResources};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0, USB};
use embassy_rp::{bind_interrupts, dma, pio, usb};
use embassy_usb::Builder;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

const WIFI_NETWORK: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<
        'static,
        cyw43::SpiBus<Output<'static>, cyw43_pio::PioSpi<'static, PIO0, 0>>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn usb_task(driver: usb::Driver<'static, USB>) {
    static VENDOR_ID: u16 = 0x04E8;
    static PRODUCT_ID: u16 = 0x6860;

    let mut config = embassy_usb::Config::new(VENDOR_ID, PRODUCT_ID);

    config.composite_with_iads = false;

    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.device_release = 0x0200;
    config.max_packet_size_0 = 64;

    config.manufacturer = Some("Samsung");
    config.product = Some("Samsung-Tizen-FamilyHub");
    config.serial_number = Some("30CDA7ABCDEF");
    config.max_power = 500;

    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

    static LOGGER_STATE: StaticCell<embassy_usb::class::cdc_acm::State> = StaticCell::new();
    let state = LOGGER_STATE.init(embassy_usb::class::cdc_acm::State::new());

    let mut builder = Builder::new(
        driver,
        config,
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64]),
    );
    let logger_class = embassy_usb::class::cdc_acm::CdcAcmClass::new(&mut builder, state, 64);
    let logger = embassy_usb_logger::with_class!(1024, log::LevelFilter::Info, logger_class);

    let mut usb = builder.build();

    embassy_futures::join::join(usb.run(), logger).await;
}

static STATE: StaticCell<cyw43::State> = StaticCell::new();
static RESOURCES: StaticCell<StackResources<5>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = usb::Driver::new(p.USB, Irqs);

    let fw = aligned_bytes!("bin/cyw43-firmware/43439A0.bin");
    let clm = aligned_bytes!("bin/cyw43-firmware/43439A0_clm.bin");
    let nvram = aligned_bytes!("bin/cyw43-firmware/nvram_rp2040.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = pio::Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        dma::Channel::new(p.DMA_CH0, Irqs),
    );

    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw, nvram).await;
    spawner.spawn(unwrap!(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let mut dhcp = DhcpConfig::default();
    if let Ok(hostname) = heapless::String::<32>::try_from("Samsung-FamilyHub") {
        dhcp.hostname = Some(hostname);
    }

    let net_config = Config::dhcpv4(dhcp);

    let mut rng = embassy_rp::clocks::RoscRng;
    let seed = rng.next_u64();
    let (stack, runner) = embassy_net::new(
        net_device,
        net_config,
        RESOURCES.init(StackResources::<5>::new()),
        seed,
    );

    spawner.spawn(unwrap!(net_task(runner)));
    spawner.spawn(unwrap!(usb_task(driver)));

    while let Err(err) = control
        .join(WIFI_NETWORK, JoinOptions::new(WIFI_PASSWORD.as_bytes()))
        .await
    {
        log::info!("net join failed: {:?}", err);
    }

    stack.wait_link_up().await;
    stack.wait_config_up().await;

    if let Some(config) = stack.config_v4() {
        log::info!("----- net stack -----");
        log::info!("mac:     {:?}", stack.hardware_address().as_eui_64());
        log::info!(
            "ip:      {:?}/{:?}",
            config.address.address(),
            config.address.netmask()
        );
        if let Some(gateway) = config.gateway {
            log::info!("gateway: {:?}", gateway);
        }
    } else {
        log::info!("net stack failed, exiting.");
        return;
    }

    spawner.spawn(unwrap!(udp_task(stack)));
    spawner.spawn(unwrap!(ssdp_task(stack)));
    spawner.spawn(unwrap!(http_task(stack)));
}
