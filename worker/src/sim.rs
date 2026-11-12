//! Simulated dual-core architecture for Linux / desktop worker.
//!
//! Emulates:
//! - Core 0: UDP networking, Discovery, Handshake, Heartbeat, and OTA Wi-Fi configuration reception.
//! - Core 1: Scoreboard layout rendering and local clock interpolation at 60 Hz.
//! - Shared Game State between Core 0 and Core 1.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use basketball::{Game, Scorable};
use hub75::{Color, DisplayBuffer, RED};
use net::event::{ClientEvent, ServerEvent};
use net::layout::PACKET_SIZE;
use net::{ClientPacket, Device, DeviceConfigPacket, Heartbeat, ServerPacket};
use tokio::net::UdpSocket;
use tokio::time;

use crate::core1::ScoreboardLayout;

/// Shared Game State between simulated Core 0 (networking) and Core 1 (display runner).
#[derive(Clone)]
pub struct SimulatedGameState {
    game: Arc<RwLock<Game>>,
}

impl Default for SimulatedGameState {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulatedGameState {
    /// Creates a new empty [`SimulatedGameState`].
    pub fn new() -> Self {
        Self {
            game: Arc::new(RwLock::new(Game::empty())),
        }
    }

    /// Reads snapshot of the game.
    pub fn get(&self) -> Game {
        *self.game.read().expect("game state read lock")
    }

    /// Stores updated game.
    pub fn set(&self, game: Game) {
        *self.game.write().expect("game state write lock") = game;
    }

    /// Updates clock tick.
    pub fn tick(&self, delta: Duration) {
        let mut guard = self.game.write().expect("game state write lock");
        if guard.state() == basketball::State::Running {
            guard.tick(delta);
        }
    }
}

/// Linux worker simulator replicating the dual-core RP235x firmware architecture.
pub struct LinuxWorkerSimulator {
    device: Device,
    shared_state: SimulatedGameState,
    running: Arc<AtomicBool>,
}

impl LinuxWorkerSimulator {
    /// Creates a new [`LinuxWorkerSimulator`] with identity `device`.
    pub fn new(device: Device) -> Self {
        Self {
            device,
            shared_state: SimulatedGameState::new(),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Runs the virtualized dual-core worker.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let shared_for_core0 = self.shared_state.clone();
        let shared_for_core1 = self.shared_state.clone();
        let running_core0 = self.running.clone();
        let running_core1 = self.running.clone();
        let device = self.device;

        // Core 1: Display refresh & local clock interpolation loop (running concurrently)
        let core1_handle = tokio::spawn(async move {
            let mut display = DisplayBuffer::<64, 32, 16>::default();
            display.set_brightness(20);
            let mut layout = ScoreboardLayout::new();
            let mut render_interval = time::interval(Duration::from_millis(16)); // ~60 Hz like HUB75
            let mut last_tick = tokio::time::Instant::now();
            let mut last_rendered_second: u32 = 999999;
            let mut last_scores = (0u16, 0u16);
            let mut last_state = basketball::State::Idle;

            while running_core1.load(Ordering::Relaxed) {
                render_interval.tick().await;

                let now = tokio::time::Instant::now();
                let delta = now.duration_since(last_tick);
                last_tick = now;

                // High-resolution clock interpolation on Core 1
                shared_for_core1.tick(delta);

                let game = shared_for_core1.get();

                // Render through the firmware HUB75 layout methods
                let color: Color = RED;
                layout.render(&mut display, &game, color, true);

                // Print terminal update whenever seconds, scores, or match state change
                let current_second = game.regulation_millis() / 1000;
                let scores = (
                    game.score(basketball::Team::Home),
                    game.score(basketball::Team::Away),
                );
                let state = game.state();

                if current_second != last_rendered_second
                    || scores != last_scores
                    || state != last_state
                {
                    last_rendered_second = current_second;
                    last_scores = scores;
                    last_state = state;
                    print_terminal_scoreboard(&game);
                }
            }
        });

        // Core 0: UDP networking, master discovery, OTA config packet handling
        let core0_handle = tokio::spawn(async move {
            if let Err(e) = run_core0_networking(device, shared_for_core0, running_core0).await {
                eprintln!("❌ Core 0 Networking error: {e}");
            }
        });

        tokio::select! {
            _ = core1_handle => {},
            _ = core0_handle => {},
        }

        Ok(())
    }
}

async fn run_core0_networking(
    device: Device,
    shared_state: SimulatedGameState,
    running: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("0.0.0.0:8001").await?;
    socket.set_broadcast(true)?;
    println!("📡 [Core 0] UDP Worker socket bound to 0.0.0.0:8001");

    let device_bytes = device.to_bytes();
    let broadcast_target: SocketAddr = "255.255.255.255:9000".parse()?;

    let mut sequence_id: u16 = 0;
    let mut receive_buffer = [0u8; 1024];

    println!("🔍 [Core 0] Searching for Master server on 255.255.255.255:9000...");

    let mut master_address: Option<SocketAddr> = None;
    let mut heartbeat_interval = time::interval(Duration::from_millis(500));
    let mut session_token: u32 = 0;

    while running.load(Ordering::Relaxed) {
        tokio::select! {
            _ = heartbeat_interval.tick() => {
                sequence_id = sequence_id.wrapping_add(1);
                let target = master_address.unwrap_or(broadcast_target);
                let packet = if master_address.is_some() && session_token != 0 {
                    let heartbeat_bytes = Heartbeat::new(0).to_bytes();
                    ClientPacket::with_session(
                        ClientEvent::HandshakeHeartbeat,
                        sequence_id,
                        session_token,
                        Heartbeat::PAYLOAD_LEN,
                        heartbeat_bytes,
                    )
                } else {
                    ClientPacket::new(ClientEvent::HandshakeRequest, sequence_id, device_bytes)
                };
                let _ = socket.send_to(&packet.to_bytes(), target).await;
            }

            recv_res = socket.recv_from(&mut receive_buffer) => {
                let (len, remote_addr) = match recv_res {
                    Ok(res) => res,
                    Err(e) => {
                        eprintln!("⚠️ [Core 0] UDP receive error: {e}");
                        continue;
                    }
                };

                // Check for OTA configuration packet (same as RP235x firmware)
                if let Ok(config_pkt) = DeviceConfigPacket::from_bytes(&receive_buffer[..len]) {
                    println!("⚙️ [Core 0] Received DeviceConfigPacket from {remote_addr}");
                    let ack_packet = ServerPacket::new(ServerEvent::DeviceConfigAck, 0, [0u8; net::layout::DATA_SIZE]);
                    let _ = socket.send_to(&ack_packet.to_bytes(), remote_addr).await;

                    if let (Ok(sn), Ok(id), Ok(ssid)) = (config_pkt.serial_number(), config_pkt.device_id(), config_pkt.wifi_ssid()) {
                        println!("📋 [Core 0] Applied new config: SN='{sn}', ID='{id}', SSID='{ssid}'");
                        let success_packet = ServerPacket::new(ServerEvent::DeviceConfigSuccess, 0, [0u8; net::layout::DATA_SIZE]);
                        let _ = socket.send_to(&success_packet.to_bytes(), remote_addr).await;
                    }
                    continue;
                }

                if len < PACKET_SIZE {
                    continue;
                }

                let mut packet_data = [0u8; PACKET_SIZE];
                packet_data.copy_from_slice(&receive_buffer[..PACKET_SIZE]);

                let server_packet = match ServerPacket::from_bytes(packet_data) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                if master_address.is_none() || master_address != Some(remote_addr) {
                    println!(" Connected to Master server at {remote_addr}");
                    master_address = Some(remote_addr);
                }

                let packet_token = server_packet.session_token();
                if session_token != 0 && packet_token != session_token {
                    continue;
                }

                match server_packet.event() {
                    ServerEvent::HeartbeatAck => {
                        // Heartbeat ack acknowledged
                    }
                    ServerEvent::HandshakeAck => {
                        if packet_token != 0 {
                            session_token = packet_token;
                        }
                    }
                    ServerEvent::GameUpdate => {
                        if let Ok(game) = Game::try_from(server_packet) {
                            shared_state.set(game);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn print_terminal_scoreboard(game: &Game) {
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
        basketball::State::Idle | _ => "IDLE",
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
