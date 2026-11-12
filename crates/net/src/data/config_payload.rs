//! Device configuration payload for over-the-air Wi-Fi network setup over UDP.

use core::str;

use config::ConfigError;

/// Layout offsets for [`DeviceConfigPayload`].
pub(crate) mod layout {
    pub(crate) const SERIAL_NUMBER_SIZE: usize = 12;
    pub(crate) const DEVICE_ID_SIZE: usize = 12;
    pub(crate) const WIFI_SSID_SIZE: usize = 14;
    pub(crate) const WIFI_PASSWORD_SIZE: usize = 14;

    pub(crate) const SERIAL_NUMBER_START: usize = 0;
    pub(crate) const SERIAL_NUMBER_END: usize = 12;
    pub(crate) const DEVICE_ID_START: usize = 12;
    pub(crate) const DEVICE_ID_END: usize = 24;
    pub(crate) const WIFI_SSID_START: usize = 24;
    pub(crate) const WIFI_SSID_END: usize = 38;
    pub(crate) const WIFI_PASSWORD_START: usize = 38;
    pub(crate) const WIFI_PASSWORD_END: usize = 52;
}

/// 52-byte payload structure mapping exactly to [`crate::layout::DATA_SIZE`].
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceConfigPayload {
    serial_number: [u8; layout::SERIAL_NUMBER_SIZE],
    device_id: [u8; layout::DEVICE_ID_SIZE],
    wifi_ssid: [u8; layout::WIFI_SSID_SIZE],
    wifi_password: [u8; layout::WIFI_PASSWORD_SIZE],
}

impl DeviceConfigPayload {
    /// Creates a new [`DeviceConfigPayload`] from string slices.
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

    /// Serializes this payload into a 59-byte array.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; crate::layout::DATA_SIZE] {
        let mut bytes = [0u8; crate::layout::DATA_SIZE];
        bytes[layout::SERIAL_NUMBER_START..layout::SERIAL_NUMBER_END]
            .copy_from_slice(&self.serial_number);
        bytes[layout::DEVICE_ID_START..layout::DEVICE_ID_END].copy_from_slice(&self.device_id);
        bytes[layout::WIFI_SSID_START..layout::WIFI_SSID_END].copy_from_slice(&self.wifi_ssid);
        bytes[layout::WIFI_PASSWORD_START..layout::WIFI_PASSWORD_END]
            .copy_from_slice(&self.wifi_password);
        bytes
    }

    /// Deserializes a [`DeviceConfigPayload`] from a 59-byte array.
    #[must_use]
    pub fn from_bytes(bytes: [u8; crate::layout::DATA_SIZE]) -> Self {
        let mut sn_buf = [0u8; layout::SERIAL_NUMBER_SIZE];
        let mut id_buf = [0u8; layout::DEVICE_ID_SIZE];
        let mut ssid_buf = [0u8; layout::WIFI_SSID_SIZE];
        let mut pass_buf = [0u8; layout::WIFI_PASSWORD_SIZE];

        sn_buf.copy_from_slice(&bytes[layout::SERIAL_NUMBER_START..layout::SERIAL_NUMBER_END]);
        id_buf.copy_from_slice(&bytes[layout::DEVICE_ID_START..layout::DEVICE_ID_END]);
        ssid_buf.copy_from_slice(&bytes[layout::WIFI_SSID_START..layout::WIFI_SSID_END]);
        pass_buf.copy_from_slice(&bytes[layout::WIFI_PASSWORD_START..layout::WIFI_PASSWORD_END]);

        Self {
            serial_number: sn_buf,
            device_id: id_buf,
            wifi_ssid: ssid_buf,
            wifi_password: pass_buf,
        }
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
        if bytes.len() > dest.len() {
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
