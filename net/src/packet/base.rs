//! Shared [`Packet`] trait and payload serialization helpers.

#[cfg(feature = "serde")]
use serde::Serializer;
#[cfg(feature = "serde")]
use serde::ser::SerializeTuple;

use crate::event::{ClientEvent, ServerEvent};
use crate::layout;
use crate::packet::{ClientPacket, ServerPacket};

/// Common interface for fixed-size 64-byte wire packets.
pub trait Packet: Sized + Copy + Send + Sync {
    /// Event enum carried by this packet type.
    type Event: Copy + Send + Sync;

    /// Returns the packet's event type.
    fn event(&self) -> Self::Event;

    /// Returns the packet's sequence identifier.
    fn sequence_id(&self) -> u8;

    /// Returns the protocol version of the packet.
    fn version(&self) -> u8;

    /// Returns the 59-byte payload of the packet.
    fn data(self) -> [u8; layout::DATA_SIZE];

    /// Serializes the packet into a 64-byte wire buffer.
    fn to_bytes(&self) -> [u8; layout::PACKET_SIZE];

    /// Parses and validates a packet from a 64-byte wire buffer.
    fn from_bytes(buffer: [u8; layout::PACKET_SIZE]) -> Result<Self, &'static str>;
}

impl Packet for ClientPacket {
    type Event = ClientEvent;

    fn event(&self) -> Self::Event {
        Self::event(self)
    }

    fn sequence_id(&self) -> u8 {
        Self::sequence_id(self)
    }

    fn version(&self) -> u8 {
        Self::version(self)
    }

    fn data(self) -> [u8; layout::DATA_SIZE] {
        Self::data(self)
    }

    fn to_bytes(&self) -> [u8; layout::PACKET_SIZE] {
        Self::to_bytes(self)
    }

    fn from_bytes(buffer: [u8; layout::PACKET_SIZE]) -> Result<Self, &'static str> {
        Self::from_bytes(buffer)
    }
}

impl Packet for ServerPacket {
    type Event = ServerEvent;

    fn event(&self) -> Self::Event {
        Self::event(self)
    }

    fn sequence_id(&self) -> u8 {
        Self::sequence_id(self)
    }

    fn version(&self) -> u8 {
        Self::version(self)
    }

    fn data(self) -> [u8; layout::DATA_SIZE] {
        Self::data(self)
    }

    fn to_bytes(&self) -> [u8; layout::PACKET_SIZE] {
        Self::to_bytes(self)
    }

    fn from_bytes(buffer: [u8; layout::PACKET_SIZE]) -> Result<Self, &'static str> {
        Self::from_bytes(buffer)
    }
}

/// Serializes the fixed-size packet payload array as a serde tuple.
#[cfg(feature = "serde")]
pub fn serialize_data<S>(data: &[u8; layout::DATA_SIZE], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut tuple = serializer.serialize_tuple(layout::DATA_SIZE)?;

    for byte in data {
        tuple.serialize_element(byte)?;
    }

    tuple.end()
}
