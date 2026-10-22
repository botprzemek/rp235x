pub mod client;
pub mod server;

pub use client::ClientPacket;
pub use server::ServerPacket;

use crate::event::{ClientEvent, ServerEvent};
use crate::layout::{DATA_SIZE, PACKET_SIZE};

pub trait Packet: Sized + Copy + Send + Sync {
    type Event: Copy + Send + Sync;

    fn event(&self) -> Self::Event;
    fn sequence_id(&self) -> u8;
    fn version(&self) -> u8;
    fn data(self) -> [u8; DATA_SIZE];
    fn to_bytes(&self) -> [u8; PACKET_SIZE];
    fn from_bytes(buffer: [u8; PACKET_SIZE]) -> Result<Self, &'static str>;
}

impl Packet for ClientPacket {
    type Event = ClientEvent;

    fn event(&self) -> Self::Event {
        ClientPacket::event(self)
    }

    fn sequence_id(&self) -> u8 {
        ClientPacket::sequence_id(self)
    }

    fn version(&self) -> u8 {
        ClientPacket::version(self)
    }

    fn data(self) -> [u8; DATA_SIZE] {
        ClientPacket::data(self)
    }

    fn to_bytes(&self) -> [u8; PACKET_SIZE] {
        ClientPacket::to_bytes(self)
    }

    fn from_bytes(buffer: [u8; PACKET_SIZE]) -> Result<Self, &'static str> {
        ClientPacket::from_bytes(buffer)
    }
}

impl Packet for ServerPacket {
    type Event = ServerEvent;

    fn event(&self) -> Self::Event {
        ServerPacket::event(self)
    }

    fn sequence_id(&self) -> u8 {
        ServerPacket::sequence_id(self)
    }

    fn version(&self) -> u8 {
        ServerPacket::version(self)
    }

    fn data(self) -> [u8; DATA_SIZE] {
        ServerPacket::data(self)
    }

    fn to_bytes(&self) -> [u8; PACKET_SIZE] {
        ServerPacket::to_bytes(self)
    }

    fn from_bytes(buffer: [u8; PACKET_SIZE]) -> Result<Self, &'static str> {
        ServerPacket::from_bytes(buffer)
    }
}
