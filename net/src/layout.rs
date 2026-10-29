//! Byte offsets and sizes for the 64-byte wire packet layout.

/// Payload size in bytes within a wire packet.
pub const DATA_SIZE: usize = 59;

/// Total wire packet size in bytes.
pub const PACKET_SIZE: usize = 64;

/// Byte index of the magic header byte.
pub const MAGIC: usize = 0x00;

/// Byte index of the event identifier byte.
pub const EVENT: usize = 0x01;

/// Byte index of the packet sequence number.
pub const SEQUENCE_ID: usize = 0x02;

/// Byte index of the protocol version byte.
pub const VERSION: usize = 0x03;

/// Start byte index (inclusive) of the packet payload.
pub const DATA_START: usize = 0x04;

/// End byte index (exclusive) of the packet payload.
pub const DATA_END: usize = 0x3f;

/// Byte index of the XOR CRC byte.
pub const CRC: usize = 0x3f;
