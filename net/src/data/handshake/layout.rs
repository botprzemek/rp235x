//! Byte offsets and sizes for the [`super::Handshake`] payload.

/// Total handshake payload size in bytes.
pub const DATA_SIZE: usize = 6;

/// Byte index of the protocol version.
pub const PROTOCOL_VERSION: usize = 0x00;

/// Start byte index (inclusive) of the device ID.
pub const DEVICE_ID_START: usize = 0x01;

/// End byte index (exclusive) of the device ID.
pub const DEVICE_ID_END: usize = 0x05;

/// Byte index of the handshake flags.
pub const FLAGS: usize = 0x05;
