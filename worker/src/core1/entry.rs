//! Core 1 executor and HUB75 BCM display refresh loop.

use crate::core1::ScoreboardLayout;
use crate::peripherals::LedPeripherals;
use crate::state::GAME_STATE;
use crate::state::handler::CORE1_READY_SIGNAL;

use embassy_executor::{Executor, Spawner};
use embassy_futures::yield_now;
use hub75::{DisplayBuffer, LedOutputs, RED};
use static_cell::StaticCell;

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
    let mut last_render = embassy_time::Instant::now();

    display.set_brightness(20);

    loop {
        let now = embassy_time::Instant::now();
        let delta_micros = (now - last_render).as_micros();
        last_render = now;

        let mut game = GAME_STATE.get();

        // High-resolution local clock interpolation between periodic master sync frames
        if game.state() == basketball::State::Running {
            game.tick(core::time::Duration::from_micros(delta_micros));
        }

        let color = RED;

        layout.render(&mut display, &game, color);
        outputs.render_bcm_frame_sync(display.bitplanes());

        yield_now().await;
    }
}
