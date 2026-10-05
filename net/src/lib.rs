#![no_std]

pub mod data;
pub mod event;
pub mod layout;
pub mod packet;

pub use event::{ClientEvent, ServerEvent};
pub use packet::{ClientPacket, ServerPacket};

extern crate alloc;
use core::future::Future;
use core::net::SocketAddr;

use crate::layout::PACKET_SIZE;

pub struct WebSocketTransport<W> {
    socket: W,
    address: Option<SocketAddr>,
}

pub trait PacketSender: Send + Sync {
    fn send_packet(&mut self, data: &[u8; PACKET_SIZE]) -> impl Future<Output = ()> + Send;
}

pub trait PacketReceiver: Send + Sync {
    fn receive_packet(
        &mut self,
    ) -> impl Future<Output = ([u8; PACKET_SIZE], Option<SocketAddr>)> + Send;
}

pub trait WebSocketConnection {
    fn send_binary(
        &mut self,
        data: &[u8; PACKET_SIZE],
    ) -> impl Future<Output = Result<(), ()>> + Send;
    fn receive_binary(
        &mut self,
        buffer: &mut [u8; PACKET_SIZE],
    ) -> impl Future<Output = Result<usize, ()>> + Send;
}

impl<W> WebSocketTransport<W> {
    pub fn new(socket: W, address: Option<SocketAddr>) -> Self {
        Self { socket, address }
    }
}

impl<W> PacketSender for WebSocketTransport<W>
where
    W: WebSocketConnection + Send + Sync,
{
    async fn send_packet(&mut self, data: &[u8; PACKET_SIZE]) {
        let _ = self.socket.send_binary(data).await;
    }
}

impl<W> PacketReceiver for WebSocketTransport<W>
where
    W: WebSocketConnection + Send + Sync,
{
    async fn receive_packet(&mut self) -> ([u8; PACKET_SIZE], Option<SocketAddr>) {
        let mut buffer = [0u8; PACKET_SIZE];

        loop {
            let response_size = match self.socket.receive_binary(&mut buffer).await {
                Ok(response_size) => response_size,
                Err(_) => continue,
            };

            if response_size != PACKET_SIZE {
                continue;
            }

            return (buffer, self.address);
        }
    }
}
