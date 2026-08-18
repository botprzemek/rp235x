mod display;
mod font;
mod led;

use crate::peripherals::LedPeripherals;
use crate::{channels::GAME_CHANNEL, handler::CORE1_READY_SIGNAL};
use defmt::unwrap;
use embassy_executor::{Executor, Spawner};
use embassy_rp::gpio::Level;
use font::get_ibm_pixel;
use led::LedOutputs;
use static_cell::StaticCell;

pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

pub struct Core1;

#[derive(Clone, Copy)]
struct DisplayState {
    home_score: u16,
    away_score: u16,
    quarter: game::Quarter,
    minutes: u32,
    seconds: u32,
    hundredths: u32,
}

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
    let mut outputs = LedOutputs::from(peripherals);

    let mut state = DisplayState {
        home_score: 0,
        away_score: 0,
        quarter: game::Quarter::None,
        minutes: 0,
        seconds: 0,
        hundredths: 0,
    };

    let mut home_str = heapless::String::<8>::new();
    let mut guest_str = heapless::String::<8>::new();
    let mut row2_str = heapless::String::<16>::new();

    let started = embassy_time::Instant::now();

    CORE1_READY_SIGNAL.signal(());

    loop {
        if let Ok(packet) = GAME_CHANNEL.try_receive()
            && let Ok(game) = game::Game::from_payload(&packet.data)
        {
            let time_ms = (embassy_time::Instant::now() - started).as_millis();
            defmt::info!("seq: {}; {}ms", &packet.seq_id, time_ms);

            state.minutes = game.millis / 1000 / 60;
            state.seconds = (game.millis / 1000) % 60;
            state.hundredths = (game.millis % 1000) / 10;
            state.home_score = game.home_score;
            state.away_score = game.away_score;
            state.quarter = game.quarter;
        }

        home_str.clear();
        let _ = core::fmt::write(&mut home_str, format_args!("{:03}", state.home_score));

        guest_str.clear();
        let _ = core::fmt::write(&mut guest_str, format_args!("{:03}", state.away_score));

        row2_str.clear();
        let _ = core::fmt::write(
            &mut row2_str,
            format_args!(
                "Q{} {:02}:{:02}",
                state.quarter as u8, state.minutes, state.seconds
            ),
        );

        let block_width: i32 = 3 * 8;
        let gap_px: i32 = 17;

        let total_width = (block_width * 2) + gap_px;

        let r1_start_x: i32 = (64 - total_width) / 2;
        let r1_start_y: u32 = 0;

        for row in 0..16 {
            outputs.oe.set_level(Level::High);
            cortex_m::asm::delay(10);

            outputs.a.set_level(if (row & 0x01) != 0 {
                Level::High
            } else {
                Level::Low
            });
            outputs.b.set_level(if (row & 0x02) != 0 {
                Level::High
            } else {
                Level::Low
            });
            outputs.c.set_level(if (row & 0x04) != 0 {
                Level::High
            } else {
                Level::Low
            });
            outputs.d.set_level(if (row & 0x08) != 0 {
                Level::High
            } else {
                Level::Low
            });
            cortex_m::asm::delay(10);

            let y_upper = row as u32;

            for column in 0..64 {
                let x = column;
                let mut r1 = false;
                let r2 = false;

                if y_upper >= r1_start_y && y_upper < r1_start_y + 9 && x >= r1_start_x {
                    let local_x = x - r1_start_x;

                    if local_x < block_width {
                        let char_idx = (local_x / 8) as usize;
                        let pixel_x = (local_x % 8) as u32;
                        let pixel_y = y_upper - r1_start_y;

                        if char_idx < home_str.len() && pixel_x < 7 {
                            let c = home_str.as_bytes()[char_idx];

                            if c.is_ascii_digit() {
                                let digit = c - b'0';
                                r1 = get_ibm_pixel(digit, pixel_x, pixel_y);
                            }
                        }
                    } else if local_x >= block_width + gap_px {
                        let right_local_x = local_x - (block_width + gap_px);
                        let char_idx = (right_local_x / 8) as usize;
                        let pixel_x = (right_local_x % 8) as u32;
                        let pixel_y = y_upper - r1_start_y;

                        if char_idx < guest_str.len() && pixel_x < 7 {
                            let c = guest_str.as_bytes()[char_idx];

                            if c.is_ascii_digit() {
                                let digit = c - b'0';
                                r1 = get_ibm_pixel(digit, pixel_x, pixel_y);
                            }
                        }
                    }
                }

                outputs
                    .r1
                    .set_level(if r1 { Level::High } else { Level::Low });
                outputs.g1.set_level(Level::Low);
                outputs.b1.set_level(Level::Low);

                outputs
                    .r2
                    .set_level(if r2 { Level::High } else { Level::Low });
                outputs.g2.set_level(Level::Low);
                outputs.b2.set_level(Level::Low);

                outputs.clk.set_level(Level::High);
                outputs.clk.set_level(Level::Low);
            }

            cortex_m::asm::delay(10);
            outputs.lat.set_level(Level::High);
            cortex_m::asm::delay(20);
            outputs.lat.set_level(Level::Low);
            cortex_m::asm::delay(20);

            outputs.oe.set_level(Level::Low);
            cortex_m::asm::delay(40);
            outputs.oe.set_level(Level::High);

            cortex_m::asm::delay(100);
        }

        embassy_time::Timer::after_micros(50).await;
    }
}
