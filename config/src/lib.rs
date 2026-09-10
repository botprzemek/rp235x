#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::fmt::{Display, Formatter};
use core::sync::atomic::{Ordering, compiler_fence};

#[cfg(feature = "std")]
pub mod file;
#[cfg(feature = "std")]
pub use file::{FileReader, FileWriter};

pub mod print;
pub use print::Print;

pub const CONFIG_SIZE: usize = core::mem::size_of::<Config>();
pub const CONFIG_ADDR: usize = 0x1003F000;

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    FileNotFound,
    InvalidEncoding,
    IntegrityCheckFailed,
    #[cfg(feature = "std")]
    Io(std::io::ErrorKind),
    Serialization,
    StringTooLong,
}

#[cfg(feature = "std")]
impl std::error::Error for ConfigError {}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FileNotFound => write!(f, "File not found"),
            Self::InvalidEncoding => write!(f, "String slice contains invalid UTF-8 sequence"),
            Self::IntegrityCheckFailed => write!(f, "Memory checksum validation failed"),
            #[cfg(feature = "std")]
            Self::Io(error) => write!(f, "I/O Error: {}", error),
            Self::Serialization => write!(f, "Serialization/Deserialization failed"),
            Self::StringTooLong => write!(f, "Input string exceeds maximum buffer capacity"),
        }
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => Self::FileNotFound,
            _ => Self::Io(error.kind()),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug)]
pub struct ConfigInput {
    pub serial_number: std::string::String,
    pub device_id: std::string::String,
    pub wifi_ssid: std::string::String,
    pub wifi_password: std::string::String,
}

#[repr(C, align(4))]
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    serial_number: [u8; 32],
    device_id: [u8; 32],
    wifi_ssid: [u8; 32],
    wifi_password: [u8; 64],
    checksum: u32,
}

pub trait DeviceConfig {
    fn get_serial_number(&self) -> Result<&str, ConfigError>;
    fn get_device_id(&self) -> Result<&str, ConfigError>;
}

pub trait NetworkConfig {
    fn get_wifi_ssid(&self) -> Result<&str, ConfigError>;
    fn get_wifi_password(&self) -> Result<&str, ConfigError>;
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Self {
            serial_number: [0; 32],
            device_id: [0; 32],
            wifi_ssid: [0; 32],
            wifi_password: [0; 64],
            checksum: 0,
        };

        let _ = Self::copy_to_array(b"sn-0000-0000", &mut config.serial_number);
        let _ = Self::copy_to_array(b"rp235x", &mut config.device_id);
        let _ = Self::copy_to_array(b"my_ssid", &mut config.wifi_ssid);
        let _ = Self::copy_to_array(b"secret_password", &mut config.wifi_password);

        config.update_checksum();
        config
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        use core::sync::atomic::{Ordering, compiler_fence};

        self.zeroize();

        compiler_fence(Ordering::SeqCst);
    }
}

impl DeviceConfig for Config {
    fn get_serial_number(&self) -> Result<&str, ConfigError> {
        Self::parse_str(&self.serial_number)
    }

    fn get_device_id(&self) -> Result<&str, ConfigError> {
        Self::parse_str(&self.device_id)
    }
}

impl NetworkConfig for Config {
    fn get_wifi_ssid(&self) -> Result<&str, ConfigError> {
        Self::parse_str(&self.wifi_ssid)
    }

    fn get_wifi_password(&self) -> Result<&str, ConfigError> {
        Self::parse_str(&self.wifi_password)
    }
}

impl Config {
    pub fn new(
        serial: &str,
        device: &str,
        ssid: &str,
        password: &str,
    ) -> Result<Self, ConfigError> {
        let mut config = Self {
            serial_number: [0; 32],
            device_id: [0; 32],
            wifi_ssid: [0; 32],
            wifi_password: [0; 64],
            checksum: 0,
        };

        if let Err(e) = Self::copy_to_array(serial.as_bytes(), &mut config.serial_number) {
            config.zeroize();
            return Err(e);
        }
        if let Err(e) = Self::copy_to_array(device.as_bytes(), &mut config.device_id) {
            config.zeroize();
            return Err(e);
        }
        if let Err(e) = Self::copy_to_array(ssid.as_bytes(), &mut config.wifi_ssid) {
            config.zeroize();
            return Err(e);
        }
        if let Err(e) = Self::copy_to_array(password.as_bytes(), &mut config.wifi_password) {
            config.zeroize();
            return Err(e);
        }

        config.update_checksum();
        Ok(config)
    }

    pub fn as_bytes(&self) -> &[u8] {
        use core::slice;

        unsafe { slice::from_raw_parts((self as *const Self) as *const u8, CONFIG_SIZE) }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ConfigError> {
        if bytes.len() < CONFIG_SIZE {
            return Err(ConfigError::Serialization);
        }

        let ptr = bytes.as_ptr();
        if !(ptr as usize).is_multiple_of(core::mem::align_of::<Self>()) {
            return Err(ConfigError::Serialization);
        }

        let config_ref = unsafe { &*(ptr as *const Self) };

        if !config_ref.verify_integrity() {
            return Err(ConfigError::IntegrityCheckFailed);
        }

        Ok(config_ref)
    }

    pub fn read() -> Result<&'static Self, ConfigError> {
        unsafe {
            let bytes = core::slice::from_raw_parts(CONFIG_ADDR as *const u8, CONFIG_SIZE);

            Self::from_bytes(bytes)
        }
    }

    #[inline(always)]
    fn calculate_checksum(data: &[u8]) -> u32 {
        let mut acc: u64 = 0x517cc1b727220a95;

        for &byte in data {
            acc = acc.wrapping_mul(31).wrapping_add(byte as u64);
        }

        (acc ^ (acc >> 32)) as u32
    }

    #[inline]
    fn copy_to_array(src: &[u8], dest: &mut [u8]) -> Result<(), ConfigError> {
        if src.len() >= dest.len() {
            return Err(ConfigError::StringTooLong);
        }

        dest.fill(0);
        dest[..src.len()].copy_from_slice(src);

        Ok(())
    }

    #[inline]
    fn parse_str(bytes: &[u8]) -> Result<&str, ConfigError> {
        let len = bytes
            .iter()
            .position(|&b| b == 0)
            .ok_or(ConfigError::Serialization)?;
        core::str::from_utf8(&bytes[..len]).map_err(|_| ConfigError::InvalidEncoding)
    }

    fn update_checksum(&mut self) {
        let slice_len = CONFIG_SIZE - core::mem::size_of::<u32>();
        let ptr = (self as *const Self) as *const u8;
        let data = unsafe { core::slice::from_raw_parts(ptr, slice_len) };

        self.checksum = Self::calculate_checksum(data);
    }

    pub fn verify_integrity(&self) -> bool {
        let slice_len = CONFIG_SIZE - core::mem::size_of::<u32>();
        let ptr = (self as *const Self) as *const u8;
        let data = unsafe { core::slice::from_raw_parts(ptr, slice_len) };

        let computed = Self::calculate_checksum(data);

        let mut diff = computed ^ self.checksum;
        diff |= diff >> 16;
        diff |= diff >> 8;
        diff |= diff >> 4;
        diff |= diff >> 2;
        diff |= diff >> 1;

        (diff & 1) == 0
    }

    fn zeroize(&mut self) {
        self.serial_number.fill(0);
        self.device_id.fill(0);
        self.wifi_ssid.fill(0);
        self.wifi_password.fill(0);
        self.checksum = 0;

        compiler_fence(Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert!(config.verify_integrity());
        assert_eq!(config.get_serial_number().unwrap(), "sn-0000-0000");
        assert_eq!(config.get_device_id().unwrap(), "rp235x");
        assert_eq!(config.get_wifi_ssid().unwrap(), "my_ssid");
        assert_eq!(config.get_wifi_password().unwrap(), "secret_password");
    }

    #[test]
    fn test_new_config() {
        let config = Config::new("sn-0000-0001", "device", "other_ssid", "super_secret").unwrap();

        assert!(config.verify_integrity());
        assert_eq!(config.get_serial_number().unwrap(), "sn-0000-0001");
        assert_eq!(config.get_device_id().unwrap(), "device");
        assert_eq!(config.get_wifi_ssid().unwrap(), "other_ssid");
        assert_eq!(config.get_wifi_password().unwrap(), "super_secret");
    }

    #[test]
    fn test_string_too_long() {
        let long_serial = "a".repeat(32);
        let config = Config::new(&long_serial, "rp235x", "my_ssid", "secret_password");

        assert_eq!(config, Err(ConfigError::StringTooLong));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = Config::default();
        let bytes = config.as_bytes();
        let parsed = Config::from_bytes(bytes).unwrap();

        assert!(parsed.verify_integrity());
        assert_eq!(parsed.get_serial_number().unwrap(), "sn-0000-0000");
        assert_eq!(parsed.get_device_id().unwrap(), "rp235x");
        assert_eq!(parsed.get_wifi_ssid().unwrap(), "my_ssid");
        assert_eq!(parsed.get_wifi_password().unwrap(), "secret_password");
    }

    #[test]
    fn test_integrity_failure() {
        let config = Config::default();
        let bytes = config.as_bytes();

        let mut bad_bytes = [0u8; CONFIG_SIZE];
        bad_bytes.copy_from_slice(bytes);
        bad_bytes[0] ^= 0xFF;

        let config = Config::from_bytes(&bad_bytes);

        assert_eq!(config, Err(ConfigError::IntegrityCheckFailed));
    }

    #[test]
    fn test_buffer_too_short() {
        let short_bytes = [0u8; 10];
        let config = Config::from_bytes(&short_bytes);

        assert_eq!(config, Err(ConfigError::Serialization));
    }
}
