#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod flash;
pub mod print;

#[cfg(feature = "std")]
pub mod file;
#[cfg(feature = "std")]
pub mod keys;

use core::fmt;
use core::sync::atomic::{Ordering, compiler_fence};
#[cfg(feature = "serde")]
pub use file::JsonReader;
#[cfg(feature = "std")]
pub use file::{BinReader, BinWriter};
#[cfg(feature = "std")]
pub use keys::{KeyError, Keys, SecureKey};
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug)]
pub struct Input {
    pub serial_number: std::string::String,
    pub device_id: std::string::String,
    pub wifi_ssid: std::string::String,
    pub wifi_password: std::string::String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigField<const N: usize>([u8; N]);

#[derive(Debug, Clone, PartialEq)]
struct SerialNumber(ConfigField<32>);

#[derive(Debug, Clone, PartialEq)]
struct DeviceIdentifier(ConfigField<32>);

#[derive(Debug, Clone, PartialEq)]
struct WifiSSID(ConfigField<32>);

#[derive(Debug, Clone, PartialEq)]
struct WifiPassword(ConfigField<64>);

#[derive(Debug, Clone, PartialEq)]
struct Checksum(u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Raw;
#[derive(Debug, Clone, PartialEq)]
pub struct Verified;

#[repr(C, align(4))]
#[derive(Debug, Clone, PartialEq)]
pub struct Config<S = Verified> {
    serial_number: SerialNumber,
    device_id: DeviceIdentifier,
    wifi_ssid: WifiSSID,
    wifi_password: WifiPassword,
    checksum: Checksum,
    _state: core::marker::PhantomData<S>,
}

#[derive(Default)]
pub struct ConfigBuilder<'a> {
    serial_number: Option<&'a str>,
    device_id: Option<&'a str>,
    wifi_ssid: Option<&'a str>,
    wifi_password: Option<&'a str>,
}

pub trait DeviceConfig {
    fn get_serial_number(&self) -> Result<&str, ConfigError>;
    fn get_device_id(&self) -> Result<&str, ConfigError>;
}

pub trait NetworkConfig {
    fn get_wifi_ssid(&self) -> Result<&str, ConfigError>;
    fn get_wifi_password(&self) -> Result<&str, ConfigError>;
}

#[cfg(feature = "std")]
impl std::error::Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> core::fmt::Result {
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

impl Default for Config<Verified> {
    fn default() -> Self {
        let mut config = Self {
            serial_number: SerialNumber(ConfigField::new()),
            device_id: DeviceIdentifier(ConfigField::new()),
            wifi_ssid: WifiSSID(ConfigField::new()),
            wifi_password: WifiPassword(ConfigField::new()),
            checksum: Checksum(0),
            _state: core::marker::PhantomData,
        };

        let _ = config.serial_number.0.copy_from_slice(b"sn-0000-0000");
        let _ = config.device_id.0.copy_from_slice(b"rp235x");
        let _ = config.wifi_ssid.0.copy_from_slice(b"my_ssid");
        let _ = config.wifi_password.0.copy_from_slice(b"secret_password");

        config.update_checksum();
        config
    }
}

impl<S> Drop for Config<S> {
    fn drop(&mut self) {
        use core::sync::atomic::{Ordering, compiler_fence};

        self.zeroize();

        compiler_fence(Ordering::SeqCst);
    }
}

impl DeviceConfig for Config<Verified> {
    fn get_serial_number(&self) -> Result<&str, ConfigError> {
        self.serial_number.0.parse_str()
    }

    fn get_device_id(&self) -> Result<&str, ConfigError> {
        self.device_id.0.parse_str()
    }
}

impl NetworkConfig for Config {
    fn get_wifi_ssid(&self) -> Result<&str, ConfigError> {
        self.wifi_ssid.0.parse_str()
    }

    fn get_wifi_password(&self) -> Result<&str, ConfigError> {
        self.wifi_password.0.parse_str()
    }
}

impl<const N: usize> ConfigField<N> {
    const fn new() -> Self {
        Self([0; N])
    }

    #[inline]
    fn copy_from_slice(&mut self, src: &[u8]) -> Result<(), ConfigError> {
        if src.len() >= N {
            return Err(ConfigError::StringTooLong);
        }

        self.0.fill(0);
        self.0[..src.len()].copy_from_slice(src);

        Ok(())
    }

    #[inline]
    fn parse_str(&self) -> Result<&str, ConfigError> {
        let length = self
            .0
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(ConfigError::Serialization)?;

        core::str::from_utf8(&self.0[..length]).map_err(|_| ConfigError::InvalidEncoding)
    }

    #[inline]
    fn zeroize(&mut self) {
        self.0.fill(0);
    }
}

impl<S> Config<S> {
    pub fn new(
        serial_number: &str,
        device_id: &str,
        wifi_ssid: &str,
        wifi_password: &str,
    ) -> Result<Self, ConfigError> {
        let mut config = Self {
            serial_number: SerialNumber(ConfigField::new()),
            device_id: DeviceIdentifier(ConfigField::new()),
            wifi_ssid: WifiSSID(ConfigField::new()),
            wifi_password: WifiPassword(ConfigField::new()),
            checksum: Checksum(0),
            _state: core::marker::PhantomData,
        };

        if let Err(e) = config
            .serial_number
            .0
            .copy_from_slice(serial_number.as_bytes())
        {
            config.zeroize();
            return Err(e);
        }
        if let Err(e) = config.device_id.0.copy_from_slice(device_id.as_bytes()) {
            config.zeroize();
            return Err(e);
        }
        if let Err(e) = config.wifi_ssid.0.copy_from_slice(wifi_ssid.as_bytes()) {
            config.zeroize();
            return Err(e);
        }
        if let Err(e) = config
            .wifi_password
            .0
            .copy_from_slice(wifi_password.as_bytes())
        {
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

    #[inline(always)]
    fn calculate_checksum(data: &[u8]) -> Checksum {
        let mut acc: u64 = 0x517cc1b727220a95;

        for &byte in data {
            acc = acc.wrapping_mul(31).wrapping_add(byte as u64);
        }

        Checksum((acc ^ (acc >> 32)) as u32)
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

        let mut diff = computed.0 ^ self.checksum.0;
        diff |= diff >> 16;
        diff |= diff >> 8;
        diff |= diff >> 4;
        diff |= diff >> 2;
        diff |= diff >> 1;

        (diff & 1) == 0
    }

    fn zeroize(&mut self) {
        self.serial_number.0.zeroize();
        self.device_id.0.zeroize();
        self.wifi_ssid.0.zeroize();
        self.wifi_password.0.zeroize();
        self.checksum = Checksum(0);

        compiler_fence(Ordering::SeqCst);
    }
}

impl Config<Raw> {
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ConfigError> {
        if bytes.len() < CONFIG_SIZE {
            return Err(ConfigError::Serialization);
        }

        let ptr = bytes.as_ptr();
        if !(ptr as usize).is_multiple_of(core::mem::align_of::<Self>()) {
            return Err(ConfigError::Serialization);
        }

        let config_ref = unsafe { &*(ptr as *const Self) };

        Ok(config_ref)
    }

    pub fn verify(&self) -> Result<&Config<Verified>, ConfigError> {
        if !self.verify_integrity() {
            return Err(ConfigError::IntegrityCheckFailed);
        }

        let verified_ref = unsafe { &*(self as *const Config<Raw> as *const Config<Verified>) };

        Ok(verified_ref)
    }
}

impl<'a> ConfigBuilder<'a> {
    pub fn serial_number(&mut self, serial_number: &'a str) -> &mut Self {
        self.serial_number = Some(serial_number);

        self
    }

    pub fn device_id(&mut self, device_id: &'a str) -> &mut Self {
        self.device_id = Some(device_id);

        self
    }

    pub fn wifi_ssid(&mut self, wifi_ssid: &'a str) -> &mut Self {
        self.wifi_ssid = Some(wifi_ssid);

        self
    }

    pub fn wifi_password(&mut self, wifi_password: &'a str) -> &mut Self {
        self.wifi_password = Some(wifi_password);

        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.serial_number = None;
        self.device_id = None;
        self.wifi_ssid = None;
        self.wifi_password = None;

        self
    }

    pub fn build(&self) -> Result<Config<Verified>, ConfigError> {
        Config::<Verified>::new(
            self.serial_number.ok_or(ConfigError::Serialization)?,
            self.device_id.ok_or(ConfigError::Serialization)?,
            self.wifi_ssid.ok_or(ConfigError::Serialization)?,
            self.wifi_password.ok_or(ConfigError::Serialization)?,
        )
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
    fn test_config_builder() {
        let mut builder = ConfigBuilder::default();
        let config = builder
            .serial_number("sn-builder-01")
            .device_id("rp235x")
            .wifi_ssid("test_ssid")
            .wifi_password("test_password")
            .build()
            .unwrap();

        assert!(config.verify_integrity());
        assert_eq!(config.get_serial_number().unwrap(), "sn-builder-01");
        assert_eq!(config.get_device_id().unwrap(), "rp235x");
        assert_eq!(config.get_wifi_ssid().unwrap(), "test_ssid");
        assert_eq!(config.get_wifi_password().unwrap(), "test_password");
    }

    #[test]
    fn test_string_too_long() {
        let long_serial = "a".repeat(32);
        let config = Config::<Verified>::new(&long_serial, "rp235x", "my_ssid", "secret_password");

        assert_eq!(config, Err(ConfigError::StringTooLong));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = Config::default();
        let bytes = config.as_bytes();

        let raw_config = Config::<Raw>::from_bytes(bytes).unwrap();
        let parsed = raw_config.verify().unwrap();

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

        let raw_config = Config::<Raw>::from_bytes(&bad_bytes).unwrap();
        let result = raw_config.verify();

        assert_eq!(result, Err(ConfigError::IntegrityCheckFailed));
    }

    #[test]
    fn test_buffer_too_short() {
        let short_bytes = [0u8; 10];
        let config = Config::<Raw>::from_bytes(&short_bytes);

        assert_eq!(config, Err(ConfigError::Serialization));
    }
}
