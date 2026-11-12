use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use crate::services::{Observable, Services};

use basketball::{Discipline, Game};
use config::DeviceConfig;
use net::layout::{DATA_SIZE, PACKET_SIZE};
use net::{
    ClientEvent, ClientPacket, Device, HandshakeAck, HeartbeatAck, ServerEvent, ServerPacket,
};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, watch};
use tokio::time::{self, MissedTickBehavior};

const SYNC_HEARTBEAT_DURATION: Duration = Duration::from_millis(100);
const WORKER_TIMEOUT_DURATION: Duration = Duration::from_secs(9);
const OUTBOUND_QUEUE_CAPACITY: usize = 4096;

#[derive(Clone)]
struct ActiveWorker {
    address: SocketAddr,
    serial_number: String,
    session_token: u32,
    last_seen: Instant,
}

struct OutboundAck {
    packet_bytes: [u8; PACKET_SIZE],
    target: SocketAddr,
}

pub struct WorkerDriver {
    listener: Arc<UdpSocket>,
}

impl WorkerDriver {

    #[must_use]
    pub fn new(listener: UdpSocket) -> Self {
        Self {
            listener: Arc::new(listener),
        }
    }

    pub fn spawn(self, services: Arc<Services>) {
        tokio::spawn(async move {
            Self::run(self, services).await;
        });
    }

    async fn run(self, services: Arc<Services>) {
        let (worker_tx, worker_rx) = watch::channel(HashMap::<String, ActiveWorker>::new());
        let (outbound_tx, outbound_rx) = mpsc::channel::<OutboundAck>(OUTBOUND_QUEUE_CAPACITY);

        let transmit_socket = Arc::clone(&self.listener);
        tokio::spawn(async move {
            Self::handle_outbound_queue(transmit_socket, outbound_rx).await;
        });

        let receive_socket = Arc::clone(&self.listener);
        let receive_services = Arc::clone(&services);
        let worker_tx_incoming = worker_tx.clone();
        tokio::spawn(async move {
            Self::handle_incoming(
                receive_socket,
                receive_services,
                worker_tx_incoming,
                outbound_tx,
            )
            .await;
        });

        let timeout_services = Arc::clone(&services);
        let worker_tx_timeout = worker_tx.clone();
        tokio::spawn(async move {
            let mut check_interval = time::interval(Duration::from_secs(1));
            loop {
                check_interval.tick().await;
                let now = Instant::now();
                let mut expired_serials = Vec::new();

                worker_tx_timeout.send_modify(|workers| {
                    workers.retain(|sn, worker| {
                        if now.duration_since(worker.last_seen) > WORKER_TIMEOUT_DURATION {
                            expired_serials.push(sn.clone());
                            false
                        } else {
                            true
                        }
                    });
                });

                for sn in expired_serials {
                    println!(
                        "WARN: [UDP] Worker sn={sn} timed out (no heartbeat for 9s). Marked as offline."
                    );
                    timeout_services.devices().disconnect_by_serial(&sn);
                }
            }
        });

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
        let mut sequence_id: u16 = 0;
        let mut receive_buffer = [0u8; PACKET_SIZE];
        let mut disconnect_rx = services.devices().subscribe_disconnect();

        loop {
            tokio::select! {
                biased;

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

                    match packet.event() {
                        ClientEvent::HandshakeHeartbeat => {
                            let packet_token = packet.session_token();
                            if packet_token == 0 {
                                continue;
                            }

                            let now = Instant::now();
                            let mut matched_worker: Option<ActiveWorker> = None;

                            worker_tx.send_modify(|workers| {
                                for worker in workers.values_mut() {
                                    if worker.address == remote_address && worker.session_token == packet_token {
                                        worker.last_seen = now;
                                        matched_worker = Some(worker.clone());
                                        break;
                                    }
                                }
                            });

                            if let Some(worker) = matched_worker {
                                sequence_id = sequence_id.wrapping_add(1);
                                let ack_payload = HeartbeatAck::new(sequence_id, 0);

                                let ack_packet = ServerPacket::with_session(
                                    ServerEvent::HeartbeatAck,
                                    sequence_id,
                                    worker.session_token,
                                    HeartbeatAck::PAYLOAD_LEN,
                                    ack_payload.to_bytes(),
                                );

                                let _ = outbound_tx.try_send(OutboundAck {
                                    packet_bytes: ack_packet.to_bytes(),
                                    target: remote_address,
                                });
                            }
                        }
                        ClientEvent::HandshakeRequest => {
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

                            let now = Instant::now();

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

                            let serial_string = serial_ref.to_string();
                            worker_tx.send_modify(|workers| {
                                workers.insert(
                                    serial_string.clone(),
                                    ActiveWorker {
                                        address: remote_address,
                                        serial_number: serial_string,
                                        session_token,
                                        last_seen: now,
                                    },
                                );
                            });

                            if is_new_or_moved {
                                println!(
                                    "INFO: [UDP {remote_address}] Connected worker {:?} (id={}, sn={serial_ref}, game_id={})",
                                    device.device_type(),
                                    device.device_id().unwrap_or("unknown"),
                                    device.game_id(),
                                );
                            }

                            sequence_id = sequence_id.wrapping_add(1);

                            let handshake_ack = HandshakeAck::new(assigned_game_id, 0);

                            let ack_packet = ServerPacket::with_session(
                                ServerEvent::HandshakeAck,
                                sequence_id,
                                session_token,
                                HandshakeAck::PAYLOAD_LEN,
                                handshake_ack.to_bytes(),
                            );

                            let _ = outbound_tx.try_send(OutboundAck {
                                packet_bytes: ack_packet.to_bytes(),
                                target: remote_address,
                            });

                            if assigned_game_id != 0
                                && let Some(game) = services.game().by_id(assigned_game_id)
                            {
                                sequence_id = sequence_id.wrapping_add(1);
                                let game_packet = ServerPacket::with_session(
                                    ServerEvent::GameUpdate,
                                    sequence_id,
                                    session_token,
                                    DATA_SIZE as u8,
                                    game.to_bytes(),
                                );
                                let _ = outbound_tx.try_send(OutboundAck {
                                    packet_bytes: game_packet.to_bytes(),
                                    target: remote_address,
                                });
                            }
                        }
                        _ => {}
                    }
                }

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
        let mut sequence_id: u16 = 0;
        let mut last_sent_games: HashMap<u16, Game> = HashMap::new();
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
                            && let Some(current_game) = services.game().by_id(assigned_game_id)
                        {
                            let should_send = match last_sent_games.get(&assigned_game_id) {
                                Some(prev) => prev != &current_game,
                                None => true,
                            };

                            if should_send {
                                last_sent_games.insert(assigned_game_id, current_game);
                                sequence_id = sequence_id.wrapping_add(1);
                                let packet = ServerPacket::with_session(
                                    ServerEvent::GameUpdate,
                                    sequence_id,
                                    worker.session_token,
                                    DATA_SIZE as u8,
                                    current_game.to_bytes(),
                                );
                                let _ = socket.send_to(&packet.to_bytes(), worker.address).await;
                            }
                        }
                    }
                }

                Ok(assignment) = game_assign_rx.recv() => {
                    if let Some(worker) = active_workers.get(&assignment.serial_number) {
                        let game = if assignment.game_id != 0 {
                            services.game().by_id(assignment.game_id)
                        } else {
                            None
                        }
                        .unwrap_or_else(|| Game::new_with_id(0, Discipline::Fiba5v5));

                        last_sent_games.insert(assignment.game_id, game);
                        sequence_id = sequence_id.wrapping_add(1);
                        let packet = ServerPacket::with_session(
                            ServerEvent::GameUpdate,
                            sequence_id,
                            worker.session_token,
                            DATA_SIZE as u8,
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
                                && let Some(game) = games.iter().find(|entry| entry.id() == assigned_game_id).copied()
                            {
                                let should_send = match last_sent_games.get(&assigned_game_id) {
                                    Some(prev) => prev != &game,
                                    None => true,
                                };

                                if should_send {
                                    last_sent_games.insert(assigned_game_id, game);
                                    sequence_id = sequence_id.wrapping_add(1);
                                    let packet = ServerPacket::with_session(
                                        ServerEvent::GameUpdate,
                                        sequence_id,
                                        worker.session_token,
                                        DATA_SIZE as u8,
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
}
