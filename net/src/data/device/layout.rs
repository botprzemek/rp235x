pub const DATA_SIZE: usize = crate::layout::DATA_SIZE;
pub const SERIAL_NUMBER_SIZE: usize = 12;
pub const DEVICE_ID_SIZE: usize = 12;

pub const DEVICE_TYPE: usize = 0x00;
pub const GAME_ID_START: usize = 0x01;
pub const GAME_ID_END: usize = 0x03;
pub const SERIAL_NUMBER_START: usize = 0x03;
pub const SERIAL_NUMBER_END: usize = 0x0F;
pub const DEVICE_ID_START: usize = 0x0F;
pub const DEVICE_ID_END: usize = 0x1B;
