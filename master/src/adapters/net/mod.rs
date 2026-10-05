mod http;
mod udp;

use std::sync::Arc;

use basketball::Game;
use bytes::Bytes;
pub use http::WebDriver;
use net::{
    ClientPacket, PacketReceiver, PacketSender, ServerPacket, WebSocketConnection,
    WebSocketTransport,
};
pub use udp::WorkerDriver;

use crate::services::Services;

pub async fn run_server<W>(mut listener: WebSocketTransport<W>, services: Arc<Services>)
where
    W: WebSocketConnection, // Przykładowy trait/obiekt dostarczający nowe połączenia
{
    // Główna pętla akceptująca nowych klientów
    while let Some(mut socket) = listener.accept().await {
        let services_clone = Arc::clone(&services);

        // DLA KAŻDEGO KLIENTA TWORZYMY OSOBNY TASK (Tokio task)
        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket, services_clone).await {
                eprintln!("Błąd obsługi klienta: {}", e);
            }
        });
    }
}

/// Obsługa pojedynczego klienta w osobnym tasku
async fn handle_client<W>(
    websocket_transport: &mut W,
    services: Arc<Services>,
) -> Result<(), Box<dyn std::error::Error>>
where
    W: PacketSender + PacketReceiver,
{
    let sequence_id: u8 = 1;
    let mut interval = tokio::time::interval(core::time::Duration::from_millis(16));

    loop {
        tokio::select! {
            // Wysyłanie stanu gry do TEGO konkretnego klienta co 16ms
            _ = interval.tick() => {
                let game = services.game().get();
                let packet = ServerPacket::new(net::ServerEvent::GameUpdate, sequence_id, game.to_bytes());
                websocket_transport.send_packet(&packet.to_bytes());
            }

            // Odbieranie pakietów od TEGO konkretnego klienta
            res = websocket_transport.receive_packet() => {
                let (data, _addr) = res;
                if data.is_empty() {
                    break; // Puste dane / rozłączenie
                }

                let packet = match ClientPacket::from_bytes(data) {
                    Ok(packet) => packet,
                    Err(error) => {
                        eprintln!("packet error: {}", error);
                        continue;
                    }
                };

                let game = match Game::from_bytes(&packet.data()) {
                    Ok(game) => game,
                    Err(error) => {
                        eprintln!("game error: {}", error);
                        continue;
                    }
                };

                println!("game: {:?}", game);
            }
        }
    }

    Ok(())
}

use futures::{sink::SinkExt, stream::StreamExt};
use net::layout::PACKET_SIZE;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct TungsteniteAdapter<S> {
    inner: tokio_tungstenite::WebSocketStream<S>,
}

impl<S> TungsteniteAdapter<S> {
    pub fn new(stream: tokio_tungstenite::WebSocketStream<S>) -> Self {
        Self { inner: stream }
    }
}

// Implementujemy nasz interfejs WebSocketConnection dla strumienia tokio-tungstenite
impl<S> WebSocketConnection for TungsteniteAdapter<S>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + Sync,
{
    async fn send_binary(&mut self, data: &[u8; PACKET_SIZE]) -> Result<(), ()> {
        // Konwertujemy naszą stałą tablicę na binarną wiadomość Tungstenite
        let msg = Message::Binary(Bytes::copy_from_slice(data));

        // Wysyłamy przez SinkExt (wymaga zaimportowania SinkExt)
        self.inner.send(msg).await.map_err(|_| ())
    }

    async fn receive_binary(&mut self, buffer: &mut [u8; PACKET_SIZE]) -> Result<usize, ()> {
        // Odbieramy kolejną wiadomość ze StreamExt
        while let Some(msg_res) = self.inner.next().await {
            match msg_res {
                Ok(Message::Binary(bin)) => {
                    let len = bin.len().min(PACKET_SIZE);
                    buffer[..len].copy_from_slice(&bin[..len]);
                    return Ok(len);
                }
                Ok(Message::Close(_)) => return Err(()),
                _ => continue, // Ignorujemy Ping/Pong/Text, czekając na binarne pakiety gry
            }
        }
        Err(())
    }
}
