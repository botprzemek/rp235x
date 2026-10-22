use std::sync::Arc;

use axum::{
    Router,
    body::Bytes,
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures::stream::StreamExt;
use net::layout::PACKET_SIZE;
use net::{RawConnection, Transport};

use crate::services::Services;

pub struct WebSocketAdapter {
    socket: WebSocket,
}

impl WebSocketAdapter {
    pub fn new(socket: WebSocket) -> Self {
        Self { socket }
    }
}

impl RawConnection for WebSocketAdapter {
    async fn send(&mut self, data: &[u8; PACKET_SIZE]) -> Result<(), ()> {
        let payload = Bytes::copy_from_slice(data);
        self.socket
            .send(Message::Binary(payload))
            .await
            .map_err(|_| ())
    }

    async fn receive(&mut self, buffer: &mut [u8; PACKET_SIZE]) -> Result<usize, ()> {
        while let Some(msg_res) = self.socket.next().await {
            match msg_res {
                Ok(Message::Binary(bin)) => {
                    let len = bin.len().min(PACKET_SIZE);
                    buffer[..len].copy_from_slice(&bin[..len]);
                    return Ok(len);
                }
                Ok(Message::Close(_)) => return Err(()),
                Err(_) => return Err(()),
                _ => continue,
            }
        }
        Err(())
    }
}

pub struct WebsocketHandler;

impl WebsocketHandler {
    pub fn create(services: Arc<Services>) -> Router {
        Router::new()
            .route("/ws", get(Self::handle))
            .with_state(services)
    }

    async fn upgrade(socket: WebSocket, services: Arc<Services>) {
        let adapter = WebSocketAdapter::new(socket);
        let transport = Transport::new(adapter, None);

        crate::adapters::net::run(transport, services).await;
    }

    pub async fn handle(
        ws: WebSocketUpgrade,
        State(services): State<Arc<Services>>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| Self::upgrade(socket, services))
    }
}
