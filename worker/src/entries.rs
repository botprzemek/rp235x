use crate::peripherals::{LedPeripherals, NetPeripherals, TrngPeripherals};
use crate::{core0, core1};
use defmt::unwrap;
use embassy_executor::Executor;
use static_cell::StaticCell;

pub static EXECUTOR_0: StaticCell<Executor> = StaticCell::new();
pub static EXECUTOR_1: StaticCell<Executor> = StaticCell::new();

pub struct Core0;
pub struct Core1;

impl Core0 {
    pub fn entry(net: NetPeripherals, trng: TrngPeripherals) -> ! {
        let executor = Executor::new();

        EXECUTOR_0.init(executor).run(|spawner| {
            spawner.spawn(unwrap!(core0::task(spawner, net, trng)));
        })
    }
}

impl Core1 {
    pub fn entry(led: LedPeripherals) -> ! {
        let executor = Executor::new();

        EXECUTOR_1.init(executor).run(|spawner| {
            spawner.spawn(unwrap!(core1::task(led)));
        })
    }
}
