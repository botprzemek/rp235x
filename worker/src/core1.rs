use crate::led;

use defmt::unwrap;
use embassy_executor::Executor;
use static_cell::StaticCell;

pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

pub struct Core1;

impl Core1 {
    pub fn entry(peripherals: led::LedPeripherals) -> ! {
        let executor = Executor::new();

        EXECUTOR.init(executor).run(|spawner| {
            spawner.spawn(unwrap!(led::task(peripherals)));
        })
    }
}
