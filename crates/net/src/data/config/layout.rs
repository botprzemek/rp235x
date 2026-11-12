pub(crate) const SERIAL_NUMBER_SIZE: usize = 32;
pub(crate) const DEVICE_ID_SIZE: usize = 32;
pub(crate) const WIFI_SSID_SIZE: usize = 32;
pub(crate) const WIFI_PASSWORD_SIZE: usize = 64;

pub(crate) const MAGIC: usize = 0;
pub(crate) const EVENT: usize = 1;
pub(crate) const SEQUENCE_ID: usize = 2;
pub(crate) const VERSION: usize = 3;

pub(crate) const SERIAL_NUMBER_START: usize = 4;
pub(crate) const SERIAL_NUMBER_END: usize = 36;
pub(crate) const DEVICE_ID_START: usize = 36;
pub(crate) const DEVICE_ID_END: usize = 68;
pub(crate) const WIFI_SSID_START: usize = 68;
pub(crate) const WIFI_SSID_END: usize = 100;
pub(crate) const WIFI_PASSWORD_START: usize = 100;
pub(crate) const WIFI_PASSWORD_END: usize = 164;

pub(crate) const CRC: usize = 164;
pub(crate) const TOTAL_SIZE: usize = 165;
