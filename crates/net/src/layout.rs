pub const HEADER_SIZE: usize = 10;
pub const DATA_SIZE: usize = 52;
pub const PACKET_SIZE: usize = 64;

pub(crate) const MAGIC: usize = 0x00;
pub(crate) const VERSION: usize = 0x01;
pub(crate) const EVENT: usize = 0x02;

pub(crate) const SEQUENCE_ID_START: usize = 0x03;
pub(crate) const SEQUENCE_ID_END: usize = 0x05;

pub(crate) const SESSION_TOKEN_START: usize = 0x05;
pub(crate) const SESSION_TOKEN_END: usize = 0x09;

pub(crate) const PAYLOAD_LEN: usize = 0x09;

pub(crate) const DATA_START: usize = 0x0a;
pub(crate) const DATA_END: usize = 0x3e;

pub(crate) const CRC_START: usize = 0x3e;
pub(crate) const CRC_END: usize = 0x40;
