//! Shared [`Packet`] trait, CRC-16 calculation, and payload serialization helpers.

use crate::event::{ClientEvent, ServerEvent};
use crate::layout;
use crate::packet::{ClientPacket, ServerPacket};

#[cfg(feature = "serde")]
use serde::Serializer;
#[cfg(feature = "serde")]
use serde::ser::SerializeTuple;

/// Computes the CRC-16-CCITT (`0x1021`, init `0xFFFF`) checksum over `bytes`.
#[must_use]
pub const fn crc16_ccitt(bytes: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    let mut i = 0;
    while i < bytes.len() {
        crc ^= (bytes[i] as u16) << 8;
        let mut bit = 0;
        while bit < 8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
            bit += 1;
        }
        i += 1;
    }
    crc
}

/// Common interface for fixed-size 64-byte wire packets.
pub trait Packet: Sized + Copy + Send + Sync {
    /// Event enum carried by this packet type.
    type Event: Copy + Send + Sync;

    /// Returns the packet's event type.
    fn event(&self) -> Self::Event;

    /// Returns the packet's 16-bit sequence identifier.
    fn sequence_id(&self) -> u16;

    /// Returns the packet's 32-bit session token.
    fn session_token(&self) -> u32;

    /// Returns the active payload length in bytes (`0..=DATA_SIZE`).
    fn payload_len(&self) -> u8;

    /// Returns the protocol version of the packet.
    fn version(&self) -> u8;

    /// Returns the 52-byte payload of the packet.
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

    fn sequence_id(&self) -> u16 {
        Self::sequence_id(self)
    }

    fn session_token(&self) -> u32 {
        Self::session_token(self)
    }

    fn payload_len(&self) -> u8 {
        Self::payload_len(self)
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

    fn sequence_id(&self) -> u16 {
        Self::sequence_id(self)
    }

    fn session_token(&self) -> u32 {
        Self::session_token(self)
    }

    fn payload_len(&self) -> u8 {
        Self::payload_len(self)
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
pub(crate) fn serialize_data<S>(
    data: &[u8; layout::DATA_SIZE],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut tuple = serializer.serialize_tuple(layout::DATA_SIZE)?;

    for byte in data {
        tuple.serialize_element(byte)?;
    }

    tuple.end()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_packet_roundtrip_and_header() {
        let mut payload = [0u8; layout::DATA_SIZE];
        payload[0] = 0xAA;
        payload[51] = 0xBB;

        let packet = ClientPacket::with_session(
            ClientEvent::HandshakeHeartbeat,
            0x1234,
            0xDEADBEEF,
            10,
            payload,
        );

        let bytes = packet.to_bytes();
        assert_eq!(bytes.len(), 64);
        assert_eq!(bytes[0], 0x42); // Magic
        assert_eq!(bytes[1], 0x01); // Version
        assert_eq!(bytes[2], ClientEvent::HandshakeHeartbeat as u8); // Event
        assert_eq!(&bytes[3..5], &[0x12, 0x34]); // SeqId (2B BE)
        assert_eq!(&bytes[5..9], &[0xDE, 0xAD, 0xBE, 0xEF]); // SessionToken (4B BE)
        assert_eq!(bytes[9], 10); // PayloadLen (1B)

        let decoded = ClientPacket::from_bytes(bytes).expect("valid packet");
        assert_eq!(decoded, packet);
        assert_eq!(decoded.sequence_id(), 0x1234);
        assert_eq!(decoded.session_token(), 0xDEADBEEF);
        assert_eq!(decoded.payload_len(), 10);
    }

    #[test]
    fn test_crc16_detects_corruption() {
        let packet = ServerPacket::with_session(
            ServerEvent::GameUpdate,
            42,
            0xCAFEBABE,
            17,
            [0x55; layout::DATA_SIZE],
        );
        let mut bytes = packet.to_bytes();
        // Flip a single bit in the session token
        bytes[6] ^= 0x01;
        assert_eq!(ServerPacket::from_bytes(bytes), Err("crc"));
    }

    #[test]
    fn test_handshake_ack_payload_roundtrip() {
        use crate::data::handshake_ack::HandshakeAck;

        let ack = HandshakeAck::new(105, 0x01);
        let payload = ack.to_bytes();
        let decoded = HandshakeAck::from_bytes(&payload).expect("valid handshake ack");
        assert_eq!(decoded.assigned_game_id(), 105);
        assert_eq!(decoded.flags(), 0x01);

        let packet = ServerPacket::with_session(
            ServerEvent::HandshakeAck,
            1,
            0x12345678,
            HandshakeAck::PAYLOAD_LEN,
            payload,
        );
        let wire = packet.to_bytes();
        let parsed = ServerPacket::from_bytes(wire).expect("valid packet");
        assert_eq!(parsed.session_token(), 0x12345678);
        assert_eq!(parsed.payload_len(), 3);
        let parsed_ack = HandshakeAck::from_bytes(&parsed.data()).expect("valid data");
        assert_eq!(parsed_ack.assigned_game_id(), 105);
    }

    #[test]
    fn test_heartbeat_and_heartbeat_ack_roundtrip() {
        use crate::data::heartbeat::Heartbeat;
        use crate::data::heartbeat_ack::HeartbeatAck;

        let hb = Heartbeat::new(0x02);
        let hb_bytes = hb.to_bytes();
        let decoded_hb = Heartbeat::from_bytes(&hb_bytes).expect("valid heartbeat");
        assert_eq!(decoded_hb.flags(), 0x02);

        let ack = HeartbeatAck::new(42, 0x00);
        let ack_bytes = ack.to_bytes();
        let decoded_ack = HeartbeatAck::from_bytes(&ack_bytes).expect("valid heartbeat ack");
        assert_eq!(decoded_ack.latest_game_seq(), 42);
        assert_eq!(decoded_ack.flags(), 0);
    }
}
