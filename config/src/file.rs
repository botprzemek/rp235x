use crate::{Config, ConfigError};

use crate::CONFIG_SIZE;
#[cfg(feature = "serde")]
use crate::ConfigInput;

use std::fs::File;
use std::io::Read;
use std::path::Path;

pub trait FileReader {
    #[cfg(feature = "serde")]
    fn read_json(path: &Path) -> Result<Config, ConfigError>;
    fn read_bin(path: &Path) -> Result<Config, ConfigError>;
    fn read_file(path: &Path) -> Result<Config, ConfigError>;
}

pub trait FileWriter {
    fn write_file(&self, path: &Path) -> Result<(), ConfigError>;
}

impl FileReader for Config {
    #[cfg(feature = "serde")]
    fn read_json(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound);
        }

        let content = std::fs::read_to_string(path)?;
        let input: ConfigInput =
            serde_json::from_str(&content).map_err(|_| ConfigError::Serialization)?;

        Config::new(
            &input.serial_number,
            &input.device_id,
            &input.wifi_ssid,
            &input.wifi_password,
        )
    }

    fn read_bin(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound);
        }

        let mut file = File::open(path)?;
        let mut buffer = [0u8; CONFIG_SIZE];
        file.read_exact(&mut buffer)?;

        let ptr = buffer.as_ptr();
        let config = unsafe { core::ptr::read_unaligned(ptr as *const Self) };
        if !config.verify_integrity() {
            return Err(ConfigError::IntegrityCheckFailed);
        }

        Ok(config)
    }

    fn read_file(path: &Path) -> Result<Self, ConfigError> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or(ConfigError::Serialization)?;

        match extension {
            #[cfg(feature = "serde")]
            "json" => Self::read_json(path),
            "bin" => Self::read_bin(path),
            _ => Ok(Self::default()),
        }
    }
}

impl FileWriter for Config {
    fn write_file(&self, path: &Path) -> Result<(), ConfigError> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        file.write_all(self.as_bytes())?;

        Ok(())
    }
}
