pub mod cyw43;
pub mod net;

use crate::peripherals::{NetPeripherals, TrngPeripherals};
use crate::state::{Input, Machine, State, handler::Handler};
use defmt::unwrap;
use embassy_executor::{Executor, Spawner};
use static_cell::StaticCell;

pub struct Core0;

pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

impl Core0 {
    pub fn entry(net: NetPeripherals, trng: TrngPeripherals) -> ! {
        let executor = Executor::new();

        EXECUTOR.init(executor).run(|spawner| {
            spawner.spawn(unwrap!(task(spawner, net, trng)));
        })
    }
}

#[embassy_executor::task]
async fn task(spawner: Spawner, net: NetPeripherals, trng: TrngPeripherals) {
    let mut state_machine = Machine::new();

    state_machine.handle_boot().await;
    state_machine.handle_sync().await;
    state_machine.handle_networking(spawner, net, trng).await;

    loop {
        match state_machine.current_state() {
            State::Running => state_machine.handle_running().await,
            State::ErrorRecovery => state_machine.handle_error_recovery().await,
            _ => state_machine.transition(Input::ErrorOccurred),
        }
    }
}
