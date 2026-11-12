//! Core 1 executor and HUB75 BCM display refresh loop.

#[cfg(feature = "rp235x")]
use crate::core1::ScoreboardLayout;
#[cfg(feature = "rp235x")]
use crate::peripherals::LedPeripherals;
#[cfg(feature = "rp235x")]
use crate::state::handler::CORE1_READY_SIGNAL;
#[cfg(feature = "rp235x")]
use crate::state::{GAME_STATE, IS_CONNECTED_TO_MASTER};

#[cfg(feature = "rp235x")]
use core::sync::atomic::Ordering;
#[cfg(feature = "rp235x")]
use embassy_executor::{Executor, Spawner};
#[cfg(feature = "rp235x")]
use embassy_futures::yield_now;
#[cfg(feature = "rp235x")]
use hub75::{CORAL_RED, DisplayBuffer, LedOutputs};
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

    defmt::info!("🚀 [Core 1] Display runner started (Hardware SIO / DMA Engine)");

    display.set_brightness(30);

    loop {
        let now = embassy_time::Instant::now();
        let delta_micros = (now - last_render).as_micros();
        last_render = now;

        let is_connected = IS_CONNECTED_TO_MASTER.load(Ordering::Acquire);

        if !is_connected {
            const PALETTE: [[u8; 3]; 3] =
                [[0xEE, 0x66, 0x3B], [0xF1, 0x56, 0x56], [0xA3, 0x49, 0xCE]];

            let elapsed_ms = (now.as_micros() / 1000) as u32;
            display.set_brightness(30);

            const STEP_DURATION_MS: u32 = 1000;
            const TOTAL_DURATION_MS: u32 = STEP_DURATION_MS * 3;

            let cycle_time = elapsed_ms % TOTAL_DURATION_MS;
            let current_step = (cycle_time / STEP_DURATION_MS) as usize;
            let next_step = (current_step + 1) % PALETTE.len();

            let step_progress = (cycle_time % STEP_DURATION_MS) * 256 / STEP_DURATION_MS;
            let inv_progress = 256 - step_progress;

            let c1 = PALETTE[current_step];
            let c2 = PALETTE[next_step];

            let current_color: [u8; 3] = [
                ((c1[0] as u32 * inv_progress + c2[0] as u32 * step_progress) / 256) as u8,
                ((c1[1] as u32 * inv_progress + c2[1] as u32 * step_progress) / 256) as u8,
                ((c1[2] as u32 * inv_progress + c2[2] as u32 * step_progress) / 256) as u8,
            ];

            display.clear();
            for y in 0..32 {
                for x in 0..64 {
                    display.set_pixel(x, y, current_color);
                }
            }

            display.prepare_bitplanes();
            outputs.render_bcm_frame_sio_fast(display.bitplanes(), 30);
        } else {
            display.set_brightness(20);
            let mut game = GAME_STATE.get();

            if game.state() == basketball::State::Running {
                game.tick(core::time::Duration::from_micros(delta_micros));
            }

            let is_connected =
                crate::state::IS_CONNECTED_TO_MASTER.load(core::sync::atomic::Ordering::Relaxed);
            let color = CORAL_RED;

            layout.render(&mut display, &game, color, is_connected);
            outputs.render_bcm_frame_sio_fast(display.bitplanes(), 20);
        }

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

            let is_connected =
                crate::state::IS_CONNECTED_TO_MASTER.load(core::sync::atomic::Ordering::Relaxed);
            layout.render(&mut display, &game, RED, is_connected);

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
