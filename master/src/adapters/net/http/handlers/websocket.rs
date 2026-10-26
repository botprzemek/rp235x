use core::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    body::Bytes,
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use futures::{
    SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use net::layout::PACKET_SIZE;
use net::{PacketReceiver, PacketSender};

use crate::services::Services;

/// Outbound half of a WebSocket connection.
pub struct WebSocketSender {
    sink: SplitSink<WebSocket, Message>,
}

impl PacketSender for WebSocketSender {
    async fn send_packet(&mut self, data: &[u8; PACKET_SIZE]) {
        let payload = Bytes::copy_from_slice(data);
        let _ = self.sink.send(Message::Binary(payload)).await;
    }
}

/// Inbound half of a WebSocket connection.
pub struct WebSocketReceiver {
    stream: SplitStream<WebSocket>,
}

impl PacketReceiver for WebSocketReceiver {
    async fn receive_packet(
        &mut self,
    ) -> Result<([u8; PACKET_SIZE], Option<SocketAddr>), &'static str> {
        while let Some(msg_res) = self.stream.next().await {
            match msg_res {
                Ok(Message::Binary(bin)) if bin.len() == PACKET_SIZE => {
                    let mut buffer = [0u8; PACKET_SIZE];
                    buffer.copy_from_slice(&bin);
                    return Ok((buffer, None));
                }
                Ok(Message::Close(_)) | Err(_) => return Err("disconnected"),
                _ => continue,
            }
        }
        Err("disconnected")
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
        let (sink, stream) = socket.split();
        let sender = WebSocketSender { sink };
        let receiver = WebSocketReceiver { stream };

        crate::adapters::net::run(receiver, sender, services).await;
    }

    pub async fn handle(
        ws: WebSocketUpgrade,
        State(services): State<Arc<Services>>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| Self::upgrade(socket, services))
    }
}
