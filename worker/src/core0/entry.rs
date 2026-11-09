//! Core 0 executor and state machine entry task.

#[cfg(feature = "rp235x")]
use crate::peripherals::{NetPeripherals, TrngPeripherals};
#[cfg(feature = "rp235x")]
use crate::state::Machine;

#[cfg(feature = "rp235x")]
use embassy_executor::{Executor, Spawner};
#[cfg(feature = "rp235x")]
use embassy_rp::Peri;
#[cfg(feature = "rp235x")]
use embassy_rp::peripherals::{FLASH, WATCHDOG};
#[cfg(feature = "rp235x")]
use static_cell::StaticCell;

#[cfg(not(feature = "rp235x"))]
use crate::state::{GAME_STATE, Machine};
#[cfg(not(feature = "rp235x"))]
use basketball::Game;
#[cfg(not(feature = "rp235x"))]
use net::event::{ClientEvent, ServerEvent};
#[cfg(not(feature = "rp235x"))]
use net::layout::PACKET_SIZE;
#[cfg(not(feature = "rp235x"))]
use net::{ClientPacket, Device, DeviceConfigPacket, DeviceType, ServerPacket};
#[cfg(not(feature = "rp235x"))]
use std::net::SocketAddr;
#[cfg(not(feature = "rp235x"))]
use std::time::Duration;
#[cfg(not(feature = "rp235x"))]
use tokio::net::UdpSocket;
#[cfg(not(feature = "rp235x"))]
use tokio::time;

/// Core 0 entry point.
pub struct Core0;

/// Static async executor for Core 0 on RP235x.
#[cfg(feature = "rp235x")]
pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[cfg(feature = "rp235x")]
impl Core0 {
    /// Initializes the Core 0 executor and spawns the main state machine task.
    pub fn entry(
        net: NetPeripherals,
        trng: TrngPeripherals,
        watchdog: Peri<'static, WATCHDOG>,
        flash: Peri<'static, FLASH>,
    ) -> ! {
        let executor = Executor::new();

        EXECUTOR.init(executor).run(|spawner| {
            spawner.spawn(defmt::unwrap!(task(spawner, net, trng, watchdog, flash)));
        })
    }
}

#[cfg(feature = "rp235x")]
#[embassy_executor::task]
async fn task(
    spawner: Spawner,
    net: NetPeripherals,
    trng: TrngPeripherals,
    watchdog: Peri<'static, WATCHDOG>,
    flash: Peri<'static, FLASH>,
) {
    let mut state_machine = Machine::new();

    state_machine.handle_boot().await;
    state_machine.handle_sync().await;
    state_machine
        .handle_networking(spawner, net, trng, watchdog, flash)
        .await;
}

#[cfg(not(feature = "rp235x"))]
impl Core0 {
    /// Linux entry point for Core 0 (state machine & UDP networking thread).
    pub fn entry() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime for Core 0");

        rt.block_on(async {
            Self::task().await;
        });
    }

    async fn task() {
        let mut state_machine = Machine::new();

        state_machine.handle_boot().await;
        state_machine.handle_sync().await;
        state_machine.handle_networking().await;
    }
}

#[cfg(not(feature = "rp235x"))]
pub async fn run_udp_worker() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind("0.0.0.0:8001").await?;
    socket.set_broadcast(true)?;
    println!("📡 [Core 0 Thread] UDP Worker socket bound to 0.0.0.0:8001");

    let device = Device::try_new(DeviceType::Worker, "sn-linux-001", "worker-linux")?;
    let device_bytes = device.to_bytes();
    let broadcast_target: SocketAddr = "255.255.255.255:9000".parse()?;

    let mut sequence_id: u8 = 0;
    let mut receive_buffer = [0u8; 1024];

    println!("🔍 [Core 0 Thread] Searching for Master server on 255.255.255.255:9000...");

    let mut master_address: Option<SocketAddr> = None;
    let mut heartbeat_interval = time::interval(Duration::from_millis(500));
    let mut session_token: u32 = 0;

    loop {
        tokio::select! {
            _ = heartbeat_interval.tick() => {
                sequence_id = sequence_id.wrapping_add(1);
                let target = master_address.unwrap_or(broadcast_target);
                let packet = ClientPacket::new(ClientEvent::HandshakeHeartbeat, sequence_id, device_bytes);
                let _ = socket.send_to(&packet.to_bytes(), target).await;
            }

            recv_res = socket.recv_from(&mut receive_buffer) => {
                let (len, remote_addr) = match recv_res {
                    Ok(res) => res,
                    Err(e) => {
                        eprintln!("⚠️ [Core 0 Thread] UDP receive error: {e}");
                        continue;
                    }
                };

                // Check for OTA configuration packet
                if let Ok(config_pkt) = DeviceConfigPacket::from_bytes(&receive_buffer[..len]) {
                    println!("⚙️ [Core 0 Thread] Received DeviceConfigPacket from {remote_addr}");
                    let ack_packet = ServerPacket::new(ServerEvent::DeviceConfigAck, 0, [0u8; net::layout::DATA_SIZE]);
                    let _ = socket.send_to(&ack_packet.to_bytes(), remote_addr).await;

                    if let (Ok(sn), Ok(id), Ok(ssid)) = (config_pkt.serial_number(), config_pkt.device_id(), config_pkt.wifi_ssid()) {
                        println!("📋 [Core 0 Thread] Applied new config: SN='{sn}', ID='{id}', SSID='{ssid}'");
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
                    println!("🎉 [Core 0 Thread] Connected to Master server at {remote_addr}");
                    master_address = Some(remote_addr);
                }

                match server_packet.event() {
                    ServerEvent::HandshakeAck => {
                        if let Ok(game) = Game::try_from(server_packet) {
                            session_token = game.session_token();
                            GAME_STATE.set(game);
                        }
                    }
                    ServerEvent::GameUpdate => {
                        if let Ok(game) = Game::try_from(server_packet) {
                            if session_token == 0 || game.session_token() == 0 || game.session_token() == session_token {
                                GAME_STATE.set(game);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
