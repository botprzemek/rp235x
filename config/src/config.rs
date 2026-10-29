//! Core configuration types, builders, and memory integrity checks.

use core::fmt;
use core::marker::PhantomData;
use core::mem;
use core::slice;
use core::str;
use core::sync::atomic::{self, Ordering};

/// Size of the [`Config`] structure in bytes.
pub const CONFIG_SIZE: usize = mem::size_of::<Config>();

/// Flash memory address where the device configuration is stored on RP235x.
pub const CONFIG_ADDR: usize = 0x1003_f000;

/// Errors that can occur when reading, writing, or validating a [`Config`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    FileNotFound,
    InvalidEncoding,
    IntegrityCheckFailed,
    #[cfg(feature = "std")]
    Io(std::io::ErrorKind),
    Serialization,
    StringTooLong,
}

/// Raw JSON-serializable input used to construct a [`Config`].
#[cfg(feature = "std")]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub serial_number: std::string::String,
    pub device_id: std::string::String,
    pub wifi_ssid: std::string::String,
    pub wifi_password: std::string::String,
}

/// Fixed-capacity null-terminated UTF-8 field inside [`Config`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigField<const N: usize>([u8; N]);

#[derive(Debug, Clone, PartialEq, Eq)]
struct SerialNumber(ConfigField<32>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeviceIdentifier(ConfigField<32>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct WifiSsid(ConfigField<32>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct WifiPassword(ConfigField<64>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Checksum(u32);

/// Typestate marker for an unverified [`Config`] loaded from raw memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Raw;

/// Typestate marker for a [`Config`] whose checksum integrity has been verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verified;

/// Device and network configuration block stored in flash or binary files.
#[repr(C, align(4))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config<State = Verified> {
    serial_number: SerialNumber,
    device_id: DeviceIdentifier,
    wifi_ssid: WifiSsid,
    wifi_password: WifiPassword,
    checksum: Checksum,
    _state: PhantomData<State>,
}

/// Builder for constructing a verified [`Config`].
#[derive(Default)]
pub struct ConfigBuilder<'input> {
    serial_number: Option<&'input str>,
    device_id: Option<&'input str>,
    wifi_ssid: Option<&'input str>,
    wifi_password: Option<&'input str>,
}

/// Accessors for device identity fields.
pub trait DeviceConfig {
    /// Returns the device's serial number.
    fn serial_number(&self) -> Result<&str, ConfigError>;

    /// Returns the device's identifier.
    fn device_id(&self) -> Result<&str, ConfigError>;
}

/// Accessors for wireless network credentials.
pub trait NetworkConfig {
    /// Returns the configured Wi-Fi SSID.
    fn wifi_ssid(&self) -> Result<&str, ConfigError>;

    /// Returns the configured Wi-Fi password.
    fn wifi_password(&self) -> Result<&str, ConfigError>;
}

#[cfg(feature = "std")]
impl std::error::Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound => write!(formatter, "File not found"),
            Self::InvalidEncoding => {
                write!(formatter, "String slice contains invalid UTF-8 sequence")
            }
            Self::IntegrityCheckFailed => write!(formatter, "Memory checksum validation failed"),
            #[cfg(feature = "std")]
            Self::Io(error) => write!(formatter, "I/O Error: {error}"),
            Self::Serialization => write!(formatter, "Serialization/Deserialization failed"),
            Self::StringTooLong => {
                write!(formatter, "Input string exceeds maximum buffer capacity")
            }
        }
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => Self::FileNotFound,
            other => Self::Io(other),
        }
    }
}

impl Default for Config<Verified> {
    fn default() -> Self {
        let mut config = Self {
            serial_number: SerialNumber(ConfigField::new()),
            device_id: DeviceIdentifier(ConfigField::new()),
            wifi_ssid: WifiSsid(ConfigField::new()),
            wifi_password: WifiPassword(ConfigField::new()),
            checksum: Checksum(0),
            _state: PhantomData,
        };

        let _ = config.serial_number.0.copy_from_slice(b"sn-0000-0000");
        let _ = config.device_id.0.copy_from_slice(b"rp235x");
        let _ = config.wifi_ssid.0.copy_from_slice(b"my_ssid");
        let _ = config.wifi_password.0.copy_from_slice(b"secret_password");

        config.update_checksum();
        config
    }
}

impl<State> Drop for Config<State> {
    fn drop(&mut self) {
        self.zeroize();
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

impl DeviceConfig for Config<Verified> {
    fn serial_number(&self) -> Result<&str, ConfigError> {
        self.serial_number.0.parse_str()
    }

    fn device_id(&self) -> Result<&str, ConfigError> {
        self.device_id.0.parse_str()
    }
}

impl NetworkConfig for Config {
    fn wifi_ssid(&self) -> Result<&str, ConfigError> {
        self.wifi_ssid.0.parse_str()
    }

    fn wifi_password(&self) -> Result<&str, ConfigError> {
        self.wifi_password.0.parse_str()
    }
}

impl<const N: usize> Default for ConfigField<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> ConfigField<N> {
    /// Creates a zero-initialized [`ConfigField`].
    #[must_use]
    pub const fn new() -> Self {
        Self([0; N])
    }

    #[inline]
    fn copy_from_slice(&mut self, source: &[u8]) -> Result<(), ConfigError> {
        if source.len() >= N {
            return Err(ConfigError::StringTooLong);
        }

        self.0.fill(0);
        self.0[..source.len()].copy_from_slice(source);

        Ok(())
    }

    #[inline]
    fn parse_str(&self) -> Result<&str, ConfigError> {
        let length = self
            .0
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(ConfigError::Serialization)?;

        str::from_utf8(&self.0[..length]).map_err(|_| ConfigError::InvalidEncoding)
    }

    #[inline]
    fn zeroize(&mut self) {
        self.0.fill(0);
    }
}

impl<State> Config<State> {
    /// Creates a new [`Config`] from the provided credentials and computes its checksum.
    pub fn try_new(
        serial_number: &str,
        device_id: &str,
        wifi_ssid: &str,
        wifi_password: &str,
    ) -> Result<Self, ConfigError> {
        let mut config = Self {
            serial_number: SerialNumber(ConfigField::new()),
            device_id: DeviceIdentifier(ConfigField::new()),
            wifi_ssid: WifiSsid(ConfigField::new()),
            wifi_password: WifiPassword(ConfigField::new()),
            checksum: Checksum(0),
            _state: PhantomData,
        };

        if let Err(error) = config
            .serial_number
            .0
            .copy_from_slice(serial_number.as_bytes())
        {
            config.zeroize();
            return Err(error);
        }
        if let Err(error) = config.device_id.0.copy_from_slice(device_id.as_bytes()) {
            config.zeroize();
            return Err(error);
        }
        if let Err(error) = config.wifi_ssid.0.copy_from_slice(wifi_ssid.as_bytes()) {
            config.zeroize();
            return Err(error);
        }
        if let Err(error) = config
            .wifi_password
            .0
            .copy_from_slice(wifi_password.as_bytes())
        {
            config.zeroize();
            return Err(error);
        }

        config.update_checksum();
        Ok(config)
    }

    /// Returns the raw byte representation of this [`Config`].
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: `Self` is `#[repr(C, align(4))]` with all fields initialized as fixed-size
        // byte arrays and a `u32` checksum, so reading `CONFIG_SIZE` bytes from `self` is valid.
        // Justification: Needed for zero-copy serialization to flash/file storage.
        unsafe { slice::from_raw_parts((self as *const Self).cast::<u8>(), CONFIG_SIZE) }
    }

    #[inline(always)]
    fn calculate_checksum(data: &[u8]) -> Checksum {
        let mut accumulator: u64 = 0x517c_c1b7_2722_0a95;

        for &byte in data {
            accumulator = accumulator.wrapping_mul(31).wrapping_add(u64::from(byte));
        }

        Checksum((accumulator ^ (accumulator >> 32)) as u32)
    }

    fn update_checksum(&mut self) {
        let slice_len = CONFIG_SIZE - mem::size_of::<u32>();
        let pointer = (self as *const Self).cast::<u8>();
        // SAFETY: `pointer` points to a valid `Self` instance and `slice_len` covers the initialized
        // fields preceding `checksum`.
        // Justification: Computes checksum directly over the `#[repr(C)]` struct layout without copying.
        let data = unsafe { slice::from_raw_parts(pointer, slice_len) };

        self.checksum = Self::calculate_checksum(data);
    }

    /// Verifies that the stored checksum matches the calculated checksum of the fields.
    #[must_use]
    pub fn verify_integrity(&self) -> bool {
        let slice_len = CONFIG_SIZE - mem::size_of::<u32>();
        let pointer = (self as *const Self).cast::<u8>();
        // SAFETY: `pointer` points to a valid `Self` instance and `slice_len` covers the initialized
        // fields preceding `checksum`.
        // Justification: Computes checksum directly over the `#[repr(C)]` struct layout without copying.
        let data = unsafe { slice::from_raw_parts(pointer, slice_len) };

        let computed = Self::calculate_checksum(data);

        let mut difference = computed.0 ^ self.checksum.0;
        difference |= difference >> 16;
        difference |= difference >> 8;
        difference |= difference >> 4;
        difference |= difference >> 2;
        difference |= difference >> 1;

        (difference & 1) == 0
    }

    fn zeroize(&mut self) {
        self.serial_number.0.zeroize();
        self.device_id.0.zeroize();
        self.wifi_ssid.0.zeroize();
        self.wifi_password.0.zeroize();
        self.checksum = Checksum(0);

        atomic::compiler_fence(Ordering::SeqCst);
    }
}

impl Config<Raw> {
    /// Reinterprets a properly aligned byte slice of at least [`CONFIG_SIZE`] bytes as a [`Config<Raw>`].
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, ConfigError> {
        if bytes.len() < CONFIG_SIZE {
            return Err(ConfigError::Serialization);
        }

        let pointer = bytes.as_ptr();
        if !(pointer as usize).is_multiple_of(mem::align_of::<Self>()) {
            return Err(ConfigError::Serialization);
        }

        // SAFETY: `bytes` has at least `CONFIG_SIZE` bytes and `pointer` satisfies `Self`'s alignment.
        // All bit patterns of `[u8; N]` and `u32` are valid for `Config<Raw>`.
        // Justification: Zero-copy reading from memory-mapped flash on embedded target.
        let config_ref = unsafe { &*pointer.cast::<Self>() };

        Ok(config_ref)
    }

    /// Verifies checksum integrity and transitions the reference from [`Raw`] to [`Verified`].
    pub fn verify(&self) -> Result<&Config<Verified>, ConfigError> {
        if !self.verify_integrity() {
            return Err(ConfigError::IntegrityCheckFailed);
        }

        // SAFETY: `Config<Raw>` and `Config<Verified>` have identical `#[repr(C, align(4))]` layout,
        // differing only in the zero-sized `PhantomData` marker.
        // Justification: Typestate transition on a borrowed flash reference without copying.
        let verified_ref = unsafe { &*(self as *const Self).cast::<Config<Verified>>() };

        Ok(verified_ref)
    }
}

impl<'input> ConfigBuilder<'input> {
    /// Creates a new empty [`ConfigBuilder`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the serial number.
    pub fn serial_number(&mut self, serial_number: &'input str) -> &mut Self {
        self.serial_number = Some(serial_number);
        self
    }

    /// Sets the device identifier.
    pub fn device_id(&mut self, device_id: &'input str) -> &mut Self {
        self.device_id = Some(device_id);
        self
    }

    /// Sets the Wi-Fi SSID.
    pub fn wifi_ssid(&mut self, wifi_ssid: &'input str) -> &mut Self {
        self.wifi_ssid = Some(wifi_ssid);
        self
    }

    /// Sets the Wi-Fi password.
    pub fn wifi_password(&mut self, wifi_password: &'input str) -> &mut Self {
        self.wifi_password = Some(wifi_password);
        self
    }

    /// Clears all configured fields in the builder.
    pub fn reset(&mut self) -> &mut Self {
        self.serial_number = None;
        self.device_id = None;
        self.wifi_ssid = None;
        self.wifi_password = None;
        self
    }

    /// Builds a verified [`Config`] from the configured fields.
    pub fn build(&self) -> Result<Config<Verified>, ConfigError> {
        Config::<Verified>::try_new(
            self.serial_number.ok_or(ConfigError::Serialization)?,
            self.device_id.ok_or(ConfigError::Serialization)?,
            self.wifi_ssid.ok_or(ConfigError::Serialization)?,
            self.wifi_password.ok_or(ConfigError::Serialization)?,
        )
    }
}

