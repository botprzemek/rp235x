use embassy_rp::{Peri, peripherals};

pub struct NetPeripherals {
    pub pio: Peri<'static, peripherals::PIO0>,
    pub dma: Peri<'static, peripherals::DMA_CH0>,
    pub pwr: Peri<'static, peripherals::PIN_23>,
    pub cs: Peri<'static, peripherals::PIN_25>,
    pub dio: Peri<'static, peripherals::PIN_24>,
    pub clk: Peri<'static, peripherals::PIN_29>,
}
