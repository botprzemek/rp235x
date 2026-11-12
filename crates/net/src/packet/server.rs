//! Server-to-client wire packet implementation.

use crate::packet::crc16_ccitt;
#[cfg(feature = "serde")]
use crate::packet::serialize_data;
use crate::{ServerEvent, layout};

const PACKET_MAGIC: u8 = 0x42;
const PACKET_VERSION: u8 = 0x01;

/// 64-byte packet sent from the server to a client.
///
/// Wire frame layout:
/// `[Magic: 1B] [Version: 1B] [Event: 1B] [SeqId: 2B] [SessionToken: 4B] [PayloadLen: 1B] [Payload: 52B] [CRC: 2B]`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ServerPacket {
    magic: u8,
    version: u8,
    event: ServerEvent,
    sequence_id: u16,
    session_token: u32,
    payload_len: u8,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_data"))]
    data: [u8; layout::DATA_SIZE],
    crc: u16,
}

impl ServerPacket {
    /// Constructs a new [`ServerPacket`] with `session_token = 0` and full payload length, computing its CRC-16 checksum.
    #[must_use]
    pub fn new(event: ServerEvent, sequence_id: u16, data: [u8; layout::DATA_SIZE]) -> Self {
        Self::with_session(event, sequence_id, 0, layout::DATA_SIZE as u8, data)
    }

    /// Constructs a new [`ServerPacket`] with an explicit `session_token` and `payload_len`.
    #[must_use]
    pub fn with_session(
        event: ServerEvent,
        sequence_id: u16,
        session_token: u32,
        payload_len: u8,
        data: [u8; layout::DATA_SIZE],
    ) -> Self {
        let clamped_len = payload_len.min(layout::DATA_SIZE as u8);
        let mut packet = Self {
            magic: PACKET_MAGIC,
            version: PACKET_VERSION,
            event,
            sequence_id,
            session_token,
            payload_len: clamped_len,
            data,
            crc: 0,
        };

        packet.crc = packet.calculate_crc();

        packet
    }

    /// Serializes this packet into a 64-byte wire array.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; layout::PACKET_SIZE] {
        let mut bytes = [0u8; layout::PACKET_SIZE];

        bytes[layout::MAGIC] = self.magic;
        bytes[layout::VERSION] = self.version;
        bytes[layout::EVENT] = self.event as u8;
        bytes[layout::SEQUENCE_ID_START..layout::SEQUENCE_ID_END]
            .copy_from_slice(&self.sequence_id.to_be_bytes());
        bytes[layout::SESSION_TOKEN_START..layout::SESSION_TOKEN_END]
            .copy_from_slice(&self.session_token.to_be_bytes());
        bytes[layout::PAYLOAD_LEN] = self.payload_len;

        bytes[layout::DATA_START..layout::DATA_END].copy_from_slice(&self.data);
        bytes[layout::CRC_START..layout::CRC_END].copy_from_slice(&self.crc.to_be_bytes());

        bytes
    }

    /// Parses and validates a [`ServerPacket`] from a 64-byte wire array.
    pub fn from_bytes(buffer: [u8; layout::PACKET_SIZE]) -> Result<Self, &'static str> {
        if buffer[layout::MAGIC] != PACKET_MAGIC {
            return Err("magic");
        }

        let payload_len = buffer[layout::PAYLOAD_LEN];
        if (payload_len as usize) > layout::DATA_SIZE {
            return Err("payload_len");
        }

        let mut seq_bytes = [0u8; 2];
        seq_bytes.copy_from_slice(&buffer[layout::SEQUENCE_ID_START..layout::SEQUENCE_ID_END]);

        let mut token_bytes = [0u8; 4];
        token_bytes
            .copy_from_slice(&buffer[layout::SESSION_TOKEN_START..layout::SESSION_TOKEN_END]);

        let mut crc_bytes = [0u8; 2];
        crc_bytes.copy_from_slice(&buffer[layout::CRC_START..layout::CRC_END]);

        let packet = Self {
            magic: buffer[layout::MAGIC],
            version: buffer[layout::VERSION],
            event: ServerEvent::try_from(buffer[layout::EVENT])?,
            sequence_id: u16::from_be_bytes(seq_bytes),
            session_token: u32::from_be_bytes(token_bytes),
            payload_len,
            data: {
                let mut data = [0u8; layout::DATA_SIZE];
                data.copy_from_slice(&buffer[layout::DATA_START..layout::DATA_END]);
                data
            },
            crc: u16::from_be_bytes(crc_bytes),
        };

        let expected_crc = crc16_ccitt(&buffer[..layout::CRC_START]);
        if expected_crc != packet.crc {
            return Err("crc");
        }

        Ok(packet)
    }

    fn calculate_crc(&self) -> u16 {
        let mut bytes = [0u8; layout::CRC_START];
        bytes[layout::MAGIC] = self.magic;
        bytes[layout::VERSION] = self.version;
        bytes[layout::EVENT] = self.event as u8;
        bytes[layout::SEQUENCE_ID_START..layout::SEQUENCE_ID_END]
            .copy_from_slice(&self.sequence_id.to_be_bytes());
        bytes[layout::SESSION_TOKEN_START..layout::SESSION_TOKEN_END]
            .copy_from_slice(&self.session_token.to_be_bytes());
        bytes[layout::PAYLOAD_LEN] = self.payload_len;
        bytes[layout::DATA_START..layout::DATA_END].copy_from_slice(&self.data);

        crc16_ccitt(&bytes)
    }

    /// Returns the [`ServerEvent`] of this packet.
    #[must_use]
    pub fn event(&self) -> ServerEvent {
        self.event
    }

    /// Returns the 16-bit sequence identifier of this packet.
    #[must_use]
    pub fn sequence_id(&self) -> u16 {
        self.sequence_id
    }

    /// Returns the 32-bit session token carried in the frame header.
    #[must_use]
    pub fn session_token(&self) -> u32 {
        self.session_token
    }

    /// Returns the active payload length in bytes.
    #[must_use]
    pub fn payload_len(&self) -> u8 {
        self.payload_len
    }

    /// Returns the protocol version of this packet.
    #[must_use]
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the 52-byte payload of this packet.
    #[must_use]
    pub fn data(self) -> [u8; layout::DATA_SIZE] {
        self.data
    }
}
