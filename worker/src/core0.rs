use crate::net;

use defmt::unwrap;
use embassy_executor::Executor;
use static_cell::StaticCell;

pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

pub struct Core0;

impl Core0 {
    pub fn entry(
        net_peripherals: net::NetPeripherals,
        trng_peripherals: net::TrngPeripherals,
    ) -> ! {
        let executor = Executor::new();

        EXECUTOR.init(executor).run(|spawner| {
            spawner.spawn(unwrap!(net::task(
                spawner,
                net_peripherals,
                trng_peripherals
            )));
        })
    }
}
