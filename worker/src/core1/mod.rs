mod layout;

pub use layout::ScoreboardLayout;

use crate::state::handler::CORE1_READY_SIGNAL;
use crate::{peripherals::LedPeripherals, state::GAME_STATE};
use defmt::unwrap;
use embassy_executor::{Executor, Spawner};
use static_cell::StaticCell;

pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

pub struct Core1;

impl Core1 {
    pub fn entry(led: LedPeripherals) -> ! {
        let executor = Executor::new();

        EXECUTOR.init(executor).run(|spawner| {
            spawner.spawn(unwrap!(task(spawner, led)));

            CORE1_READY_SIGNAL.signal(());
        })
    }
}

#[embassy_executor::task]
async fn task(_spawner: Spawner, peripherals: LedPeripherals) {
    let mut outputs = hub75::LedOutputs::from(peripherals);
    let mut display = hub75::DisplayBuffer::<64, 32, 16>::default();
    let mut layout = ScoreboardLayout::new();

    let start_instant = embassy_time::Instant::now().as_millis();

    loop {
        let game = {
            let game_ref = GAME_STATE.lock().await;
            *game_ref.borrow()
        };

        let phase_hue = ((start_instant % 10_000) * 360 / 10_000) as u16;
        let color = hub75::hue_to_rgb(phase_hue);

        layout.render(&mut display, &game, color);
        outputs.render_bcm_frame_sync(display.bitplanes());

        embassy_futures::yield_now().await;
    }
}
