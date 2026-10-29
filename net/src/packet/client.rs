//! Client-to-server wire packet implementation.

use crate::{ClientEvent, layout};

#[cfg(feature = "serde")]
use crate::packet::serialize_data;

const PACKET_MAGIC: u8 = 0x42;
const PACKET_VERSION: u8 = 0x01;

/// 64-byte packet sent from a client to the server.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ClientPacket {
    magic: u8,
    event: ClientEvent,
    sequence_id: u8,
    version: u8,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_data"))]
    data: [u8; layout::DATA_SIZE],
    crc: u8,
}

impl ClientPacket {
    /// Constructs a new [`ClientPacket`] and computes its CRC checksum.
    #[must_use]
    pub fn new(event: ClientEvent, sequence_id: u8, data: [u8; layout::DATA_SIZE]) -> Self {
        let mut packet = Self {
            magic: PACKET_MAGIC,
            event,
            sequence_id,
            version: PACKET_VERSION,
            data,
            crc: 0,
        };

        packet.crc = packet.calculate_crc();

        packet
    }

    /// Serializes this packet into a 64-byte array.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; layout::PACKET_SIZE] {
        let mut bytes = [0u8; layout::PACKET_SIZE];

        bytes[layout::MAGIC] = self.magic;
        bytes[layout::EVENT] = self.event as u8;
        bytes[layout::SEQUENCE_ID] = self.sequence_id;
        bytes[layout::VERSION] = self.version;

        bytes[layout::DATA_START..layout::DATA_END].copy_from_slice(&self.data);
        bytes[layout::CRC] = self.crc;

        bytes
    }

    /// Parses and validates a [`ClientPacket`] from a 64-byte array.
    pub fn from_bytes(buffer: [u8; layout::PACKET_SIZE]) -> Result<Self, &'static str> {
        if buffer[layout::MAGIC] != PACKET_MAGIC {
            return Err("magic");
        }

        let packet = Self {
            magic: buffer[layout::MAGIC],
            event: ClientEvent::try_from(buffer[layout::EVENT])?,
            sequence_id: buffer[layout::SEQUENCE_ID],
            version: buffer[layout::VERSION],
            data: {
                let mut data = [0u8; layout::DATA_SIZE];
                data.copy_from_slice(&buffer[layout::DATA_START..layout::DATA_END]);
                data
            },
            crc: buffer[layout::CRC],
        };

        if packet.calculate_crc() != packet.crc {
            return Err("crc");
        }

        Ok(packet)
    }

    fn calculate_crc(&self) -> u8 {
        let mut crc = self.magic ^ (self.event as u8) ^ self.sequence_id ^ self.version;

        for byte in self.data {
            crc ^= byte;
        }

        crc
    }

    /// Returns the [`ClientEvent`] of this packet.
    #[must_use]
    pub fn event(&self) -> ClientEvent {
        self.event
    }

    /// Returns the sequence identifier of this packet.
    #[must_use]
    pub fn sequence_id(&self) -> u8 {
        self.sequence_id
    }

    /// Returns the protocol version of this packet.
    #[must_use]
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the 59-byte payload of this packet.
    #[must_use]
    pub fn data(self) -> [u8; layout::DATA_SIZE] {
        self.data
    }
}
