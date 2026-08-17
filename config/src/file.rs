#[cfg(feature = "std")]
use crate::{CONFIG_SIZE, Config, ConfigError, ConfigInput};

#[cfg(feature = "std")]
use std::path::Path;

#[cfg(feature = "std")]
pub trait FileReader {
    fn read_json(path: &Path) -> Result<Config, ConfigError>;
    fn read_bin(path: &Path) -> Result<Config, ConfigError>;
    fn read_file(path: &Path) -> Result<Config, ConfigError>;
}

#[cfg(feature = "std")]
pub trait FileWriter {
    fn write_file(&self, path: &Path) -> Result<(), ConfigError>;
}

#[cfg(feature = "std")]
impl FileReader for Config {
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
        use std::fs::File;
        use std::io::Read;

        if !path.exists() {
            return Err(ConfigError::FileNotFound);
        }

        let mut file = File::open(path)?;
        let mut buffer = [0u8; CONFIG_SIZE];
        file.read_exact(&mut buffer)?;

        Self::from_bytes(&buffer)
    }

    fn read_file(path: &Path) -> Result<Self, ConfigError> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or(ConfigError::Serialization)?;

        match extension {
            "json" => Self::read_json(path),
            "bin" => Self::read_bin(path),
            _ => Ok(Self::default()),
        }
    }
}

#[cfg(feature = "std")]
impl FileWriter for Config {
    fn write_file(&self, path: &Path) -> Result<(), ConfigError> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        file.write_all(self.as_bytes())?;

        Ok(())
    }
}
