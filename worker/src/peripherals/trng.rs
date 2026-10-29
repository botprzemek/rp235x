//! Hardware True Random Number Generator (TRNG) peripheral wrapper.

use embassy_rp::trng::{self, Trng};
use embassy_rp::{Peri, peripherals};

use crate::interrupts::Irqs;

/// RP235x TRNG peripheral bundle.
pub struct TrngPeripherals {
    pub trng: Peri<'static, peripherals::TRNG>,
}

impl TrngPeripherals {
    /// Initializes the hardware TRNG and generates an 8-byte random seed.
    pub async fn init(peripherals: Self) -> [u8; 8] {
        let config = {
            let mut config = trng::Config::default();
            config.sample_count = 2500;
            config
        };

        let mut driver = Trng::new(peripherals.trng, Irqs, config);
        let mut seed = [0u8; 8];

        driver.fill_bytes(&mut seed).await;

        seed
    }
}
