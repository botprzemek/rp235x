use crate::interrupts::Irqs;
use embassy_rp::{Peri, peripherals, trng};

pub struct TrngPeripherals {
    pub trng: Peri<'static, peripherals::TRNG>,
}

impl TrngPeripherals {
    pub async fn init(peripherals: TrngPeripherals) -> [u8; 8] {
        let config = {
            let mut config = trng::Config::default();
            config.sample_count = 2500;

            config
        };

        let mut driver = trng::Trng::new(peripherals.trng, Irqs, config);
        let mut seed = [0u8; 8];

        driver.fill_bytes(&mut seed).await;

        seed
    }
}
