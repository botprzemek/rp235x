use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::UdpSocket, time};

use basketball::{Discipline, Game};
use config::DeviceConfig;
use net::{ClientEvent, ClientPacket, Device, ServerEvent, ServerPacket};

use crate::services::{Observable, Services};

const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60); // 60 Hz (60 updates/s)

pub struct WorkerDriver {
    listener: Arc<UdpSocket>,
}

impl WorkerDriver {
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
        let (worker_tx, worker_rx) = tokio::sync::watch::channel(None);

        let rx_socket = Arc::clone(&self.listener);
        let rx_services = Arc::clone(&services);
        tokio::spawn(async move {
            Self::handle_incoming(rx_socket, rx_services, worker_tx).await;
        });

        let tx_socket = Arc::clone(&self.listener);
        let tx_services = Arc::clone(&services);
        tokio::spawn(async move {
            Self::handle_ticks(tx_socket, tx_services, worker_rx).await;
        });
    }

    async fn handle_incoming(
        socket: Arc<UdpSocket>,
        services: Arc<Services>,
        worker_tx: tokio::sync::watch::Sender<Option<(SocketAddr, String)>>,
    ) {
        let mut sequence_id: u8 = 0;
        let mut rx_buf = [0u8; net::layout::PACKET_SIZE];
        let mut disconnect_rx = services.devices().subscribe_disconnect();

        loop {
            tokio::select! {
                Ok(target_sn) = disconnect_rx.recv() => {
                    let should_clear = worker_tx
                        .borrow()
                        .as_ref()
                        .is_some_and(|(_, sn)| sn == &target_sn);
                    if should_clear {
                        println!("INFO: [UDP] Forcefully disconnected worker sn={}", target_sn);
                        let _ = worker_tx.send(None);
                    }
                }

                recv_res = socket.recv_from(&mut rx_buf) => {
                    let (size, remote_addr) = match recv_res {
                        Ok(res) => res,
                        Err(_) => continue,
                    };

                    if size != net::layout::PACKET_SIZE {
                        continue;
                    }

                    let packet_array = match rx_buf[..net::layout::PACKET_SIZE].try_into() {
                        Ok(arr) => arr,
                        Err(_) => continue,
                    };

                    let packet = match ClientPacket::from_bytes(packet_array) {
                        Ok(p) => p,
                        Err(_) => continue,
                    };

                    match packet.event() {
                        ClientEvent::HandshakeRequest | ClientEvent::HandshakeHeartbeat => {
                            let mut device = match Device::try_from(packet) {
                                Ok(device) => device,
                                Err(err) => {
                                    eprintln!(
                                        "WARN: [UDP {}] Invalid worker handshake payload: {}",
                                        remote_addr, err
                                    );
                                    continue;
                                }
                            };

                            let sn = device.get_serial_number().unwrap_or("unknown").to_string();
                            let assigned_game_id = services
                                .devices()
                                .get_game_id_by_serial(&sn)
                                .unwrap_or(0);
                            device.set_game_id(assigned_game_id);

                            if let Err(err) = services.devices().validate_and_register(device) {
                                eprintln!(
                                    "WARN: [UDP {}] Worker device validation failed: {}",
                                    remote_addr, err
                                );
                                continue;
                            }

                            println!(
                                "INFO: [UDP {}] Validated device {:?} (id={}, sn={}, game_id={})",
                                remote_addr,
                                device.device_type(),
                                device.get_device_id().unwrap_or("unknown"),
                                sn,
                                device.game_id(),
                            );

                            let _ = worker_tx.send(Some((remote_addr, sn)));

                            sequence_id = sequence_id.wrapping_add(1);

                            let current_game = if assigned_game_id != 0 {
                                services.game().get_by_id(assigned_game_id)
                            } else {
                                None
                            }
                            .unwrap_or_else(|| Game::with_id(0, Discipline::Fiba5vs5));

                            let ack_packet = ServerPacket::new(
                                ServerEvent::HandshakeAck,
                                sequence_id,
                                current_game.to_bytes(),
                            );
                            let _ = socket.send_to(&ack_packet.to_bytes(), remote_addr).await;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    async fn handle_ticks(
        socket: Arc<UdpSocket>,
        services: Arc<Services>,
        worker_rx: tokio::sync::watch::Receiver<Option<(SocketAddr, String)>>,
    ) {
        let mut interval = time::interval(FRAME_DURATION);
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
        let mut sequence_id: u8 = 0;
        let mut broadcast_rx = services.game().subscribe();
        let mut game_assign_rx = services.devices().subscribe_game_assign();

        loop {
            let active_worker = worker_rx.borrow().clone();

            tokio::select! {
                _ = interval.tick() => {
                    if let Some((addr, ref sn)) = active_worker {
                        let assigned_game_id = services
                            .devices()
                            .get_game_id_by_serial(sn)
                            .unwrap_or(0);
                        if assigned_game_id != 0
                            && let Some(current_game) = services.game().get_by_id(assigned_game_id)
                        {
                            sequence_id = sequence_id.wrapping_add(1);
                            let packet = ServerPacket::new(
                                ServerEvent::GameUpdate,
                                sequence_id,
                                current_game.to_bytes(),
                            );
                            let _ = socket.send_to(&packet.to_bytes(), addr).await;
                        }
                    }
                }

                Ok((target_sn, new_game_id)) = game_assign_rx.recv() => {
                    if let Some((addr, ref sn)) = active_worker
                        && sn == &target_sn
                    {
                        let game = if new_game_id != 0 {
                            services.game().get_by_id(new_game_id)
                        } else {
                            None
                        }
                        .unwrap_or_else(|| Game::with_id(0, Discipline::Fiba5vs5));

                        sequence_id = sequence_id.wrapping_add(1);
                        let packet = ServerPacket::new(
                            ServerEvent::GameUpdate,
                            sequence_id,
                            game.to_bytes(),
                        );
                        let _ = socket.send_to(&packet.to_bytes(), addr).await;
                    }
                }

                result = broadcast_rx.recv() => {
                    if let Ok(games) = result
                        && let Some((addr, ref sn)) = active_worker
                    {
                        let assigned_game_id = services
                            .devices()
                            .get_game_id_by_serial(sn)
                            .unwrap_or(0);
                        if assigned_game_id != 0
                            && let Some(game) = games.iter().find(|g| g.id() == assigned_game_id)
                        {
                            sequence_id = sequence_id.wrapping_add(1);
                            let packet = ServerPacket::new(
                                ServerEvent::GameUpdate,
                                sequence_id,
                                game.to_bytes(),
                            );
                            let _ = socket.send_to(&packet.to_bytes(), addr).await;
                        }
                    }
                }
            }
        }
    }
}
