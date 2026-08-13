#![no_std]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec;
use core::{mem, slice};

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::format;

#[cfg(feature = "std")]
use std::fs;

#[cfg(feature = "std")]
use std::fs::File;

#[cfg(feature = "std")]
use std::io::{Read, Seek, SeekFrom, Write};

#[cfg(feature = "std")]
use std::path::PathBuf;

#[cfg(feature = "std")]
use std::println;

#[cfg(feature = "std")]
use anyhow::{Error, anyhow};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub const CONFIG_OFFSET: u64 = 0x3F000;

#[cfg(feature = "std")]
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigInput {
    pub serial_number: String,
    pub device_id: String,
    pub firmware_version: String,
    pub wifi_ssid: String,
    pub wifi_password: String,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeviceConfig {
    serial_number: [u8; 32],
    device_id: [u8; 32],
    firmware_version: [u8; 16],
    wifi_ssid: [u8; 32],
    wifi_password: [u8; 64],
}

impl Default for DeviceConfig {
    fn default() -> Self {
        let mut config = Self {
            serial_number: [0; 32],
            device_id: [0; 32],
            firmware_version: [0; 16],
            wifi_ssid: [0; 32],
            wifi_password: [0; 64],
        };

        let _ = copy_to_array("sn-0000-0000", &mut config.serial_number);
        let _ = copy_to_array("rp235x", &mut config.device_id);
        let _ = copy_to_array("0.1.0", &mut config.firmware_version);
        let _ = copy_to_array("my_ssid", &mut config.wifi_ssid);
        let _ = copy_to_array("secret_password", &mut config.wifi_password);

        config
    }
}

impl DeviceConfig {
    pub fn new(
        serial: &str,
        device: &str,
        firmware: &str,
        ssid: &str,
        password: &str,
    ) -> Result<Self, &'static str> {
        let mut config = Self {
            serial_number: [0; 32],
            device_id: [0; 32],
            firmware_version: [0; 16],
            wifi_ssid: [0; 32],
            wifi_password: [0; 64],
        };

        copy_to_array(serial, &mut config.serial_number)?;
        copy_to_array(device, &mut config.device_id)?;
        copy_to_array(firmware, &mut config.firmware_version)?;
        copy_to_array(ssid, &mut config.wifi_ssid)?;
        copy_to_array(password, &mut config.wifi_password)?;

        Ok(config)
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts((self as *const Self) as *const u8, mem::size_of::<Self>()) }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < mem::size_of::<Self>() {
            return Err(anyhow!("TODO"));
        }

        let config = unsafe { core::ptr::read_unaligned(bytes.as_ptr() as *const Self) };

        Ok(config)
    }

    pub fn get_serial_number(&self) -> String {
        String::from_utf8_lossy(&self.serial_number)
            .trim_end_matches('\0')
            .to_string()
    }

    pub fn get_device_id(&self) -> String {
        String::from_utf8_lossy(&self.device_id)
            .trim_end_matches('\0')
            .to_string()
    }

    pub fn get_wifi_ssid(&self) -> String {
        String::from_utf8_lossy(&self.wifi_ssid)
            .trim_end_matches('\0')
            .to_string()
    }

    pub fn get_wifi_password(&self) -> String {
        String::from_utf8_lossy(&self.wifi_password)
            .trim_end_matches('\0')
            .to_string()
    }

    pub fn get_firmware_version(&self) -> String {
        String::from_utf8_lossy(&self.firmware_version)
            .trim_end_matches('\0')
            .to_string()
    }

    #[cfg(feature = "std")]
    pub fn read_file(file_path: &PathBuf) -> Result<Self, Error> {
        let mut file = File::open(file_path)?;
        file.seek(SeekFrom::Start(CONFIG_OFFSET))?;

        let struct_size = mem::size_of::<DeviceConfig>();
        let mut buffer = vec![0u8; struct_size];
        file.read_exact(&mut buffer)?;

        Self::from_bytes(&buffer)
    }

    #[cfg(feature = "std")]
    pub fn write_file(
        config_path: &PathBuf,
        firmware_path: &PathBuf,
        output_path: &PathBuf,
    ) -> Result<Self, Error> {
        let config = if !config_path.exists() {
            DeviceConfig::default()
        } else {
            let content = fs::read_to_string(config_path)?;
            let input: ConfigInput = serde_json::from_str(&content)?;

            DeviceConfig::new(
                &input.serial_number,
                &input.device_id,
                &input.firmware_version,
                &input.wifi_ssid,
                &input.wifi_password,
            )
            .map_err(|error| anyhow::anyhow!(error))?
        };

        let mut file = if firmware_path.exists() {
            fs::copy(firmware_path, output_path)?;

            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(output_path)?
        } else {
            File::create(output_path)?
        };

        file.seek(SeekFrom::Start(CONFIG_OFFSET))?;

        file.write_all(config.as_bytes())
            .map_err(|error| anyhow::anyhow!(error))?;

        Ok(config)
    }

    #[cfg(feature = "std")]
    pub fn display(&self) {
        let width = 42;

        println!("┌{:─<width$}┐", "", width = width);
        println!("│ {:^width$} │", "configuration", width = width - 2);
        println!("├{:─<width$}┤", "", width = width);

        println!(
            "│ {:<width$} │",
            format!("serial_number      {}", self.get_serial_number()),
            width = width - 2
        );
        println!(
            "│ {:<width$} │",
            format!("device_id          {}", self.get_device_id()),
            width = width - 2
        );
        println!(
            "│ {:<width$} │",
            format!("firmware_version   {}", self.get_firmware_version()),
            width = width - 2
        );
        println!(
            "│ {:<width$} │",
            format!("wifi_ssid          {}", self.get_wifi_ssid()),
            width = width - 2
        );
        println!(
            "│ {:<width$} │",
            format!("wifi_password      {}", self.get_wifi_password()),
            width = width - 2
        );

        println!("├{:─<width$}┤", "", width = width);

        println!(
            "│ {:<width$} │",
            format!("offset             0x{:X}", CONFIG_OFFSET),
            width = width - 2
        );

        println!("└{:─<width$}┘", "", width = width);
    }
}

fn copy_to_array(s: &str, dest: &mut [u8]) -> Result<(), &'static str> {
    let bytes = s.as_bytes();
    if bytes.len() >= dest.len() {
        return Err("");
    }
    dest[..bytes.len()].copy_from_slice(bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config = DeviceConfig::new(
            "sn-9999-9999",
            "custom_device",
            "1.2.3",
            "network",
            "secret_password",
        )
        .expect("Failed to create config");

        assert_eq!(config.get_serial_number(), "sn-9999-9999");
        assert_eq!(config.get_device_id(), "custom_device");
        assert_eq!(config.get_firmware_version(), "1.2.3");
        assert_eq!(config.get_wifi_ssid(), "network");
        assert_eq!(config.get_wifi_password(), "secret_password");
    }

    #[test]
    fn test_byte_conversion() {
        let config = DeviceConfig::new(
            "sn-9999-9999",
            "custom_device",
            "1.2.3",
            "network",
            "secret_password",
        )
        .expect("Failed to create config");

        let bytes = config.as_bytes();
        let deserialized = DeviceConfig::from_bytes(bytes).expect("Failed to deserialize");

        assert_eq!(config.get_serial_number(), deserialized.get_serial_number());
        assert_eq!(config.get_device_id(), deserialized.get_device_id());
        assert_eq!(
            config.get_firmware_version(),
            deserialized.get_firmware_version()
        );
        assert_eq!(config.get_wifi_ssid(), deserialized.get_wifi_ssid());
        assert_eq!(config.get_wifi_password(), deserialized.get_wifi_password());
    }

    #[test]
    fn test_from_bytes_too_short() {
        let short_buffer = vec![0u8; 10];
        let result = DeviceConfig::from_bytes(&short_buffer);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_fields_too_long() {
        let too_long = "a".repeat(100);

        assert!(DeviceConfig::new(&too_long, "dev", "1.0", "ssid", "pass").is_err());
        assert!(DeviceConfig::new("sn", &too_long, "1.0", "ssid", "pass").is_err());
        assert!(DeviceConfig::new("sn", "dev", &too_long, "ssid", "pass").is_err());
        assert!(DeviceConfig::new("sn", "dev", "1.0", &too_long, "pass").is_err());
        assert!(DeviceConfig::new("sn", "dev", "1.0", "ssid", &too_long).is_err());
    }

    #[test]
    fn test_write_file() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");

        let config_path = dir.path().join("config.json");
        let firmware_path = dir.path().join("firmware.bin");
        let output_path = dir.path().join("output.bin");

        let config = DeviceConfig::write_file(&config_path, &firmware_path, &output_path)
            .expect("Failed to write default config without firmware");
        let default = DeviceConfig::default();

        assert_eq!(config.get_serial_number(), default.get_serial_number());
        assert_eq!(config.get_device_id(), default.get_device_id());
        assert_eq!(
            config.get_firmware_version(),
            default.get_firmware_version()
        );
        assert_eq!(config.get_wifi_ssid(), default.get_wifi_ssid());
        assert_eq!(config.get_wifi_password(), default.get_wifi_password());

        let json_data = r#"{
                "serial_number": "sn-9999-9999",
                "device_id": "custom_device",
                "firmware_version": "1.2.3",
                "wifi_ssid": "network",
                "wifi_password": "secret_password"
            }"#;
        let mut json_file = File::create(&config_path).unwrap();
        json_file.write_all(json_data.as_bytes()).unwrap();

        let mut fw_file = File::create(&firmware_path).unwrap();
        fw_file.write_all(&vec![0xFF; 500000]).unwrap();

        let custom = DeviceConfig::write_file(&config_path, &firmware_path, &output_path)
            .expect("Failed to write custom config with firmware");

        assert_eq!(custom.get_serial_number(), "sn-9999-9999");
        assert_eq!(custom.get_device_id(), "custom_device");
        assert_eq!(custom.get_firmware_version(), "2.0.0");
        assert_eq!(custom.get_wifi_ssid(), "network");
        assert_eq!(custom.get_wifi_password(), "secret_password");
    }

    #[test]
    fn test_read_file() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");

        let config_path = dir.path().join("config.json");
        let firmware_path = dir.path().join("firmware.bin");
        let output_path = dir.path().join("output.bin");

        DeviceConfig::write_file(&config_path, &firmware_path, &output_path)
            .expect("Failed to prepare output file for reading");

        let config =
            DeviceConfig::read_file(&output_path).expect("Failed to read config from binary file");
        let default = DeviceConfig::default();

        assert_eq!(config.get_serial_number(), default.get_serial_number());
        assert_eq!(config.get_device_id(), default.get_device_id());
        assert_eq!(
            config.get_firmware_version(),
            default.get_firmware_version()
        );
        assert_eq!(config.get_wifi_ssid(), default.get_wifi_ssid());
        assert_eq!(config.get_wifi_password(), default.get_wifi_password());
    }
}
