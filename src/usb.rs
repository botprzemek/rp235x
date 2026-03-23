use crate::irqs;

use defmt::unwrap;
use embassy_futures::join::join;
use embassy_rp::peripherals;
use embassy_rp::usb;
use embassy_usb::class::cdc_acm;
use embassy_usb_logger::with_class;
use static_cell::StaticCell;

static CONFIG_DESCRIPTOR_BUF: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR_BUF: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
static LOGGER_STATE: StaticCell<cdc_acm::State> = StaticCell::new();

const USB_CONFIG: embassy_usb::Config<'static> = {
    let mut config = embassy_usb::Config::new(0x04E8, 0x6860);

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

    config
};

pub struct Usb;

impl Usb {
    pub fn init(
        spawner: embassy_executor::Spawner,
        usb: embassy_rp::Peri<'static, embassy_rp::peripherals::USB>,
    ) {
        let driver = embassy_rp::usb::Driver::new(usb, irqs::Irqs);

        spawner.spawn(unwrap!(usb_task(driver)));
    }
}

#[embassy_executor::task]
async fn usb_task(driver: usb::Driver<'static, peripherals::USB>) -> () {
    let logger_state = LOGGER_STATE.init(cdc_acm::State::new());

    let mut usb_builder = embassy_usb::Builder::new(
        driver,
        USB_CONFIG,
        CONFIG_DESCRIPTOR_BUF.init([0; 256]),
        BOS_DESCRIPTOR_BUF.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64]),
    );

    let logger_class = cdc_acm::CdcAcmClass::new(&mut usb_builder, logger_state, 64);
    let with_logger = with_class!(2048, log::LevelFilter::Debug, logger_class);
    let mut usb_device = usb_builder.build();

    join(usb_device.run(), with_logger).await;

    log::info!("usb::device up");
    log::info!("usb::logger up");
}
