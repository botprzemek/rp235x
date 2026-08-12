use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};

use game::{Discipline, Game};
use game_net::{Command, PACKET_SIZE, Packet};

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:12345").expect("Nie udało się powiązać gniazda UDP");

    socket
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();

    let mut game = Game::new(Discipline::FIBA5V5);
    let pico_endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 104)), 12346);

    let mut last_tick_time: i64 = 0;
    let mut sequence_id: u8 = 0;
    let start_instant = SystemTime::now();
    let mut packet_buffer = [0u8; 2048];

    const TICK_RATE_MS: i64 = 16;
    let mut last_random_score_time: i64 = 0;
    const RANDOM_SCORE_INTERVAL_MS: i64 = 3000;

    loop {
        let current_time_ms = start_instant.elapsed().unwrap().as_millis() as i64;

        if current_time_ms >= last_tick_time + TICK_RATE_MS {
            last_tick_time += TICK_RATE_MS;

            game.tick(TICK_RATE_MS as u32);
            sequence_id = sequence_id.wrapping_add(1);

            if current_time_ms >= last_random_score_time + RANDOM_SCORE_INTERVAL_MS {
                last_random_score_time = current_time_ms;

                let random_bit = (current_time_ms + sequence_id as i64) % 10;

                if random_bit <= 3 {
                    let points = if random_bit % 2 == 0 { 2 } else { 3 };
                    game.home_score += points;
                    println!("[LOSOWO] Gospodarze zdobyli +{} pkt!", points);
                } else if random_bit <= 7 {
                    let points = if random_bit % 2 == 0 { 2 } else { 3 };
                    game.away_score += points;
                    println!("[LOSOWO] Gościom przyznano +{} pkt!", points);
                }
            }

            render_terminal_scoreboard(&game, sequence_id, current_time_ms);

            let state_packet = Packet::new(Command::UpdateScore, sequence_id, &game);
            let _ = socket.send_to(&state_packet.to_bytes(), pico_endpoint);
        }

        match socket.recv_from(&mut packet_buffer) {
            Ok((amt, src_addr)) => {
                if amt < PACKET_SIZE {
                    continue;
                }

                let mut fixed_buffer = [0u8; PACKET_SIZE];
                fixed_buffer.copy_from_slice(&packet_buffer[0..PACKET_SIZE]);

                if let Ok(incoming_packet) = Packet::from_bytes(&fixed_buffer)
                    && let Command::Ping = incoming_packet.command
                {
                    let response = Packet::new(Command::Ping, incoming_packet.seq_id, &game);
                    let _ = socket.send_to(&response.to_bytes(), src_addr);
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => {}
        }
    }
}

fn render_terminal_scoreboard(game: &Game, seq_id: u8, time_ms: i64) {
    std::println!("seq: {}; {}ms", seq_id, time_ms);
}
