use embassy_rp::Peri;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals;

use crate::shared;

pub struct LedPeripherals {
    pub r1: Peri<'static, peripherals::PIN_2>,
    pub g1: Peri<'static, peripherals::PIN_3>,
    pub b1: Peri<'static, peripherals::PIN_4>,
    pub r2: Peri<'static, peripherals::PIN_5>,
    pub g2: Peri<'static, peripherals::PIN_8>,
    pub b2: Peri<'static, peripherals::PIN_9>,

    pub a: Peri<'static, peripherals::PIN_10>,
    pub b: Peri<'static, peripherals::PIN_16>,
    pub c: Peri<'static, peripherals::PIN_18>,
    pub d: Peri<'static, peripherals::PIN_20>,
    pub e: Peri<'static, peripherals::PIN_22>,

    pub clk: Peri<'static, peripherals::PIN_11>,
    pub lat: Peri<'static, peripherals::PIN_12>,
    pub oe: Peri<'static, peripherals::PIN_13>,
}

pub struct LedOutputs {
    pub r1: Output<'static>,
    pub g1: Output<'static>,
    pub b1: Output<'static>,
    pub r2: Output<'static>,
    pub g2: Output<'static>,
    pub b2: Output<'static>,

    pub a: Output<'static>,
    pub b: Output<'static>,
    pub c: Output<'static>,
    pub d: Output<'static>,
    pub _e: Output<'static>,

    pub clk: Output<'static>,
    pub lat: Output<'static>,
    pub oe: Output<'static>,
}

impl From<LedPeripherals> for LedOutputs {
    fn from(peripherals: LedPeripherals) -> Self {
        Self {
            r1: Output::new(peripherals.r1, Level::Low),
            g1: Output::new(peripherals.g1, Level::Low),
            b1: Output::new(peripherals.b1, Level::Low),
            r2: Output::new(peripherals.r2, Level::Low),
            g2: Output::new(peripherals.g2, Level::Low),
            b2: Output::new(peripherals.b2, Level::Low),

            a: Output::new(peripherals.a, Level::Low),
            b: Output::new(peripherals.b, Level::Low),
            c: Output::new(peripherals.c, Level::Low),
            d: Output::new(peripherals.d, Level::Low),
            _e: Output::new(peripherals.e, Level::Low),

            clk: Output::new(peripherals.clk, Level::Low),
            lat: Output::new(peripherals.lat, Level::Low),
            oe: Output::new(peripherals.oe, Level::Low),
        }
    }
}

pub const IBM_FONT_7X9: [[u8; 9]; 10] = [
    // '0'
    [
        0b01111100, 0b11000110, 0b11001110, 0b11011110, 0b11110110, 0b11100110, 0b11000110,
        0b11000110, 0b01111100,
    ],
    // '1'
    [
        0b00011000, 0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
        0b01111110, 0b00000000,
    ],
    // '2'
    [
        0b01111100, 0b11000110, 0b00000110, 0b00011100, 0b00110000, 0b01100000, 0b11000000,
        0b11111110, 0b00000000,
    ],
    // '3'
    [
        0b01111100, 0b11000110, 0b00000110, 0b00111100, 0b00000110, 0b00000110, 0b11000110,
        0b01111100, 0b00000000,
    ],
    // '4'
    [
        0b00001100, 0b00011100, 0b00110100, 0b01100100, 0b11000100, 0b11111110, 0b00000100,
        0b00000100, 0b00000000,
    ],
    // '5'
    [
        0b11111110, 0b11000000, 0b11000000, 0b11111100, 0b00000110, 0b00000110, 0b11000110,
        0b01111100, 0b00000000,
    ],
    // '6'
    [
        0b00111100, 0b01100000, 0b11000000, 0b11111100, 0b11000110, 0b11000110, 0b11000110,
        0b01111100, 0b00000000,
    ],
    // '7'
    [
        0b11111110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b01100000, 0b01100000,
        0b01100000, 0b00000000,
    ],
    // '8'
    [
        0b01111100, 0b11000110, 0b11000110, 0b01111100, 0b11000110, 0b11000110, 0b11000110,
        0b01111100, 0b00000000,
    ],
    // '9'
    [
        0b01111100, 0b11000110, 0b11000110, 0b11000110, 0b01111110, 0b00000110, 0b01100110,
        0b00111100, 0b00000000,
    ],
];

static CHARS: [[u8; 5]; 11] = [
    // 0
    [0b01111100, 0b10001010, 0b10010010, 0b10100010, 0b01111100],
    // 1
    [0b00000010, 0b01000010, 0b11111110, 0b00000010, 0b00000010],
    // 2
    [0b01000110, 0b10001010, 0b10010010, 0b10010010, 0b01100010],
    // 3
    [0b01000100, 0b10000010, 0b10010010, 0b10010010, 0b01101100],
    // 4
    [0b00011000, 0b00101000, 0b01001000, 0b10001000, 0b11111110],
    // 5
    [0b11100100, 0b10100010, 0b10100010, 0b10100010, 0b10011100],
    // 6
    [0b00111100, 0b01010010, 0b10010010, 0b10010010, 0b00001100],
    // 7
    [0b11000000, 0b10000000, 0b10001110, 0b10010000, 0b11100000],
    // 8
    [0b01101100, 0b10010010, 0b10010010, 0b10010010, 0b01101100],
    // 9
    [0b01100000, 0b10010010, 0b10010010, 0b10010100, 0b01111000],
    // :
    [0b00000000, 0b00000000, 0b01101100, 0b00000000, 0b00000000],
];

pub fn get_char_pixel(char: usize, x: u32, y: u32) -> bool {
    if x >= 5 || y >= 8 || char > 10 {
        return false;
    }

    let column_data = CHARS[char][x as usize];

    (column_data & (0x80 >> y)) != 0
}

#[inline]
pub fn get_ibm_pixel(digit: u8, pixel_x: u32, pixel_y: u32) -> bool {
    if digit > 9 || pixel_x >= 7 || pixel_y >= 9 {
        return false;
    }

    let row_data = IBM_FONT_7X9[digit as usize][pixel_y as usize];

    (row_data & (1 << (7 - pixel_x))) != 0
}

// #[embassy_executor::task]
// async fn task(_spawner: embassy_executor::Spawner, peripherals: led::LedPeripherals) {
//     let mut outputs = led::LedOutputs::from(peripherals);

//     let text_start_x: u32 = 8;
//     let text_end_x: u32 = 8 + 47;
//     let text_start_y: u32 = 12;
//     let text_end_y: u32 = 12 + 8;

//     let mut hours: u8 = 0;
//     let mut minutes: u8 = 0;
//     let mut seconds: u8 = 0;

//     // Przykładowe kolory RGB (wartości 3-bitowe: 0..7) dla poszczególnych sekcji zegara
//     // Możesz je zmieniać dynamicznie!
//     let color_hours = (7, 0, 0); // Czerwony (R=7, G=0, B=0)
//     let color_colon = (4, 4, 4); // Biały/Szary (R=4, G=4, B=4)
//     let color_minutes = (0, 7, 0); // Zielony (R=0, G=7, B=0)
//     let color_seconds = (0, 0, 7); // Niebieski (R=0, G=0, B=7)

//     loop {
//         if let Ok(update) = shared::CLOCK_CHANNEL.try_receive() {
//             hours = update.hours;
//             minutes = update.minutes;
//             seconds = update.seconds;
//         }

//         let get_font_and_color = |char_idx: u32| -> (usize, (u8, u8, u8)) {
//             match char_idx {
//                 0 => ((hours / 10) as usize, color_hours),
//                 1 => ((hours % 10) as usize, color_hours),
//                 2 => (10, color_colon), // :
//                 3 => ((minutes / 10) as usize, color_minutes),
//                 4 => ((minutes % 10) as usize, color_minutes),
//                 5 => (10, color_colon), // :
//                 6 => ((seconds / 10) as usize, color_seconds),
//                 7 => ((seconds % 10) as usize, color_seconds),
//                 _ => (0, (0, 0, 0)),
//             }
//         };

//         // --- MULTIPLEKSOWANIE EKRANU HUB75 Z BWM (3 bity jasności) ---
//         for row in 0..16 {
//             // Adresowanie linii (wspólne dla wszystkich bitów jasności tego wiersza)
//             outputs.a.set_level(if (row & 0x01) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             outputs.b.set_level(if (row & 0x02) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             outputs.c.set_level(if (row & 0x04) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             outputs.d.set_level(if (row & 0x08) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             cortex_m::asm::delay(10);

//             let y_upper = row as u32;
//             let y_lower = row as u32 + 16;

//             // Iteracja po bitach jasności (BWM): bit 0 (LSB), bit 1, bit 2 (MSB)
//             for bwm_bit in 0..3 {
//                 let bit_mask = 1 << bwm_bit;

//                 // Wyłączamy OE na czas zatrzaskiwania i wpychania nowych danych
//                 outputs.oe.set_level(Level::High);

//                 for column in 0..64 {
//                     let x = column as u32;

//                     let mut r1 = false;
//                     let mut g1 = false;
//                     let mut b1 = false;
//                     let mut r2 = false;
//                     let mut g2 = false;
//                     let mut b2 = false;

//                     // --- Górna połowa (R1, G1, B1) ---
//                     if x >= text_start_x
//                         && x < text_end_x
//                         && y_upper >= text_start_y
//                         && y_upper < text_end_y
//                     {
//                         let local_x = x - text_start_x;
//                         let char_idx = local_x / 6;
//                         let pixel_x = local_x % 6;
//                         let pixel_y = y_upper - text_start_y;

//                         if pixel_x < 5 {
//                             let (font_idx, (r_val, g_val, b_val)) = get_font_and_color(char_idx);
//                             if led::get_char_pixel(font_idx, pixel_x, pixel_y) {
//                                 // Sprawdzamy, czy dany bit koloru jest zapalony w obecnej wadze BWM
//                                 r1 = (r_val & bit_mask) != 0;
//                                 g1 = (g_val & bit_mask) != 0;
//                                 b1 = (b_val & bit_mask) != 0;
//                             }
//                         }
//                     }

//                     // --- Dolna połowa (R2, G2, B2) ---
//                     if x >= text_start_x
//                         && x < text_end_x
//                         && y_lower >= text_start_y
//                         && y_lower < text_end_y
//                     {
//                         let local_x = x - text_start_x;
//                         let char_idx = local_x / 6;
//                         let pixel_x = local_x % 6;
//                         let pixel_y = y_lower - text_start_y;

//                         if pixel_x < 5 {
//                             let (font_idx, (r_val, g_val, b_val)) = get_font_and_color(char_idx);
//                             if led::get_char_pixel(font_idx, pixel_x, pixel_y) {
//                                 r2 = (r_val & bit_mask) != 0;
//                                 g2 = (g_val & bit_mask) != 0;
//                                 b2 = (b_val & bit_mask) != 0;
//                             }
//                         }
//                     }

//                     // Wypchnięcie stanów RGB na piny dla obecnego bitu
//                     outputs
//                         .r1
//                         .set_level(if r1 { Level::High } else { Level::Low });
//                     outputs
//                         .g1
//                         .set_level(if g1 { Level::High } else { Level::Low });
//                     outputs
//                         .b1
//                         .set_level(if b1 { Level::High } else { Level::Low });

//                     outputs
//                         .r2
//                         .set_level(if r2 { Level::High } else { Level::Low });
//                     outputs
//                         .g2
//                         .set_level(if g2 { Level::High } else { Level::Low });
//                     outputs
//                         .b2
//                         .set_level(if b2 { Level::High } else { Level::Low });

//                     // Taktowanie zegarem (Shift Register)
//                     outputs.clk.set_level(Level::High);
//                     outputs.clk.set_level(Level::Low);
//                 }

//                 // Zatrzask danych (LAT) po załadowaniu całej linii dla tego bitu jasności
//                 cortex_m::asm::delay(10);
//                 outputs.lat.set_level(Level::High);
//                 cortex_m::asm::delay(10);
//                 outputs.lat.set_level(Level::Low);

//                 // --- BWM Taktowanie czasowe jasności ---
//                 // Czas świecenia (OE Low) musi być proporcjonalny do wagi bitu:
//                 // Bit 0: baza (np. 15 cykli/opóźnienia)
//                 // Bit 1: baza * 2 (np. 30 cykli)
//                 // Bit 2: baza * 4 (np. 60 cykli)
//                 let oe_delay = match bwm_bit {
//                     0 => 20,
//                     1 => 40,
//                     2 => 80,
//                     _ => 0,
//                 };

//                 // Włączenie wyświetlania na wyliczony czas
//                 outputs.oe.set_level(Level::Low);
//                 cortex_m::asm::delay(oe_delay);
//                 outputs.oe.set_level(Level::High);
//             }

//             // Krótkie opóźnienie między wierszami
//             cortex_m::asm::delay(50);
//         }

//         // Ustępujemy miejsca egzekutorowi Embassy
//         embassy_time::Timer::after_micros(50).await;
//     }
// }

// #[embassy_executor::task]
// async fn task(spawner: embassy_executor::Spawner, peripherals: led::LedPeripherals) {
//     let mut outputs = led::LedOutputs::from(peripherals);

//     let text_start_x: u32 = 8;
//     let text_end_x: u32 = 8 + 47;
//     let text_start_y: u32 = 12;
//     let text_end_y: u32 = 12 + 8;

//     let mut hours: u8 = 0;
//     let mut minutes: u8 = 0;
//     let mut seconds: u8 = 0;

//     loop {
//         if let Ok(update) = shared::CLOCK_CHANNEL.try_receive() {
//             hours = update.hours;
//             minutes = update.minutes;
//             seconds = update.seconds;
//         }

//         let get_font_for_char = |char_idx: u32| -> usize {
//             match char_idx {
//                 0 => (hours / 10) as usize,
//                 1 => (hours % 10) as usize,
//                 2 => 10, // :
//                 3 => (minutes / 10) as usize,
//                 4 => (minutes % 10) as usize,
//                 5 => 10, // :
//                 6 => (seconds / 10) as usize,
//                 7 => (seconds % 10) as usize,
//                 _ => 0,
//             }
//         };

//         // --- MULTIPLEKSOWANIE EKRANU HUB75 ---
//         for row in 0..16 {
//             outputs.oe.set_level(Level::High);
//             cortex_m::asm::delay(10);

//             // Adresowanie linii
//             outputs.a.set_level(if (row & 0x01) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             outputs.b.set_level(if (row & 0x02) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             outputs.c.set_level(if (row & 0x04) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             outputs.d.set_level(if (row & 0x08) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             cortex_m::asm::delay(10);

//             let y_upper = row as u32;
//             let y_lower = row as u32 + 16;

//             for column in 0..64 {
//                 let x = column as u32;
//                 let mut r1 = false;
//                 let mut r2 = false;

//                 // Sprawdzamy górną połowę (wiersze 12..16 wejdą tutaj)
//                 if x >= text_start_x
//                     && x < text_end_x
//                     && y_upper >= text_start_y
//                     && y_upper < text_end_y
//                 {
//                     let local_x = x - text_start_x;
//                     let char_idx = local_x / 6;
//                     let pixel_x = local_x % 6;
//                     let pixel_y = y_upper - text_start_y;

//                     if pixel_x < 5 {
//                         let font_idx = get_font_for_char(char_idx);
//                         r1 = led::get_char_pixel(font_idx, pixel_x, pixel_y); // CZERWONY CZAS
//                     }
//                 }

//                 // Sprawdzamy dolną połowę (wiersze 16..20 wejdą tutaj)
//                 if x >= text_start_x
//                     && x < text_end_x
//                     && y_lower >= text_start_y
//                     && y_lower < text_end_y
//                 {
//                     let local_x = x - text_start_x;
//                     let char_idx = local_x / 6;
//                     let pixel_x = local_x % 6;
//                     let pixel_y = y_lower - text_start_y;

//                     if pixel_x < 5 {
//                         let font_idx = get_font_for_char(char_idx);
//                         r2 = led::get_char_pixel(font_idx, pixel_x, pixel_y); // CZERWONY CZAS
//                     }
//                 }

//                 // Wypchnięcie na piny
//                 outputs
//                     .r1
//                     .set_level(if r1 { Level::High } else { Level::Low });
//                 outputs.g1.set_level(Level::Low);
//                 outputs.b1.set_level(Level::Low);
//                 outputs
//                     .r2
//                     .set_level(if r2 { Level::High } else { Level::Low });
//                 outputs.g2.set_level(Level::Low);
//                 outputs.b2.set_level(Level::Low);

//                 outputs.clk.set_level(Level::High);
//                 outputs.clk.set_level(Level::Low);
//             }

//             // Zatrzask LAT
//             cortex_m::asm::delay(10);
//             outputs.lat.set_level(Level::High);
//             cortex_m::asm::delay(20);
//             outputs.lat.set_level(Level::Low);
//             cortex_m::asm::delay(20);

//             // Błysk OE
//             outputs.oe.set_level(Level::Low);
//             cortex_m::asm::delay(40);
//             outputs.oe.set_level(Level::High);

//             cortex_m::asm::delay(100);
//         }

//         // Bardzo krótkie uśpienie dla stabilizacji egzekutora
//         embassy_time::Timer::after_micros(100).await;
//     }
// }

// #[embassy_executor::task]
// async fn display() {
//     use defmt::info;

//     // Punkt startowy czasu na Pico
//     let started = embassy_time::Instant::now();

//     info!("Task wyświetlacza (Slave) uruchomiony. Oczekiwanie na dane...");

//     loop {
//         // ZAMIANA: .receive().await blokuje/usypia ten task do momentu,
//         // aż w kanale pojawi się nowy pakiet sieciowy. Zużycie procesora = 0%.
//         let update = shared::GAME_CHANNEL.receive().await;

//         // Próba odtworzenia stanu gry z payloadu pakietu
//         if let Ok(game) = game::Game::from_payload(&update.data) {
//             let total_seconds = game.millis / 1000;
//             let minutes = total_seconds / 60;
//             let seconds = total_seconds % 60;
//             let hundredths = (game.millis % 1000) / 10;

//             // Obliczamy ile czasu minęło na Pico od startu
//     let started = embassy_time::Instant::now();

//             let pico_elapsed_ms = (embassy_time::Instant::now() - started).as_millis();

//             // info!("=======================================");
//             // info!("        TABLICA WYNIKÓW - PICO         ");
//             // info!("  Typ rozgrywek: {:?}", game.discipline);
//             // info!("=======================================");
//             // info!("   GOSPODARZE               GOŚCIE    ");
//             // info!(
//             //     "      {:?}                      {:?}    ",
//             //     game.home_score, game.guest_score
//             // );
//             // info!("=======================================");
//             // info!("   KWARTA: {:?}", game.quarter);
//             // info!(
//             //     "   CZAS GRY: {:02}:{:02}.{:02}",
//             //     minutes, seconds, hundredths
//             // );
//             // info!("=======================================");
//             // info!(
//             //     " [Pakiet nr: {}, Czas Pico: {} ms]",
//             //     update.seq_id, pico_elapsed_ms
//             // );
//             // info!("=======================================");
//             info!("{}ms", &pico_elapsed_ms);
//         }
//     }
// }

// Struktura pomocnicza reprezentująca to, co aktualnie ma być renderowane na ekranie
#[derive(Clone, Copy)]
struct DisplayState {
    home_score: u16,
    away_score: u16,
    quarter: game::Quarter,
    minutes: u32,
    seconds: u32,
    hundredths: u32,
}

// Funkcja pomocnicza mapująca znak ASCII na indeks w Twojej tablicy czcionek `NUM_FONTS`
fn ascii_to_font_idx(c: char) -> usize {
    match c {
        '0'..='9' => (c as u8 - b'0') as usize,
        ':' => 10,
        ' ' => 11,
        '-' => 12,
        'H' => 13,
        'O' => 14,
        'M' => 15,
        'E' => 16,
        'A' => 17,
        'W' => 18,
        'Y' => 19,
        'Q' => 20,
        _ => 11, // Domyślnie spacja
    }
}

// #[embassy_executor::task]
// async fn task(peripherals: led::LedPeripherals) {
//     let mut outputs = led::LedOutputs::from(peripherals);

//     // Domniemany stan początkowy tablicy wyników
//     let mut state = DisplayState {
//         home_score: 0,
//         guest_score: 0,
//         quarter: game::Quarter::None,
//         minutes: 0,
//         seconds: 0,
//         hundredths: 0,
//     };

//     // Bufor tekstowy na 2 rzędy (wymiary matrycy 64x32)
//     // Szerokość jednego znaku to 5 pikseli + 1 piksel odstępu = 6 pikseli na znak.
//     // 64 / 6 = max ~10 znaków w rzędzie. Rozmieśćmy je odpowiednio.
//     // Rząd 1 (y: 0..8):  "HOME 000-000 AWAY" -> za długie na 64 piksele!
//     // Skróćmy do: "H 000-000 A" lub użyj samego formatu "000 - 000" z podpisem.
//     // Zgodnie z Twoim życzeniem zmieścimy: "HOME 00-00 AWAY" (14 znaków * 4px) lub ściśnięte 5px:
//     // "H 000-000 A" (11 znaków * 6px = 66px - też minimalnie za dużo).
//     // Zastosujmy formatowanie mieszczące się w 64 pikselach (np. 10 znaków dla czcionki 5px z odstępem 1px):
//     // Rząd 1: "H 000:000 A" -> 11 znaków. Zmniejszmy odstępy lub zróbmy: "000 - 000" na środku.
//     // Jeśli ma być dokładnie: "HOME 000 - 000 AWAY" - potrzebna byłaby czcionka 3x5.
//     // Zakładając czcionkę 5x8 (szerokość znaku 6 pikseli z odstępem):

//     let mut row1_str = heapless::String::<16>::new();
//     let mut row2_str = heapless::String::<16>::new();

//     let started = embassy_time::Instant::now();

//     loop {
//         // 1. Sprawdzamy nieblokująco (try_receive), czy przyszedł nowy pakiet z sieci
//         if let Ok(packet) = shared::GAME_CHANNEL.try_receive()
//             && let Ok(game) = game::Game::from_payload(&packet.data)
//         {
//             let time_ms = (embassy_time::Instant::now() - started).as_millis();
//             defmt::info!("seq: {}; {}ms", &packet.seq_id, time_ms);

//             state.minutes = game.millis / 1000 / 60;
//             state.seconds = (game.millis / 1000) % 60;
//             state.hundredths = (game.millis % 1000) / 10;
//             state.home_score = game.home_score;
//             state.guest_score = game.guest_score;
//             state.quarter = game.quarter;
//         }

//         // 2. Formatowanie napisów dla obu rzędów
//         row1_str.clear();
//         // Format zoptymalizowany pod szerokość 64 pikseli: "H:000-000:A" (11 znaków * 6px = 66px, ucinamy odstęp na końcu -> 65px)
//         // Bezpieczniej dla 64 pikseli (10 znaków): "000 -- 000" lub "H 00-00 A"
//         // Wpiszmy żądany tekst (w razie potrzeby zmień szerokość znaków/odstępy w logice poniżej):
//         let _ = core::fmt::write(
//             &mut row1_str,
//             format_args!("H{:03}-{:03}A", state.home_score, state.guest_score),
//         );

//         row2_str.clear();
//         let _ = core::fmt::write(
//             &mut row2_str,
//             format_args!(
//                 "Q{} {:02}:{:02}:{:02}",
//                 state.quarter as u8, state.minutes, state.seconds, state.hundredths
//             ),
//         );

//         // Wybór pozycji X startowej dla wyśrodkowania tekstów (64px szerokości)
//         // row1: np. "H000-000A" ma 9 znaków * 6px = 54px. Start X = (64 - 54) / 2 = 5
//         let r1_start_x: i32 = (64 - (row1_str.len() as i32 * 6)) / 2;
//         // row2: "Q1 00:00:00" ma 11 znaków * 6px = 66px (lekko wystaje, zmienimy odstęp na 0 dla ostatniego znaku)
//         let r2_start_x: i32 = 0;

//         // Definicja pozycji Y dla rzędów
//         let r1_start_y: u32 = 0; // Górny rząd (0-7)
//         let r2_start_y: u32 = 8; // Dolny rząd w górnej połówce (8-15)

//         // --- MULTIPLEKSOWANIE EKRANU HUB75 ---
//         for row in 0..16 {
//             outputs.oe.set_level(Level::High);
//             cortex_m::asm::delay(10);

//             // Adresowanie linii (A, B, C, D)
//             outputs.a.set_level(if (row & 0x01) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             outputs.b.set_level(if (row & 0x02) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             outputs.c.set_level(if (row & 0x04) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             outputs.d.set_level(if (row & 0x08) != 0 {
//                 Level::High
//             } else {
//                 Level::Low
//             });
//             cortex_m::asm::delay(10);

//             let y_upper = row as u32;

//             for column in 0..64 {
//                 let x = column;
//                 let mut r1 = false; // Kolor czerwony dla połówki górnej (wiersze 0..15)
//                 let r2 = false; // Kolor czerwony dla połówki dolnej (wiersze 16..31)

//                 // ================= GÓRNA POŁÓWKA (wiersze 0..15) =================
//                 // RZĄD 1: HOME / AWAY i WYNIK (y_upper w przedziale 0..8)
//                 if y_upper >= r1_start_y && y_upper < r1_start_y + 8 && x >= r1_start_x {
//                     let local_x = x - r1_start_x;
//                     let char_idx = (local_x / 6) as usize;
//                     let pixel_x = (local_x % 6) as u32;
//                     let pixel_y = y_upper - r1_start_y;

//                     if char_idx < row1_str.len() && pixel_x < 5 {
//                         let c = row1_str.as_bytes()[char_idx] as char;
//                         let font_idx = ascii_to_font_idx(c);
//                         r1 = led::get_char_pixel(font_idx, pixel_x, pixel_y);
//                     }
//                 }

//                 // RZĄD 2: KWARTA I CZAS GRY (y_upper w przedziale 8..16)
//                 if y_upper >= r2_start_y && y_upper < r2_start_y + 8 && x >= r2_start_x {
//                     let local_x = x - r2_start_x;
//                     let char_idx = (local_x / 6) as usize;
//                     let pixel_x = (local_x % 6) as u32;
//                     let pixel_y = y_upper - r2_start_y;

//                     if char_idx < row2_str.len() && pixel_x < 5 {
//                         let c = row2_str.as_bytes()[char_idx] as char;
//                         let font_idx = ascii_to_font_idx(c);
//                         r1 = led::get_char_pixel(font_idx, pixel_x, pixel_y);
//                     }
//                 }

//                 // ================= DOLNA POŁÓWKA (wiersze 16..31) =================
//                 // Jeśli chcesz zduplikować obraz lub wyświetlić coś na dolnych 16 wierszach,
//                 // modyfikujesz flagę `r2` korzystając z `y_lower`. Obecnie zostawiamy puste (false).

//                 // Wypchnięcie stanów na piny sterujące panelem
//                 outputs
//                     .r1
//                     .set_level(if r1 { Level::High } else { Level::Low });
//                 outputs.g1.set_level(Level::Low);
//                 outputs.b1.set_level(Level::Low);
//                 outputs
//                     .r2
//                     .set_level(if r2 { Level::High } else { Level::Low });
//                 outputs.g2.set_level(Level::Low);
//                 outputs.b2.set_level(Level::Low);

//                 // Taktowanie zegara CLK
//                 outputs.clk.set_level(Level::High);
//                 outputs.clk.set_level(Level::Low);
//             }

//             // Zatrzask danych (LAT) po przejściu całej linii 64 pikseli
//             cortex_m::asm::delay(10);
//             outputs.lat.set_level(Level::High);
//             cortex_m::asm::delay(20);
//             outputs.lat.set_level(Level::Low);
//             cortex_m::asm::delay(20);

//             // Wyzwolenie wyjścia (OE) - sterowanie jasnością/czasem świecenia wiersza
//             outputs.oe.set_level(Level::Low);
//             cortex_m::asm::delay(40);
//             outputs.oe.set_level(Level::High);

//             cortex_m::asm::delay(100);
//         }

//         // Bardzo krótkie uśpienie dla ustąpienia miejsca w egzekutorze Embassy
//         embassy_time::Timer::after_micros(50).await;
//     }
// }

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

    let mut home_str = heapless::String::<8>::new(); // np. "000"
    let mut guest_str = heapless::String::<8>::new(); // np. "000"
    let mut row2_str = heapless::String::<16>::new();

    let started = embassy_time::Instant::now();

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
