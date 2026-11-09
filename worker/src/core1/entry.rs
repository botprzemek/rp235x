//! Core 1 executor and HUB75 BCM display refresh loop.

#[cfg(feature = "rp235x")]
use crate::core1::ScoreboardLayout;
#[cfg(feature = "rp235x")]
use crate::peripherals::LedPeripherals;
#[cfg(feature = "rp235x")]
use crate::state::GAME_STATE;
#[cfg(feature = "rp235x")]
use crate::state::handler::CORE1_READY_SIGNAL;

#[cfg(feature = "rp235x")]
use embassy_executor::{Executor, Spawner};
#[cfg(feature = "rp235x")]
use embassy_futures::yield_now;
#[cfg(feature = "rp235x")]
use hub75::{DisplayBuffer, LedOutputs, RED};
#[cfg(feature = "rp235x")]
use static_cell::StaticCell;

#[cfg(not(feature = "rp235x"))]
use crate::core1::ScoreboardLayout;
#[cfg(not(feature = "rp235x"))]
use crate::state::GAME_STATE;
#[cfg(not(feature = "rp235x"))]
use crate::state::handler::CORE1_READY_SIGNAL;
#[cfg(not(feature = "rp235x"))]
use basketball::Scorable;
#[cfg(not(feature = "rp235x"))]
use hub75::{DisplayBuffer, RED};

/// Static async executor for Core 1 on RP235x.
#[cfg(feature = "rp235x")]
pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

/// Core 1 entry point.
pub struct Core1;

#[cfg(feature = "rp235x")]
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

#[cfg(feature = "rp235x")]
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

#[cfg(not(feature = "rp235x"))]
impl Core1 {
    /// Linux entry point for Core 1 (HUB75 display rendering thread).
    pub fn entry() {
        println!("🚀 [Core 1 Thread] HUB75 Display Runner started");
        CORE1_READY_SIGNAL.signal();

        let mut display = DisplayBuffer::<64, 32, 16>::default();
        display.set_brightness(20);
        let mut layout = ScoreboardLayout::new();
        let mut last_render = std::time::Instant::now();
        let mut last_log = std::time::Instant::now();

        loop {
            let now = std::time::Instant::now();
            let delta = now.duration_since(last_render);
            last_render = now;

            let mut game = GAME_STATE.get();

            if game.state() == basketball::State::Running {
                GAME_STATE.update(|g| {
                    if g.state() == basketball::State::Running {
                        g.tick(delta);
                    }
                });
                game = GAME_STATE.get();
            }

            let color = RED;
            layout.render(&mut display, &game, color);

            if now.duration_since(last_log) >= std::time::Duration::from_secs(1) {
                last_log = now;
                print_terminal_scoreboard(&game);
            }

            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}

#[cfg(not(feature = "rp235x"))]
fn print_terminal_scoreboard(game: &basketball::Game) {
    let total_seconds = game.regulation_millis() / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    let tenths = (game.regulation_millis() % 1000) / 100;

    let time_str = if minutes == 0 && total_seconds < 60 {
        format!("{seconds:02}.{tenths}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    };

    let scores = game.scores();
    let state_str = match game.state() {
        basketball::State::Running => "RUNNING ▶",
        basketball::State::Paused => "PAUSED ⏸",
        basketball::State::QuarterEnd => "QUARTER END ⏹",
        basketball::State::GameEnd => "GAME END 🏁",
        _ => "IDLE",
    };

    println!(
        "\r[HUB75 SIM] [GAME #{}] Q{:?} | TIME: {} | HOME {:>3} : {:<3} AWAY | [{}]",
        game.id(),
        game.quarter(),
        time_str,
        scores.home,
        scores.away,
        state_str
    );
}
