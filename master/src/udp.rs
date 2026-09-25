pub async fn run() {
    let active_worker: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));
    let rx_udp_socket = Arc::clone(&udp_socket);
    let rx_worker_addr = Arc::clone(&active_worker);
    let rx_state_rx = state_rx.clone();

    tokio::spawn(async move {
        let mut sequence_id: u8 = 0;
        let mut rx_buf = [0u8; net::layout::PACKET_SIZE];

        loop {
            let (size, remote_addr) = match rx_udp_socket.recv_from(&mut rx_buf).await {
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
                ClientEvent::HandshakeRequest | ClientEvent::Start | ClientEvent::Heartbeat => {
                    let mut worker_lock = rx_worker_addr.lock().await;
                    if worker_lock.is_none() || worker_lock.unwrap() != remote_addr {
                        println!("[MASTER] Połączono z workerem UDP: {}", remote_addr);
                    }
                    *worker_lock = Some(remote_addr);

                    sequence_id = sequence_id.wrapping_add(1);
                    let data = *rx_state_rx.borrow();
                    let ack_packet =
                        ServerPacket::new(ServerEvent::HandshakeAck, sequence_id, data.to_bytes());
                    let _ = rx_udp_socket
                        .send_to(&ack_packet.to_bytes(), remote_addr)
                        .await;
                }
                _ => {}
            }
        }
    });

    let tick_state_rx = state_rx.clone();
    let tick_state_tx = state_tx.clone();
    let tick_ws_tx = tx.clone();
    let tick_udp_socket = Arc::clone(&udp_socket);
    let tick_worker_addr = Arc::clone(&active_worker);

    tokio::spawn(async move {
        let mut interval = time::interval(FRAME_DURATION);
        let mut sequence_id: u8 = 0;

        loop {
            interval.tick().await;

            let mut current_state = *tick_state_rx.borrow();
            current_state.tick(FRAME_DURATION);

            let _ = tick_state_tx.send(current_state);

            if let Ok(json) = serde_json::to_string(&current_state) {
                let _ = tick_ws_tx.send(json);
            }

            let worker = *tick_worker_addr.lock().await;
            if let Some(addr) = worker {
                sequence_id = sequence_id.wrapping_add(1);
                let packet =
                    ServerPacket::new(ServerEvent::Snapshot, sequence_id, current_state.to_bytes());
                let _ = tick_udp_socket.send_to(&packet.to_bytes(), addr).await;
            }
        }
    });
}
