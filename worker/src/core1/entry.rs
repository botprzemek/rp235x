//! Core 1 executor and HUB75 BCM display refresh loop.

use embassy_executor::{Executor, Spawner};
use embassy_futures::yield_now;
use hub75::{DisplayBuffer, LedOutputs, RED};
use static_cell::StaticCell;

use crate::core1::ScoreboardLayout;
use crate::peripherals::LedPeripherals;
use crate::state::GAME_STATE;
use crate::state::handler::CORE1_READY_SIGNAL;

/// Static async executor for Core 1.
pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

/// Core 1 entry point.
pub struct Core1;

impl Core1 {
    /// Initializes the Core 1 executor, spawns the HUB75 refresh task, and signals Core 0.
    pub fn entry(led: LedPeripherals) -> ! {
        let executor = Executor::new();

        EXECUTOR.init(executor).run(|spawner| {
            spawner.spawn(defmt::unwrap!(task(spawner, led)));

            CORE1_READY_SIGNAL.signal(());
        })
    }
}

#[embassy_executor::task]
async fn task(_spawner: Spawner, peripherals: LedPeripherals) {
    let mut outputs = LedOutputs::from_peripherals(peripherals);
    let mut display = DisplayBuffer::<64, 32, 16>::default();
    let mut layout = ScoreboardLayout::new();

    display.set_brightness(20);

    loop {
        let game = {
            let game_ref = GAME_STATE.lock().await;
            *game_ref.borrow()
        };

        let color = RED;

        layout.render(&mut display, &game, color);
        outputs.render_bcm_frame_sync(display.bitplanes());

        yield_now().await;
    }
}
