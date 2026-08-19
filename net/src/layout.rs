pub const DATA_SIZE: usize = 27;
pub const PACKET_SIZE: usize = 32;

pub const MAGIC: usize = 0x00;
pub const EVENT: usize = 0x01;
pub const SEQUENCE_ID: usize = 0x02;
pub const VERSION: usize = 0x03;
pub const DATA_START: usize = 0x04;
pub const DATA_END: usize = 0x1F;
pub const CRC: usize = 0x1F;
