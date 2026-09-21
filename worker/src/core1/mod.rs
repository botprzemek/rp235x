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
    let mut display = hub75::DisplayBuffer::<64, 32, 16>::new();

    let mut home_str = heapless::String::<8>::new();
    let mut guest_str = heapless::String::<8>::new();
    let mut quarter_str = heapless::String::<8>::new();
    let mut clock_str = heapless::String::<8>::new();

    let start_instant = embassy_time::Instant::now().as_millis();

    loop {
        let game = {
            let game_ref = GAME_STATE.lock().await;
            *game_ref.borrow()
        };

        home_str.clear();
        guest_str.clear();
        quarter_str.clear();
        clock_str.clear();

        let total_seconds = game.regulation_millis() / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;

        let _ = core::fmt::write(&mut home_str, format_args!("{:03}", game.home_score()));
        let _ = core::fmt::write(&mut guest_str, format_args!("{:03}", game.away_score()));
        let _ = core::fmt::write(&mut quarter_str, format_args!("{:?}", game.quarter()));
        let _ = core::fmt::write(
            &mut clock_str,
            format_args!("{:02}:{:02}", minutes, seconds),
        );

        let phase_hue = ((start_instant % 10_000) * 360 / 10_000) as u16;
        let color = hub75::hue_to_rgb(phase_hue);

        display.draw_animated_text(color, &home_str, &guest_str, &quarter_str, &clock_str);
        outputs.render_bcm_frame_sync(display.bitplanes());

        embassy_futures::yield_now().await;
    }
}
