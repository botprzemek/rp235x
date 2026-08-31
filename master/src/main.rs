use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, watch};
use tokio::time::{self, Instant};
use warp::Filter;

use net::{
    ClientPacket, ServerPacket,
    data::snapshot::{Discipline, Snapshot, State},
    event::{ClientEvent, ServerEvent},
};

const FRAME_DURATION: Duration = Duration::from_millis(16);

#[derive(Deserialize)]
struct ScoreRequest {
    team: String,
    points: u32,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let rx_socket = Arc::new(UdpSocket::bind("0.0.0.0:8000").await?);
    rx_socket.set_broadcast(true)?;

    // Używamy channela typu watch do bezlockowego rozgłaszania stanu w pętli asynchronicznej
    let (state_tx, state_rx) = watch::channel(Snapshot::new(Discipline::FIBA5V5));
    let (ws_broadcast_tx, _ws_rx) = broadcast::channel(100);

    // Zadanie zarządzające głównym stanem gry (Actor)
    let mut actor_state_rx = state_rx.clone();
    let actor_state_tx = state_tx.clone();
    let actor_ws_tx = ws_broadcast_tx.clone();

    tokio::spawn(async move {
        let mut interval = time::interval(FRAME_DURATION);
        let mut last_score_update = Instant::now();

        loop {
            interval.tick().await;

            let mut current_state = *actor_state_rx.borrow();

            // Logika gry
            if current_state.state == State::Running
                && last_score_update.elapsed() >= Duration::from_secs(1)
            {
                last_score_update = Instant::now();
            }
            current_state.tick(FRAME_DURATION);

            // Wysyłamy nowy stan do watch i broadcast dla websocketów
            let _ = actor_state_tx.send(current_state);
            let _ = actor_ws_tx.send(current_state);
        }
    });

    // Uruchomienie serwera HTTP/WS
    let server_state_rx = state_rx.clone();
    let server_ws_tx = ws_broadcast_tx.clone();
    let api_state_tx = state_tx.clone();

    tokio::spawn(async move {
        start_http_server(server_state_rx, server_ws_tx, api_state_tx).await;
    });

    let udp_socket = Arc::clone(&rx_socket);
    let udp_state_rx = state_rx.clone();
    let udp_state_tx = state_tx.clone();

    tokio::spawn(async move {
        let mut sequence_id: u8 = 0;
        let mut tick_counter: u32 = 0;

        loop {
            let mut worker_addr = None;
            println!("[MASTER] Oczekiwanie na handshake...");

            while worker_addr.is_none() {
                let mut rx_buf = [0u8; net::layout::PACKET_SIZE];
                match udp_socket.recv_from(&mut rx_buf).await {
                    Ok((size, remote_addr)) => {
                        if size != net::layout::PACKET_SIZE {
                            continue;
                        }
                        let packet_array = match rx_buf[..net::layout::PACKET_SIZE].try_into() {
                            Ok(arr) => arr,
                            Err(_) => continue,
                        };

                        if let Ok(packet) = ClientPacket::from_bytes(&packet_array) {
                            // Akceptujemy HandshakeRequest LUB event Start/Heartbeat jako sygnał zgłoszenia się workera
                            match packet.event {
                                ClientEvent::HandshakeRequest
                                | ClientEvent::Start
                                | ClientEvent::Heartbeat => {
                                    worker_addr = Some(remote_addr);
                                    println!("[MASTER] Połączono z workerem: {}", remote_addr);

                                    sequence_id = sequence_id.wrapping_add(1);
                                    let data = *udp_state_rx.borrow();
                                    let ack_packet = ServerPacket::new(
                                        ServerEvent::HandshakeAck,
                                        sequence_id,
                                        data.to_bytes(),
                                    );
                                    let _ = udp_socket
                                        .send_to(&ack_packet.to_bytes(), remote_addr)
                                        .await;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("udp_read_error podczas handshake: {:?}", e);
                    }
                }
            }

            let worker_addr = worker_addr.unwrap();
            let mut last_packet_received = Instant::now();
            let mut interval = time::interval(FRAME_DURATION);

            let mut disconnected = false;
            while !disconnected {
                sequence_id = sequence_id.wrapping_add(1);
                tick_counter = tick_counter.wrapping_add(1);

                interval.tick().await;

                let mut rx_buf = [0u8; net::layout::PACKET_SIZE];
                match tokio::time::timeout(
                    Duration::from_millis(1),
                    udp_socket.recv_from(&mut rx_buf),
                )
                .await
                {
                    Ok(Ok((size, remote_addr))) => {
                        if remote_addr == worker_addr {
                            last_packet_received = Instant::now();
                        }
                        if size == net::layout::PACKET_SIZE {
                            if let Ok(packet_array) = rx_buf[..net::layout::PACKET_SIZE].try_into()
                            {
                                if let Ok(packet) = ClientPacket::from_bytes(&packet_array) {
                                    if let ClientEvent::Heartbeat = packet.event {
                                        last_packet_received = Instant::now();
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }

                if last_packet_received.elapsed() > Duration::from_secs(3) {
                    println!("[MASTER] Worker rozłączony. Powrót do handshaku...");
                    disconnected = true;
                    let mut paused_data = *udp_state_rx.borrow();
                    paused_data.state = State::Paused;
                    let _ = udp_state_tx.send(paused_data);
                    break;
                }

                let data = *udp_state_rx.borrow();
                let packet = ServerPacket::new(ServerEvent::Snapshot, sequence_id, data.to_bytes());
                let _ = udp_socket.send_to(&packet.to_bytes(), worker_addr).await;
            }
        }
    });

    std::future::pending::<()>().await;
    Ok(())
}

async fn start_http_server(
    state_rx: watch::Receiver<Snapshot>,
    tx: broadcast::Sender<Snapshot>,
    api_state_tx: watch::Sender<Snapshot>,
) {
    let state_filter = warp::any().map(move || state_rx.clone());
    let tx_filter = warp::any().map(move || tx.clone());
    let api_tx_filter = warp::any().map(move || api_state_tx.clone());

    let index = warp::path::end().and_then(|| async {
        match tokio::fs::read_to_string("index.html").await {
            Ok(content) => Ok(warp::reply::html(content)),
            Err(_) => Err(warp::reject::not_found()),
        }
    });

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(state_filter.clone())
        .and(tx_filter)
        .map(|ws: warp::ws::Ws, state, tx| {
            ws.on_upgrade(move |socket| handle_ws(socket, state, tx))
        });

    let start_route = warp::path("api")
        .and(warp::path("start"))
        .and(warp::post())
        .and(api_tx_filter.clone())
        .and_then(|tx: watch::Sender<Snapshot>| async move {
            tx.send_modify(|data| {
                data.state = State::Running;
            });
            Result::<_, warp::Rejection>::Ok(warp::reply::json(&"OK"))
        });

    let stop_route = warp::path("api")
        .and(warp::path("stop"))
        .and(warp::post())
        .and(api_tx_filter.clone())
        .and_then(|tx: watch::Sender<Snapshot>| async move {
            tx.send_modify(|data| {
                data.state = State::Paused;
            });
            Result::<_, warp::Rejection>::Ok(warp::reply::json(&"OK"))
        });

    let score_route = warp::path("api")
        .and(warp::path("score"))
        .and(warp::post())
        .and(warp::body::json())
        .and(api_tx_filter)
        .and_then(
            |body: ScoreRequest, tx: watch::Sender<Snapshot>| async move {
                tx.send_modify(|data| {
                    if body.team == "home" {
                        data.home_score = data.home_score.saturating_add(body.points as u16);
                    } else if body.team == "away" {
                        data.away_score = data.away_score.saturating_add(body.points as u16);
                    }
                });
                Result::<_, warp::Rejection>::Ok(warp::reply::json(&"OK"))
            },
        );

    let routes = index
        .or(ws_route)
        .or(start_route)
        .or(stop_route)
        .or(score_route);

    println!("[HTTP/WS] Serwer uruchomiony pod adresem http://localhost:8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

async fn handle_ws(
    ws: warp::ws::WebSocket,
    state_rx: watch::Receiver<Snapshot>,
    tx: broadcast::Sender<Snapshot>,
) {
    use futures_util::{SinkExt, StreamExt};
    let (mut ws_tx, _ws_rx) = ws.split();

    {
        let data = *state_rx.borrow();
        if let Ok(json) = serde_json::to_string(&data) {
            let _ = ws_tx.send(warp::ws::Message::text(json)).await;
        }
    }

    let mut rx = tx.subscribe();
    tokio::spawn(async move {
        while let Ok(game) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&game) {
                if ws_tx.send(warp::ws::Message::text(json)).await.is_err() {
                    break;
                }
            }
        }
    });
}
