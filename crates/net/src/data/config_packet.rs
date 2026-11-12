//! Dedicated over-the-air Wi-Fi configuration packet over UDP.
//!
//! Accommodates full standard-compliant field capacities:
//! - 32-byte Serial Number
//! - 32-byte Device ID
//! - 32-byte Wi-Fi SSID
//! - 64-byte Wi-Fi Password (WPA2-PSK)

use core::str;

use config::ConfigError;

/// Magic header byte for [`DeviceConfigPacket`].
pub const CONFIG_PACKET_MAGIC: u8 = 0x43; // 'C'
/// Protocol version byte.
pub const CONFIG_PACKET_VERSION: u8 = 0x01;
/// Total size in bytes of a [`DeviceConfigPacket`] on the wire (168 bytes payload + 4 header + 1 CRC = 173 bytes).
pub const CONFIG_PACKET_SIZE: usize = 173;

/// Field capacities matching [`config::Config`].
pub(crate) mod layout {
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
}

/// Dedicated wire packet for configuring RP2350 Wi-Fi parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceConfigPacket {
    serial_number: [u8; layout::SERIAL_NUMBER_SIZE],
    device_id: [u8; layout::DEVICE_ID_SIZE],
    wifi_ssid: [u8; layout::WIFI_SSID_SIZE],
    wifi_password: [u8; layout::WIFI_PASSWORD_SIZE],
}

impl DeviceConfigPacket {
    /// Creates a new [`DeviceConfigPacket`].
    pub fn try_new(
        serial_number: &str,
        device_id: &str,
        wifi_ssid: &str,
        wifi_password: &str,
    ) -> Result<Self, ConfigError> {
        let mut sn_buf = [0u8; layout::SERIAL_NUMBER_SIZE];
        let mut id_buf = [0u8; layout::DEVICE_ID_SIZE];
        let mut ssid_buf = [0u8; layout::WIFI_SSID_SIZE];
        let mut pass_buf = [0u8; layout::WIFI_PASSWORD_SIZE];

        Self::copy_field(&mut sn_buf, serial_number)?;
        Self::copy_field(&mut id_buf, device_id)?;
        Self::copy_field(&mut ssid_buf, wifi_ssid)?;
        Self::copy_field(&mut pass_buf, wifi_password)?;

        Ok(Self {
            serial_number: sn_buf,
            device_id: id_buf,
            wifi_ssid: ssid_buf,
            wifi_password: pass_buf,
        })
    }

    /// Serializes this packet into a wire byte array of size [`layout::TOTAL_SIZE`].
    #[must_use]
    pub fn to_bytes(&self) -> [u8; layout::TOTAL_SIZE] {
        let mut bytes = [0u8; layout::TOTAL_SIZE];
        bytes[layout::MAGIC] = CONFIG_PACKET_MAGIC;
        bytes[layout::EVENT] = 0x42; // DeviceConfigSet
        bytes[layout::SEQUENCE_ID] = 0;
        bytes[layout::VERSION] = CONFIG_PACKET_VERSION;

        bytes[layout::SERIAL_NUMBER_START..layout::SERIAL_NUMBER_END]
            .copy_from_slice(&self.serial_number);
        bytes[layout::DEVICE_ID_START..layout::DEVICE_ID_END].copy_from_slice(&self.device_id);
        bytes[layout::WIFI_SSID_START..layout::WIFI_SSID_END].copy_from_slice(&self.wifi_ssid);
        bytes[layout::WIFI_PASSWORD_START..layout::WIFI_PASSWORD_END]
            .copy_from_slice(&self.wifi_password);

        let mut crc = CONFIG_PACKET_MAGIC ^ 0x42 ^ CONFIG_PACKET_VERSION;
        for &b in &bytes[4..164] {
            crc ^= b;
        }
        bytes[layout::CRC] = crc;

        bytes
    }

    /// Deserializes and validates a [`DeviceConfigPacket`] from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != layout::TOTAL_SIZE {
            return Err("invalid_size");
        }
        if bytes[layout::MAGIC] != CONFIG_PACKET_MAGIC {
            return Err("magic");
        }
        if bytes[layout::EVENT] != 0x42 {
            return Err("event");
        }

        let mut expected_crc = CONFIG_PACKET_MAGIC
            ^ bytes[layout::EVENT]
            ^ bytes[layout::SEQUENCE_ID]
            ^ bytes[layout::VERSION];
        for &b in &bytes[4..164] {
            expected_crc ^= b;
        }
        if bytes[layout::CRC] != expected_crc {
            return Err("crc");
        }

        let mut sn_buf = [0u8; layout::SERIAL_NUMBER_SIZE];
        let mut id_buf = [0u8; layout::DEVICE_ID_SIZE];
        let mut ssid_buf = [0u8; layout::WIFI_SSID_SIZE];
        let mut pass_buf = [0u8; layout::WIFI_PASSWORD_SIZE];

        sn_buf.copy_from_slice(&bytes[layout::SERIAL_NUMBER_START..layout::SERIAL_NUMBER_END]);
        id_buf.copy_from_slice(&bytes[layout::DEVICE_ID_START..layout::DEVICE_ID_END]);
        ssid_buf.copy_from_slice(&bytes[layout::WIFI_SSID_START..layout::WIFI_SSID_END]);
        pass_buf.copy_from_slice(&bytes[layout::WIFI_PASSWORD_START..layout::WIFI_PASSWORD_END]);

        Ok(Self {
            serial_number: sn_buf,
            device_id: id_buf,
            wifi_ssid: ssid_buf,
            wifi_password: pass_buf,
        })
    }

    /// Returns the serial number string slice.
    pub fn serial_number(&self) -> Result<&str, ConfigError> {
        Self::parse_str(&self.serial_number)
    }

    /// Returns the device ID string slice.
    pub fn device_id(&self) -> Result<&str, ConfigError> {
        Self::parse_str(&self.device_id)
    }

    /// Returns the Wi-Fi SSID string slice.
    pub fn wifi_ssid(&self) -> Result<&str, ConfigError> {
        Self::parse_str(&self.wifi_ssid)
    }

    /// Returns the Wi-Fi password string slice.
    pub fn wifi_password(&self) -> Result<&str, ConfigError> {
        Self::parse_str(&self.wifi_password)
    }

    fn copy_field(dest: &mut [u8], src: &str) -> Result<(), ConfigError> {
        let bytes = src.as_bytes();
        if bytes.len() >= dest.len() {
            return Err(ConfigError::StringTooLong);
        }
        dest.fill(0);
        dest[..bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    fn parse_str(buf: &[u8]) -> Result<&str, ConfigError> {
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        str::from_utf8(&buf[..len]).map_err(|_| ConfigError::InvalidEncoding)
    }
}
