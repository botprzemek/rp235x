use crate::irqs;

use defmt::unwrap;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_rp::{gpio, peripherals, rom_data};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

const WIFI_NETWORK: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

static WIFI_STATE: StaticCell<cyw43::State> = StaticCell::new();
static SOCKETS: StaticCell<embassy_net::StackResources<5>> = StaticCell::new();

static UDP_RX_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
static UDP_TX_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();

static CYW43_FIRMWARE: &cyw43::Aligned<cyw43::A4, [u8]> =
    cyw43::aligned_bytes!("bin/cyw43-firmware/43439A0.bin");
static CYW43_CLM: &cyw43::Aligned<cyw43::A4, [u8]> =
    cyw43::aligned_bytes!("bin/cyw43-firmware/43439A0_clm.bin");
static CYW43_NVRAM: &cyw43::Aligned<cyw43::A4, [u8]> =
    cyw43::aligned_bytes!("bin/cyw43-firmware/nvram_rp2040.bin");

pub struct NetPeripherals {
    pub trng: embassy_rp::Peri<'static, peripherals::TRNG>,
    pub pio: embassy_rp::Peri<'static, peripherals::PIO0>,
    pub dma: embassy_rp::Peri<'static, peripherals::DMA_CH0>,
    pub pwr: embassy_rp::Peri<'static, peripherals::PIN_23>,
    pub cs: embassy_rp::Peri<'static, peripherals::PIN_25>,
    pub dio: embassy_rp::Peri<'static, peripherals::PIN_24>,
    pub clk: embassy_rp::Peri<'static, peripherals::PIN_29>,
}

pub struct Net;

impl Net {
    pub async fn init(spawner: embassy_executor::Spawner, net_peripherals: NetPeripherals) {
        let pwr = embassy_rp::gpio::Output::new(net_peripherals.pwr, embassy_rp::gpio::Level::Low);
        let cs = embassy_rp::gpio::Output::new(net_peripherals.cs, embassy_rp::gpio::Level::High);
        let mut pio = embassy_rp::pio::Pio::new(net_peripherals.pio, irqs::Irqs1);
        let dma_channel = embassy_rp::dma::Channel::new(net_peripherals.dma, irqs::Irqs1);
        let spi = cyw43_pio::PioSpi::new(
            &mut pio.common,
            pio.sm0,
            cyw43_pio::DEFAULT_CLOCK_DIVIDER,
            pio.irq0,
            cs,
            net_peripherals.dio,
            net_peripherals.clk,
            dma_channel,
        );

        let state = WIFI_STATE.init(cyw43::State::new());
        let (net_device, mut control, runner) =
            cyw43::new(state, pwr, spi, CYW43_FIRMWARE, CYW43_NVRAM).await;

        spawner.spawn(unwrap!(wifi_task(runner)));

        control.init(CYW43_CLM).await;
        control
            .set_power_management(cyw43::PowerManagementMode::PowerSave)
            .await;

        let dhcp_config = embassy_net::DhcpConfig::default();
        let net_config = embassy_net::Config::dhcpv4(dhcp_config);

        let sockets = SOCKETS.init(embassy_net::StackResources::<5>::new());

        let mut trng = embassy_rp::trng::Trng::new(
            net_peripherals.trng,
            irqs::Irqs1,
            embassy_rp::trng::Config::default(),
        );

        let mut seed = [0u8; 8];

        trng.fill_bytes(&mut seed).await;

        let seed_u64 = u64::from_le_bytes(seed);

        let (stack, runner) = embassy_net::new(net_device, net_config, sockets, seed_u64);

        spawner.spawn(unwrap!(net_task(runner)));

        while let Err(err) = control
            .join(
                WIFI_NETWORK,
                cyw43::JoinOptions::new(WIFI_PASSWORD.as_bytes()),
            )
            .await
        {
            log::error!("net::join down {:?}", err);
        }

        stack.wait_link_up().await;
        stack.wait_config_up().await;

        if let Some(config) = stack.config_v4() {
            let mut hardware_address: heapless::String<24> = heapless::String::new();
            let hw = stack.hardware_address();
            let hw_bytes = hw.as_bytes();

            for (i, b) in hw_bytes.iter().enumerate() {
                for &nibble in &[(b >> 4), (b & 0xf)] {
                    let c = if nibble < 10 {
                        (b'0' + nibble) as char
                    } else {
                        (b'A' + nibble - 10) as char
                    };
                    let _ = hardware_address.push(c);
                }
                if i < hw_bytes.len() - 1 {
                    let _ = hardware_address.push(':');
                }
            }

            log::info!("net::stack up");
            log::info!("   ::mac     {}", hardware_address);
            log::info!("   ::ip      {}", config.address.address());

            if let Some(gateway) = config.gateway {
                log::info!("   ::gateway {}", gateway);
            }
        } else {
            log::error!("net stack failed, exiting.");
            return;
        }

        let rx_buffer = UDP_RX_BUFFER.init([0; 256]);
        let tx_buffer = UDP_TX_BUFFER.init([0; 256]);

        spawner.spawn(unwrap!(udp_task(stack, rx_buffer, tx_buffer)));
    }
}

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        cyw43::SpiBus<gpio::Output<'static>, cyw43_pio::PioSpi<'static, peripherals::PIO0, 0>>,
    >,
) -> ! {
    log::info!("net::device up");
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    log::info!("net::connection up");
    runner.run().await
}

#[embassy_executor::task]
async fn udp_task(
    stack: embassy_net::Stack<'static>,
    rx_buffer: &'static mut [u8; 256],
    tx_buffer: &'static mut [u8; 256],
) {
    log::info!("net::service::udp up");
    log::info!("   ::listen :9000");

    let mut rx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_meta = [PacketMetadata::EMPTY; 2];
    let mut buf = [0; 16];

    let mut socket = UdpSocket::new(stack, &mut rx_meta, rx_buffer, &mut tx_meta, tx_buffer);
    unwrap!(socket.bind(9000));

    loop {
        let (n, ep) = unwrap!(socket.recv_from(&mut buf).await);

        if let Ok(s) = core::str::from_utf8(&buf[..n]) {
            if s == "BOOTSEL" {
                log::info!("net::service::udp (to {}): {}", ep, s);
                unwrap!(socket.send_to(&buf[..n], ep).await);
                Timer::after(Duration::from_secs(1)).await;

                rom_data::reset_to_usb_boot(0, 0);
            }

            log::info!("ECHO (to {}): {}", ep, s);
        } else {
            log::info!("ECHO (to {}): bytearray len {}", ep, n);
        }

        unwrap!(socket.send_to(&buf[..n], ep).await);
    }
}
