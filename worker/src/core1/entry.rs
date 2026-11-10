//! Core 1 executor and HUB75 BCM display refresh loop.

#[cfg(feature = "rp235x")]
use crate::core1::ScoreboardLayout;
#[cfg(feature = "rp235x")]
use crate::peripherals::LedPeripherals;
#[cfg(feature = "rp235x")]
use crate::state::{GAME_STATE, IS_CONNECTED_TO_MASTER};
#[cfg(feature = "rp235x")]
use crate::state::handler::CORE1_READY_SIGNAL;

#[cfg(feature = "rp235x")]
use basketball::Scorable;
#[cfg(feature = "rp235x")]
use core::sync::atomic::Ordering;
#[cfg(feature = "rp235x")]
use embassy_executor::{Executor, Spawner};
#[cfg(feature = "rp235x")]
use embassy_futures::yield_now;
#[cfg(feature = "rp235x")]
use hub75::{DisplayBuffer, LedOutputs, RED, hue_to_rgb};
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
    defmt::info!("🚀 [CORE1] HUB75 Display Task starting...");
    let mut outputs = LedOutputs::from_peripherals(peripherals);
    let mut display = DisplayBuffer::<64, 32, 16>::default();
    let mut layout = ScoreboardLayout::new();
    let mut last_render = embassy_time::Instant::now();
    let mut last_log = embassy_time::Instant::now();
    let mut frame_count: u32 = 0;
    let mut last_brightness_logged: u8 = 0;

    display.set_brightness(20);
    defmt::info!("🚀 [CORE1] Initialized HUB75 display buffer (brightness=20)");

    loop {
        let now = embassy_time::Instant::now();
        let delta_micros = (now - last_render).as_micros();
        last_render = now;

        let is_connected = IS_CONNECTED_TO_MASTER.load(Ordering::Acquire);

        if !is_connected {
            // Diagnostic test pattern while waiting for Master connection:
            // 1. Horizontal stepped gradient: transition Orange -> Green -> Cyan -> Blue -> Purple -> Red
            //    with 8 discrete columns/steps (8 pixels each), smoothly animated across the panel.
            // 2. Smooth brightness breathing cycle between 10% and 100% over 10 seconds (5s up, 5s down).
            let elapsed_ms = (now.as_micros() / 1000) as u32;

            // Brightness breathing 10% - 100% (10s period)
            let t_10s = elapsed_ms % 10_000;
            let triangle = if t_10s <= 5_000 {
                2 * t_10s
            } else {
                2 * (10_000 - t_10s)
            };
            let brightness = (10 + (90 * triangle) / 10_000) as u8;
            display.set_brightness(brightness);

            // Shifting color offset over time for smooth animated transition
            // Speed: 1 full color cycle every ~6 seconds
            let time_hue_offset = ((elapsed_ms / 16) % 360) as u16;

            display.clear();
            for x in 0..64 {
                // Stepped bands: 8 steps across 64 pixels (each step = 8 pixels wide)
                let step_index = (x / 8) as u16;
                // Base hue: start at Orange (~30°), stepping through Green (~110°), Cyan (~180°), Blue (~240°), etc.
                let step_hue = 30 + (step_index * 45);
                let hue = (step_hue + time_hue_offset) % 360;
                let color = hue_to_rgb(hue);

                for y in 0..32 {
                    display.set_pixel(x, y, color);
                }
            }
            display.prepare_bitplanes();
            outputs.render_bcm_frame_sync(display.bitplanes());

            if brightness != last_brightness_logged && (now - last_log).as_secs() >= 1 {
                last_brightness_logged = brightness;
                last_log = now;
                defmt::info!(
                    "[CORE1 DIAGNOSTIC] Waiting for Master... Stepped gradient active, Brightness: {}%, FPS: {}",
                    brightness,
                    frame_count
                );
                frame_count = 0;
            } else {
                frame_count += 1;
            }
        } else {
            display.set_brightness(20);
            let mut game = GAME_STATE.get();

            // High-resolution local clock interpolation between periodic master sync frames
            if game.state() == basketball::State::Running {
                game.tick(core::time::Duration::from_micros(delta_micros));
            }

            let color = RED;

            layout.render(&mut display, &game, color);
            outputs.render_bcm_frame_sync(display.bitplanes());

            frame_count += 1;
            if (now - last_log).as_secs() >= 1 {
                last_log = now;
                defmt::info!(
                    "[CORE1] Render FPS: {} | Game ID: {} | Q{:?} | Time: {}ms | Score: {} - {}",
                    frame_count,
                    game.id(),
                    game.quarter(),
                    game.regulation_millis(),
                    game.score(basketball::Team::Home),
                    game.score(basketball::Team::Away)
                );
                frame_count = 0;
            }
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
