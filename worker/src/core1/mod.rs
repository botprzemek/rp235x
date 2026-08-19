mod display;
mod font;
mod led;

use crate::peripherals::LedPeripherals;
use crate::state::handler::CORE1_READY_SIGNAL;
use defmt::unwrap;
use display::DisplayState;
use embassy_executor::{Executor, Spawner};
use led::LedOutputs;
use static_cell::StaticCell;

pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

pub struct Core1;

impl Core1 {
    pub fn entry(led: LedPeripherals) -> ! {
        let executor = Executor::new();

        EXECUTOR.init(executor).run(|spawner| {
            spawner.spawn(unwrap!(task(spawner, led)));
        })
    }
}

#[embassy_executor::task]
async fn task(_spawner: Spawner, peripherals: LedPeripherals) {
    CORE1_READY_SIGNAL.signal(());

    let mut _outputs = LedOutputs::from(peripherals);

    let _state = DisplayState::default();
}
