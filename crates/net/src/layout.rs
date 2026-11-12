//! Byte offsets and sizes for the 64-byte wire packet layout.
//!
//! Frame Header (10 bytes):
//! `[Magic: 1B] [Version: 1B] [Event: 1B] [SeqId: 2B] [SessionToken: 4B] [PayloadLen: 1B]`
//! Body & Trailer (54 bytes):
//! `[Payload: 52B] [CRC-16: 2B]`

/// Header size in bytes (`Magic`..=`PayloadLen`).
pub const HEADER_SIZE: usize = 10;

/// Maximum payload size in bytes within a wire packet.
pub const DATA_SIZE: usize = 52;

/// Total wire packet size in bytes.
pub const PACKET_SIZE: usize = 64;

/// Byte index of the magic header byte (`1B`).
pub(crate) const MAGIC: usize = 0x00;

/// Byte index of the protocol version byte (`1B`).
pub(crate) const VERSION: usize = 0x01;

/// Byte index of the event identifier byte (`1B`).
pub(crate) const EVENT: usize = 0x02;

/// Start byte index (inclusive) of the packet sequence number (`u16` BE, `2B`).
pub(crate) const SEQUENCE_ID_START: usize = 0x03;

/// End byte index (exclusive) of the packet sequence number (`u16` BE, `2B`).
pub(crate) const SEQUENCE_ID_END: usize = 0x05;

/// Start byte index (inclusive) of the session token (`u32` BE, `4B`).
pub(crate) const SESSION_TOKEN_START: usize = 0x05;

/// End byte index (exclusive) of the session token (`u32` BE, `4B`).
pub(crate) const SESSION_TOKEN_END: usize = 0x09;

/// Byte index of the payload length byte (`1B`).
pub(crate) const PAYLOAD_LEN: usize = 0x09;

/// Start byte index (inclusive) of the packet payload (`52B`).
pub(crate) const DATA_START: usize = 0x0a;

/// End byte index (exclusive) of the packet payload (`52B`).
pub(crate) const DATA_END: usize = 0x3e;

/// Start byte index (inclusive) of the 16-bit CRC checksum (`u16` BE, `2B`).
pub(crate) const CRC_START: usize = 0x3e;

/// End byte index (exclusive) of the 16-bit CRC checksum (`u16` BE, `2B`).
pub(crate) const CRC_END: usize = 0x40;
