use crate::irqs;

use defmt::unwrap;
use embassy_rp::{gpio, peripherals};
use static_cell::StaticCell;

const WIFI_NETWORK: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

static WIFI_STATE: StaticCell<cyw43::State> = StaticCell::new();
static SOCKETS: StaticCell<embassy_net::StackResources<5>> = StaticCell::new();

static CYW43_FIRMWARE: &cyw43::Aligned<cyw43::A4, [u8]> =
    cyw43::aligned_bytes!("bin/cyw43-firmware/43439A0.bin");
static CYW43_CLM: &cyw43::Aligned<cyw43::A4, [u8]> =
    cyw43::aligned_bytes!("bin/cyw43-firmware/43439A0_clm.bin");
static CYW43_NVRAM: &cyw43::Aligned<cyw43::A4, [u8]> =
    cyw43::aligned_bytes!("bin/cyw43-firmware/nvram_rp2040.bin");

const _XML_BODY: &str = "<?xml version=\"1.0\"?>\r\n\
<root xmlns=\"urn:schemas-upnp-org:device-1-0\">\r\n\
<specVersion><major>1</major><minor>0</minor></specVersion>\r\n\
<device>\r\n\
<deviceType>urn:schemas-upnp-org:device:Basic:1</deviceType>\r\n\
<friendlyName>[Samsung] Family Hub</friendlyName>\r\n\
<manufacturer>Samsung Electronics</manufacturer>\r\n\
<modelName>RF28NHEDBSR</modelName>\r\n\
<modelNumber>FamilyHub 2.0</modelNumber>\r\n\
<serialNumber>30CDA7ABCDEF</serialNumber>\r\n\
<UDN>uuid:30cda7ab-cdef-4940-8f1d-30cda7abcdef</UDN>\r\n\
</device></root>\r\n";

const _HTTP_BODY: &str = "NOTIFY * HTTP/1.1\r\n\
HOST: 239.255.255.250:1900\r\n\
CACHE-CONTROL: max-age=1800\r\n\
LOCATION: http://{:?}:80/device.xml\r\n\
NT: upnp:rootdevice\r\n\
NTS: ssap:alive\r\n\
SERVER: Tizen/5.5 UPnP/1.1 Samsung-Smart-Fridge/1.0\r\n\
USN: uuid:30cda7ab-cdef-4940-8f1d-30cda7abcdef::upnp:rootdevice\r\n\r\n";

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
        let mut pio = embassy_rp::pio::Pio::new(net_peripherals.pio, irqs::Irqs);
        let dma_channel = embassy_rp::dma::Channel::new(net_peripherals.dma, irqs::Irqs);
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

        let mut dhcp_config = embassy_net::DhcpConfig::default();

        if let Ok(hostname) = heapless::String::try_from("Samsung-FamilyHub") {
            dhcp_config.hostname = Some(hostname);
        }

        let net_config = embassy_net::Config::dhcpv4(dhcp_config);

        let sockets = SOCKETS.init(embassy_net::StackResources::<5>::new());

        let mut trng = embassy_rp::trng::Trng::new(
            net_peripherals.trng,
            irqs::Irqs,
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
            log::error!("net join failed: {:?}", err);
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
            log::error!("net stack failed, exiting.");
            return;
        }

        Self::initialize_services(spawner, stack);
    }

    fn initialize_services(spawner: embassy_executor::Spawner, stack: embassy_net::Stack<'static>) {
        spawner.spawn(unwrap!(udp_task(stack)));
        spawner.spawn(unwrap!(ssdp_task(stack)));
        spawner.spawn(unwrap!(http_task(stack)));
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
async fn udp_task(_stack: embassy_net::Stack<'static>) {
    log::info!("net::service::udp up");
}

#[embassy_executor::task]
async fn ssdp_task(_stack: embassy_net::Stack<'static>) {
    log::info!("net::service::ssdp up");
}

#[embassy_executor::task]
async fn http_task(_stack: embassy_net::Stack<'static>) {
    log::info!("net::service::http up");
}
