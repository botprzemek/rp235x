//! PIO, DMA, and GPIO pins assigned to the CYW43 Wi-Fi chip.

use embassy_rp::{Peri, peripherals};

/// RP235x peripherals required by the CYW43439 wireless driver.
pub struct NetPeripherals {
    pub pio: Peri<'static, peripherals::PIO0>,
    pub dma: Peri<'static, peripherals::DMA_CH0>,
    pub pwr: Peri<'static, peripherals::PIN_23>,
    pub cs: Peri<'static, peripherals::PIN_25>,
    pub dio: Peri<'static, peripherals::PIN_24>,
    pub clk: Peri<'static, peripherals::PIN_29>,
}
