//! UDP server driver for communicating with RP235x hardware workers.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use basketball::{Discipline, Game};
use config::DeviceConfig;
use net::layout::PACKET_SIZE;
use net::{ClientEvent, ClientPacket, Device, ServerEvent, ServerPacket};
use tokio::net::UdpSocket;
use tokio::sync::watch;
use tokio::time::{self, MissedTickBehavior};

use crate::services::{Observable, Services};

const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60); // 60 Hz (60 updates/s)

#[derive(Clone)]
struct ActiveWorker {
    address: SocketAddr,
    serial_number: String,
}

/// UDP transport driver streaming 60 Hz game updates to RP235x workers.
pub struct WorkerDriver {
    listener: Arc<UdpSocket>,
}

impl WorkerDriver {
    /// Creates a new [`WorkerDriver`] wrapping `listener`.
    #[must_use]
    pub fn new(listener: UdpSocket) -> Self {
        Self {
            listener: Arc::new(listener),
        }
    }

    /// Spawns the UDP worker driver tasks onto the Tokio runtime.
    pub fn spawn(self, services: Arc<Services>) {
        tokio::spawn(async move {
            Self::run(self, services).await;
        });
    }

    async fn run(self, services: Arc<Services>) {
        let (worker_tx, worker_rx) = watch::channel(None);

        let receive_socket = Arc::clone(&self.listener);
        let receive_services = Arc::clone(&services);
        tokio::spawn(async move {
            Self::handle_incoming(receive_socket, receive_services, worker_tx).await;
        });

        let transmit_socket = Arc::clone(&self.listener);
        let transmit_services = Arc::clone(&services);
        tokio::spawn(async move {
            Self::handle_ticks(transmit_socket, transmit_services, worker_rx).await;
        });
    }

    async fn handle_incoming(
        socket: Arc<UdpSocket>,
        services: Arc<Services>,
        worker_tx: watch::Sender<Option<ActiveWorker>>,
    ) {
        let mut sequence_id: u8 = 0;
        let mut receive_buffer = [0u8; PACKET_SIZE];
        let mut disconnect_rx = services.devices().subscribe_disconnect();

        loop {
            tokio::select! {
                Ok(target_serial_number) = disconnect_rx.recv() => {
                    let should_clear = worker_tx
                        .borrow()
                        .as_ref()
                        .is_some_and(|worker| worker.serial_number == target_serial_number);
                    if should_clear {
                        println!("INFO: [UDP] Forcefully disconnected worker sn={target_serial_number}");
                        let _ = worker_tx.send(None);
                    }
                }

                receive_result = socket.recv_from(&mut receive_buffer) => {
                    let (size, remote_address) = match receive_result {
                        Ok(result) => result,
                        Err(_) => continue,
                    };

                    if size != PACKET_SIZE {
                        continue;
                    }

                    let Ok(packet_array) = receive_buffer[..PACKET_SIZE].try_into() else {
                        continue;
                    };

                    let Ok(packet) = ClientPacket::from_bytes(packet_array) else {
                        continue;
                    };

                    if let ClientEvent::HandshakeRequest | ClientEvent::HandshakeHeartbeat = packet.event() {
                        let mut device = match Device::try_from(packet) {
                            Ok(device) => device,
                            Err(error) => {
                                eprintln!(
                                    "WARN: [UDP {remote_address}] Invalid worker handshake payload: {error}"
                                );
                                continue;
                            }
                        };

                        let serial_number = device.serial_number().unwrap_or("unknown").to_string();
                        let assigned_game_id = services
                            .devices()
                            .game_id_by_serial(&serial_number)
                            .unwrap_or(0);
                        device.set_game_id(assigned_game_id);

                        if let Err(error) = services.devices().validate_and_register(device) {
                            eprintln!(
                                "WARN: [UDP {remote_address}] Worker device validation failed: {error}"
                            );
                            continue;
                        }

                        println!(
                            "INFO: [UDP {remote_address}] Validated device {:?} (id={}, sn={serial_number}, game_id={})",
                            device.device_type(),
                            device.device_id().unwrap_or("unknown"),
                            device.game_id(),
                        );

                        let _ = worker_tx.send(Some(ActiveWorker {
                            address: remote_address,
                            serial_number,
                        }));

                        sequence_id = sequence_id.wrapping_add(1);

                        let current_game = if assigned_game_id != 0 {
                            services.game().by_id(assigned_game_id)
                        } else {
                            None
                        }
                        .unwrap_or_else(|| Game::new_with_id(0, Discipline::Fiba5vs5));

                        let ack_packet = ServerPacket::new(
                            ServerEvent::HandshakeAck,
                            sequence_id,
                            current_game.to_bytes(),
                        );
                        let _ = socket.send_to(&ack_packet.to_bytes(), remote_address).await;
                    }
                }
            }
        }
    }

    async fn handle_ticks(
        socket: Arc<UdpSocket>,
        services: Arc<Services>,
        worker_rx: watch::Receiver<Option<ActiveWorker>>,
    ) {
        let mut interval = time::interval(FRAME_DURATION);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut sequence_id: u8 = 0;
        let mut broadcast_rx = services.game().subscribe();
        let mut game_assign_rx = services.devices().subscribe_game_assign();

        loop {
            let active_worker = worker_rx.borrow().clone();

            tokio::select! {
                _ = interval.tick() => {
                    if let Some(ref worker) = active_worker {
                        let assigned_game_id = services
                            .devices()
                            .game_id_by_serial(&worker.serial_number)
                            .unwrap_or(0);
                        if assigned_game_id != 0
                            && let Some(current_game) = services.game().by_id(assigned_game_id)
                        {
                            sequence_id = sequence_id.wrapping_add(1);
                            let packet = ServerPacket::new(
                                ServerEvent::GameUpdate,
                                sequence_id,
                                current_game.to_bytes(),
                            );
                            let _ = socket.send_to(&packet.to_bytes(), worker.address).await;
                        }
                    }
                }

                Ok(assignment) = game_assign_rx.recv() => {
                    if let Some(ref worker) = active_worker
                        && worker.serial_number == assignment.serial_number
                    {
                        let game = if assignment.game_id != 0 {
                            services.game().by_id(assignment.game_id)
                        } else {
                            None
                        }
                        .unwrap_or_else(|| Game::new_with_id(0, Discipline::Fiba5vs5));

                        sequence_id = sequence_id.wrapping_add(1);
                        let packet = ServerPacket::new(
                            ServerEvent::GameUpdate,
                            sequence_id,
                            game.to_bytes(),
                        );
                        let _ = socket.send_to(&packet.to_bytes(), worker.address).await;
                    }
                }

                result = broadcast_rx.recv() => {
                    if let Ok(games) = result
                        && let Some(ref worker) = active_worker
                    {
                        let assigned_game_id = services
                            .devices()
                            .game_id_by_serial(&worker.serial_number)
                            .unwrap_or(0);
                        if assigned_game_id != 0
                            && let Some(game) = games.iter().find(|entry| entry.id() == assigned_game_id)
                        {
                            sequence_id = sequence_id.wrapping_add(1);
                            let packet = ServerPacket::new(
                                ServerEvent::GameUpdate,
                                sequence_id,
                                game.to_bytes(),
                            );
                            let _ = socket.send_to(&packet.to_bytes(), worker.address).await;
                        }
                    }
                }
            }
        }
    }
}
