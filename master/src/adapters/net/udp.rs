//! UDP server driver for communicating with RP235x hardware workers.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::services::{Observable, Services};

use basketball::{Discipline, Game};
use config::DeviceConfig;
use net::layout::PACKET_SIZE;
use net::{ClientEvent, ClientPacket, Device, ServerEvent, ServerPacket};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, watch};
use tokio::time::{self, MissedTickBehavior};

const SYNC_HEARTBEAT_DURATION: Duration = Duration::from_millis(100); // 10 Hz periodic sync (event-driven broadcast handles immediate updates)
const OUTBOUND_QUEUE_CAPACITY: usize = 4096;

#[derive(Clone)]
struct ActiveWorker {
    address: SocketAddr,
    serial_number: String,
    session_token: u32,
}

struct OutboundAck {
    packet_bytes: [u8; PACKET_SIZE],
    target: SocketAddr,
}

/// UDP transport driver streaming periodic and event-driven game updates to RP235x workers.
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
        let (worker_tx, worker_rx) = watch::channel(HashMap::<String, ActiveWorker>::new());
        let (outbound_tx, outbound_rx) = mpsc::channel::<OutboundAck>(OUTBOUND_QUEUE_CAPACITY);

        // Independent transmit task: drains responses without blocking the receive loop
        let transmit_socket = Arc::clone(&self.listener);
        tokio::spawn(async move {
            Self::handle_outbound_queue(transmit_socket, outbound_rx).await;
        });

        // Dedicated receive task: non-blocking packet ingestion into processing pipeline
        let receive_socket = Arc::clone(&self.listener);
        let receive_services = Arc::clone(&services);
        tokio::spawn(async move {
            Self::handle_incoming(receive_socket, receive_services, worker_tx, outbound_tx).await;
        });

        // Periodic tick broadcast task
        let tick_socket = Arc::clone(&self.listener);
        let tick_services = Arc::clone(&services);
        tokio::spawn(async move {
            Self::handle_ticks(tick_socket, tick_services, worker_rx).await;
        });
    }

    async fn handle_outbound_queue(
        socket: Arc<UdpSocket>,
        mut outbound_rx: mpsc::Receiver<OutboundAck>,
    ) {
        while let Some(outbound) = outbound_rx.recv().await {
            let _ = socket
                .send_to(&outbound.packet_bytes, outbound.target)
                .await;
        }
    }

    async fn handle_incoming(
        socket: Arc<UdpSocket>,
        services: Arc<Services>,
        worker_tx: watch::Sender<HashMap<String, ActiveWorker>>,
        outbound_tx: mpsc::Sender<OutboundAck>,
    ) {
        let mut sequence_id: u8 = 0;
        let mut receive_buffer = [0u8; PACKET_SIZE];
        let mut disconnect_rx = services.devices().subscribe_disconnect();

        loop {
            tokio::select! {
                biased;

                // Priority 1: Ingest and process UDP packet immediately
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

                        let assigned_game_id = match device.serial_number() {
                            Ok(serial) => services.devices().game_id_by_serial(serial).unwrap_or(0),
                            Err(_) => continue,
                        };
                        device.set_game_id(assigned_game_id);

                        let Ok(serial_ref) = device.serial_number() else {
                            continue;
                        };

                        if let Err(error) = services.devices().validate_and_register(device) {
                            eprintln!(
                                "WARN: [UDP {remote_address}] Worker device validation failed: {error}"
                            );
                            continue;
                        }

                        // Check if this worker is already known with the same address
                        let (session_token, is_new_or_moved) = {
                            let workers_guard = worker_tx.borrow();
                            match workers_guard.get(serial_ref) {
                                Some(existing) => (existing.session_token, existing.address != remote_address),
                                None => {
                                    let new_token = (SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_nanos() as u32)
                                        | 0x8000_0000;
                                    (new_token, true)
                                }
                            }
                        };

                        if is_new_or_moved {
                            println!(
                                "INFO: [UDP {remote_address}] Connected worker {:?} (id={}, sn={serial_ref}, game_id={})",
                                device.device_type(),
                                device.device_id().unwrap_or("unknown"),
                                device.game_id(),
                            );

                            let serial_string = serial_ref.to_string();
                            worker_tx.send_modify(|workers| {
                                workers.insert(
                                    serial_string.clone(),
                                    ActiveWorker {
                                        address: remote_address,
                                        serial_number: serial_string,
                                        session_token,
                                    },
                                );
                            });
                        }

                        sequence_id = sequence_id.wrapping_add(1);

                        let mut current_game = if assigned_game_id != 0 {
                            services.game().by_id(assigned_game_id)
                        } else {
                            None
                        }
                        .unwrap_or_else(|| Game::new_with_id(0, Discipline::Fiba5v5));
                        current_game.set_session_token(session_token);

                        let ack_packet = ServerPacket::new(
                            ServerEvent::HandshakeAck,
                            sequence_id,
                            current_game.to_bytes(),
                        );

                        // Asynchronous, non-blocking enqueue of the ACK packet
                        let _ = outbound_tx.try_send(OutboundAck {
                            packet_bytes: ack_packet.to_bytes(),
                            target: remote_address,
                        });
                    }
                }

                // Priority 2: Disconnection events
                Ok(target_serial_number) = disconnect_rx.recv() => {
                    let mut removed = false;
                    worker_tx.send_modify(|workers| {
                        if workers.remove(&target_serial_number).is_some() {
                            removed = true;
                        }
                    });
                    if removed {
                        println!("INFO: [UDP] Forcefully disconnected worker sn={target_serial_number}");
                    }
                }
            }
        }
    }

    async fn handle_ticks(
        socket: Arc<UdpSocket>,
        services: Arc<Services>,
        worker_rx: watch::Receiver<HashMap<String, ActiveWorker>>,
    ) {
        let mut interval = time::interval(SYNC_HEARTBEAT_DURATION);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut sequence_id: u8 = 0;
        let mut broadcast_rx = services.game().subscribe();
        let mut game_assign_rx = services.devices().subscribe_game_assign();

        loop {
            let active_workers = worker_rx.borrow().clone();

            tokio::select! {
                _ = interval.tick() => {
                    for worker in active_workers.values() {
                        let assigned_game_id = services
                            .devices()
                            .game_id_by_serial(&worker.serial_number)
                            .unwrap_or(0);
                        if assigned_game_id != 0
                            && let Some(mut current_game) = services.game().by_id(assigned_game_id)
                        {
                            current_game.set_session_token(worker.session_token);
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
                    if let Some(worker) = active_workers.get(&assignment.serial_number) {
                        let mut game = if assignment.game_id != 0 {
                            services.game().by_id(assignment.game_id)
                        } else {
                            None
                        }
                        .unwrap_or_else(|| Game::new_with_id(0, Discipline::Fiba5v5));
                        game.set_session_token(worker.session_token);

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
                    if let Ok(games) = result {
                        for worker in active_workers.values() {
                            let assigned_game_id = services
                                .devices()
                                .game_id_by_serial(&worker.serial_number)
                                .unwrap_or(0);
                            if assigned_game_id != 0
                                && let Some(mut game) = games.iter().find(|entry| entry.id() == assigned_game_id).copied()
                            {
                                game.set_session_token(worker.session_token);
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
}
