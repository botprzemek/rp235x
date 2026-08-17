mod display;
mod font;
mod led;

use crate::peripherals::LedPeripherals;
use led::LedOutputs;

#[embassy_executor::task]
pub async fn task(peripherals: LedPeripherals) {
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
        // 1. Sprawdzanie pakietu z sieci
        if let Ok(packet) = shared::channels::GAME_CHANNEL.try_receive()
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

        // 2. Formatowanie osobno dla obu bloków punktów
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

        // --- Konfiguracja układu górnego rzędu ---
        // Każdy blok ma 3 znaki po 8 pikseli (7px szerokości fontu + 1px odstępu wewnętrznego) = 24px
        let block_width: i32 = 3 * 8;
        let gap_px: i32 = 17; // Konfigurowalny odstęp między blokami (np. 4 piksele)

        // Całkowita szerokość obu bloków + przerwa
        let total_width = (block_width * 2) + gap_px;

        // Wyśrodkowanie całości na matrycy o szerokości 64px
        let r1_start_x: i32 = (64 - total_width) / 2;
        let r1_start_y: u32 = 0; // Górny rząd (wysokość 9 pikseli: 0..8)

        // let r2_start_x: i32 = 0;
        // let r2_start_y: u32 = 9;

        // --- MULTIPLEKSOWANIE EKRANU HUB75 ---
        for row in 0..16 {
            outputs.oe.set_level(Level::High);
            cortex_m::asm::delay(10);

            // Adresowanie linii (A, B, C, D)
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

                // ================= GÓRNA POŁÓWKA: WYNIK (Dwa bloki + Gap) =================
                if y_upper >= r1_start_y && y_upper < r1_start_y + 9 && x >= r1_start_x {
                    let local_x = x - r1_start_x;

                    // 1. Sprawdzamy lewy blok (home_str)
                    if local_x < block_width {
                        let char_idx = (local_x / 8) as usize;
                        let pixel_x = (local_x % 8) as u32;
                        let pixel_y = y_upper - r1_start_y;

                        if char_idx < home_str.len() && pixel_x < 7 {
                            let c = home_str.as_bytes()[char_idx];
                            if c >= b'0' && c <= b'9' {
                                let digit = c - b'0';
                                r1 = get_ibm_pixel(digit, pixel_x, pixel_y);
                            }
                        }
                    }
                    // 2. Prawy blok (guest_str) zaczyna się po uwzględnieniu szerokości lewego bloku i gapu
                    else if local_x >= block_width + gap_px {
                        let right_local_x = local_x - (block_width + gap_px);
                        let char_idx = (right_local_x / 8) as usize;
                        let pixel_x = (right_local_x % 8) as u32;
                        let pixel_y = y_upper - r1_start_y;

                        if char_idx < guest_str.len() && pixel_x < 7 {
                            let c = guest_str.as_bytes()[char_idx];
                            if c >= b'0' && c <= b'9' {
                                let digit = c - b'0';
                                r1 = get_ibm_pixel(digit, pixel_x, pixel_y);
                            }
                        }
                    }
                    // W przestrzeni między `block_width` a `block_width + gap_px` nic nie rysujemy (r1 pozostaje false – czarny odstęp)
                }

                // Wypchnięcie stanów na piny sterujące
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

                // Taktowanie zegara CLK
                outputs.clk.set_level(Level::High);
                outputs.clk.set_level(Level::Low);
            }

            // Zatrzask danych (LAT)
            cortex_m::asm::delay(10);
            outputs.lat.set_level(Level::High);
            cortex_m::asm::delay(20);
            outputs.lat.set_level(Level::Low);
            cortex_m::asm::delay(20);

            // Wyzwolenie wyjścia (OE)
            outputs.oe.set_level(Level::Low);
            cortex_m::asm::delay(40);
            outputs.oe.set_level(Level::High);

            cortex_m::asm::delay(100);
        }

        embassy_time::Timer::after_micros(50).await;
    }
}
