use crate::AppState;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use futures::SinkExt;
use futures::stream::StreamExt;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let response = sender.send(Message::Text(msg.into())).await;
            if response.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

pub async fn run(state: AppState) {
    let ws_app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state)
        .layer(CorsLayer::permissive());

    tokio::spawn(async move {
        println!("ws://localhost:9000/ws");
        let listener = TcpListener::bind("0.0.0.0:9000").await.unwrap();
        axum::serve(listener, ws_app).await.unwrap();
    });
}
