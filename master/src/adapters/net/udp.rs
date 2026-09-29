use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::UdpSocket, time};

use crate::services::{Observable, Services};
use net::{ClientEvent, ClientPacket, ServerEvent, ServerPacket};

const FRAME_DURATION: Duration = Duration::from_millis(16); // ~60 FPS

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
        // Tworzymy kanał watch do przekazywania adresu aktywnego workera bez Mutexów
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
        worker_tx: tokio::sync::watch::Sender<Option<SocketAddr>>,
    ) {
        let mut sequence_id: u8 = 0;
        let mut rx_buf = [0u8; net::layout::PACKET_SIZE];

        loop {
            let (size, remote_addr) = match socket.recv_from(&mut rx_buf).await {
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

            let packet = match ClientPacket::from_bytes(&packet_array) {
                Ok(p) => p,
                Err(_) => continue,
            };

            match packet.event() {
                ClientEvent::GameStart
                | ClientEvent::HandshakeRequest
                | ClientEvent::HandshakeHeartbeat => {
                    // Rejestrujemy / odświeżamy adres workera w kanale watch
                    let _ = worker_tx.send(Some(remote_addr));

                    sequence_id = sequence_id.wrapping_add(1);
                    let current_game = services.game().get();

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

    async fn handle_ticks(
        socket: Arc<UdpSocket>,
        services: Arc<Services>,
        mut worker_rx: tokio::sync::watch::Receiver<Option<SocketAddr>>,
    ) {
        let mut interval = time::interval(FRAME_DURATION);
        let mut sequence_id: u8 = 0;
        let mut broadcast_rx = services.game().subscribe();

        loop {
            let active_worker = *worker_rx.borrow();

            tokio::select! {
                _ = interval.tick() => {
                    let mut current_game = services.game().get();
                    current_game.tick(FRAME_DURATION);
                    services.game().update(current_game.clone());

                    if let Some(addr) = active_worker {
                        sequence_id = sequence_id.wrapping_add(1);
                        let packet = ServerPacket::new(
                            ServerEvent::GameUpdate,
                            sequence_id,
                            current_game.to_bytes(),
                        );
                        let _ = socket.send_to(&packet.to_bytes(), addr).await;
                    }
                }

                // 2. Gdy stan zmieni się przez HTTP / WebSockety (odbieramy z kanału broadcast)
                result = broadcast_rx.recv() => {
                    match result {
                        Ok(game) => {
                            if let Some(addr) = active_worker {
                                sequence_id = sequence_id.wrapping_add(1);
                                let packet = ServerPacket::new(
                                    ServerEvent::GameUpdate,
                                    sequence_id,
                                    game.to_bytes(),
                                );
                                let _ = socket.send_to(&packet.to_bytes(), addr).await;
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }
}
