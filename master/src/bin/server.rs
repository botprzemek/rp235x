use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::State as AxumState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use futures::SinkExt;
use futures::stream::StreamExt;
use rust_embed::RustEmbed;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, watch};
use tokio::time::{self, Instant};
use tower_http::cors::CorsLayer;

use master::App;
use net::{
    ClientPacket, ServerPacket,
    data::snapshot::{Discipline, Snapshot, State, Team},
    event::{ClientEvent, ServerEvent},
};

const FRAME_DURATION: Duration = Duration::from_millis(64);

#[derive(serde::Deserialize)]
struct ScoreRequest {
    team: Team,
    points: u16,
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
    state_tx: watch::Sender<Snapshot>,
}

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Assets;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => match Assets::get("index.html") {
            Some(content) => Response::builder()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(content.data.into_owned()))
                .unwrap(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
    }
}

async fn render_handler() -> impl IntoResponse {
    let renderer = yew::ServerRenderer::<App>::new();
    let rendered_html = renderer.render().await;

    let index_html = match Assets::get("index.html") {
        Some(content) => String::from_utf8_lossy(&content.data).into_owned(),
        None => "<html><body></body></html>".to_string(),
    };

    let html = if let Some((before, after)) = index_html.split_once("<body>") {
        format!("{}<body>{}{}", before, rendered_html, after)
    } else {
        rendered_html
    };

    axum::response::Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    AxumState(state): AxumState<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let response = sender.send(Message::Text(msg.into())).await;
            if response.is_err() {
                break;
            }

            continue;
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }

            continue;
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

async fn api_score_handler(
    AxumState(state): AxumState<AppState>,
    axum::Json(body): axum::Json<ScoreRequest>,
) -> impl IntoResponse {
    state.state_tx.send_modify(|data| {
        data.make_field_goal(body.team, body.points);
    });
    let json = serde_json::to_string(&*state.state_tx.borrow()).unwrap_or_default();
    let _ = state.tx.send(json);
    axum::http::StatusCode::OK
}

async fn api_match_state_handler(
    AxumState(state): AxumState<AppState>,
    uri: axum::http::Uri,
) -> impl IntoResponse {
    let action = uri.path().strip_prefix("/api/").unwrap_or("unknown");
    let new_state = match action {
        "start" => State::Running,
        "stop" => State::Paused,
        _ => State::Idle,
    };

    state.state_tx.send_modify(|data| {
        data.set_state(new_state);
    });
    let json = serde_json::to_string(&*state.state_tx.borrow()).unwrap_or_default();
    let _ = state.tx.send(json);
    axum::http::StatusCode::OK
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let rx_socket = Arc::new(UdpSocket::bind("0.0.0.0:8000").await?);
    rx_socket.set_broadcast(true)?;

    let (state_tx, state_rx) = watch::channel(Snapshot::new(Discipline::FIBA5V5));
    let (tx, _rx) = broadcast::channel(100);

    let actor_state_rx = state_rx.clone();
    let actor_state_tx = state_tx.clone();
    let actor_ws_tx = tx.clone();

    tokio::spawn(async move {
        let mut interval = time::interval(FRAME_DURATION);
        let mut last_score_update = Instant::now();

        loop {
            interval.tick().await;

            let mut current_state = *actor_state_rx.borrow();

            if current_state.state() == State::Running
                && last_score_update.elapsed() >= Duration::from_secs(1)
            {
                last_score_update = Instant::now();
            }
            current_state.tick(FRAME_DURATION);

            let _ = actor_state_tx.send(current_state);
            if let Ok(json) = serde_json::to_string(&current_state) {
                let _ = actor_ws_tx.send(json);
            }
        }
    });

    let udp_socket = Arc::clone(&rx_socket);
    let udp_state_rx = state_rx.clone();
    let _udp_state_tx = state_tx.clone();

    tokio::spawn(async move {
        let mut sequence_id: u8 = 0;

        loop {
            let mut worker_addr = None;

            while worker_addr.is_none() {
                let mut rx_buf = [0u8; net::layout::PACKET_SIZE];
                let (size, remote_addr) = match udp_socket.recv_from(&mut rx_buf).await {
                    Ok((size, remote_addr)) => (size, remote_addr),
                    Err(_) => continue,
                };

                if size != net::layout::PACKET_SIZE {
                    continue;
                }

                let packet_array = match rx_buf[..net::layout::PACKET_SIZE].try_into() {
                    Ok(packet_array) => packet_array,
                    Err(_) => continue,
                };

                let packet = match ClientPacket::from_bytes(&packet_array) {
                    Ok(packet) => packet,
                    Err(_) => continue,
                };

                match packet.event() {
                    ClientEvent::HandshakeRequest | ClientEvent::Start | ClientEvent::Heartbeat => {
                        worker_addr = Some(remote_addr);
                        println!("[MASTER] Połączono z workerem UDP: {}", remote_addr);

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
                    _ => (),
                }
            }

            let worker_addr = worker_addr.unwrap();
            let mut _last_packet_received = Instant::now();
            let mut interval = time::interval(FRAME_DURATION);

            loop {
                let mut rx_buf = [0u8; net::layout::PACKET_SIZE];

                sequence_id = sequence_id.wrapping_add(1);
                interval.tick().await;

                let (size, remote_addr) = match tokio::time::timeout(
                    Duration::from_millis(1),
                    udp_socket.recv_from(&mut rx_buf),
                )
                .await
                {
                    Ok(Ok((size, remote_addr))) => (size, remote_addr),
                    _ => continue,
                };

                if remote_addr == worker_addr {
                    _last_packet_received = Instant::now();
                }

                if size != net::layout::PACKET_SIZE {
                    continue;
                }

                let packet_array = match rx_buf[..net::layout::PACKET_SIZE].try_into() {
                    Ok(packet_array) => packet_array,
                    Err(_) => continue,
                };

                let packet = match ClientPacket::from_bytes(&packet_array) {
                    Ok(packet) => packet,
                    Err(_) => continue,
                };

                match packet.event() {
                    ClientEvent::Heartbeat => _last_packet_received = Instant::now(),
                    _ => continue,
                }

                let data = *udp_state_rx.borrow();
                let packet = ServerPacket::new(ServerEvent::Snapshot, sequence_id, data.to_bytes());
                let _ = udp_socket.send_to(&packet.to_bytes(), worker_addr).await;
            }
        }
    });

    let app_state = AppState { tx, state_tx };

    let app = Router::new()
        .route("/", get(render_handler))
        .route("/ws", get(ws_handler))
        .route("/api/score", post(api_score_handler))
        .route("/api/start", post(api_match_state_handler))
        .route("/api/stop", post(api_match_state_handler))
        .fallback(static_handler)
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    println!("http://localhost:8080/");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
