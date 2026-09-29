use std::sync::Arc;

use crate::services::{Observable, Services};

use axum::{
    Router,
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures::{SinkExt, stream::StreamExt};

pub struct WebsocketHandler;

impl WebsocketHandler {
    pub fn create(services: Arc<Services>) -> Router {
        Router::new()
            .route("/ws", get(Self::handle))
            .with_state(services)
    }

    async fn upgrade(socket: WebSocket, services: Arc<Services>) {
        let (mut sender, mut receiver) = socket.split();
        let mut rx = services.game().subscribe();
        let _ = services.game().watch();

        let mut send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                let payload = axum::body::Bytes::copy_from_slice(&msg.to_bytes());
                let response = sender.send(Message::Binary(payload)).await;

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

    pub async fn handle(
        ws: WebSocketUpgrade,
        State(services): State<Arc<Services>>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| Self::upgrade(socket, services))
    }
}
