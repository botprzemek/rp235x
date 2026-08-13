use embassy_rp::{
    bind_interrupts, dma,
    peripherals::{DMA_CH0, PIO0, PIO1, TRNG},
    pio, trng,
};

bind_interrupts!(pub struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    PIO1_IRQ_0 => pio::InterruptHandler<PIO1>;
    TRNG_IRQ => trng::InterruptHandler<TRNG>;
});
