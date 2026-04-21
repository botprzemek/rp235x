use embassy_rp::{dma, peripherals, pio, trng, usb};

embassy_rp::bind_interrupts!(pub struct Irqs0 {
    USBCTRL_IRQ => usb::InterruptHandler<peripherals::USB>;
});

embassy_rp::bind_interrupts!(pub struct Irqs1 {
    TRNG_IRQ => trng::InterruptHandler<peripherals::TRNG>;
    PIO0_IRQ_0 => pio::InterruptHandler<peripherals::PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<peripherals::DMA_CH0>;
});
