#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

mod irqs;
mod net;
mod usb;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    usb::Usb::init(spawner, peripherals.USB);

    net::Net::init(
        spawner,
        net::NetPeripherals {
            trng: peripherals.TRNG,
            pio: peripherals.PIO0,
            dma: peripherals.DMA_CH0,
            pwr: peripherals.PIN_23,
            cs: peripherals.PIN_25,
            dio: peripherals.PIN_24,
            clk: peripherals.PIN_29,
        },
    )
    .await;
}
