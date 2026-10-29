//! Core 0 executor and state machine entry task.

use embassy_executor::{Executor, Spawner};
use static_cell::StaticCell;

use crate::peripherals::{NetPeripherals, TrngPeripherals};
use crate::state::Machine;

/// Core 0 entry point.
pub struct Core0;

/// Static async executor for Core 0.
pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

impl Core0 {
    /// Initializes the Core 0 executor and spawns the main state machine task.
    pub fn entry(net: NetPeripherals, trng: TrngPeripherals) -> ! {
        let executor = Executor::new();

        EXECUTOR.init(executor).run(|spawner| {
            spawner.spawn(defmt::unwrap!(task(spawner, net, trng)));
        })
    }
}

#[embassy_executor::task]
async fn task(spawner: Spawner, net: NetPeripherals, trng: TrngPeripherals) {
    let mut state_machine = Machine::new();

    state_machine.handle_boot().await;
    state_machine.handle_sync().await;
    state_machine.handle_networking(spawner, net, trng).await;
}
