#[cfg(feature = "std")]
use std::{format, println};

#[cfg(any(feature = "std", feature = "defmt"))]
use crate::DeviceConfig;
#[cfg(feature = "std")]
use crate::NetworkConfig;
use crate::{Config, ConfigError};

#[cfg(all(feature = "defmt", not(feature = "std")))]
use defmt::info;

pub trait Print {

    fn print(&self) -> Result<(), ConfigError>;
}

#[cfg(feature = "std")]
const LINE_WIDTH: usize = 42;

impl Print for Config {
    #[cfg(feature = "std")]
    fn print(&self) -> Result<(), ConfigError> {
        println!("┌{:─<width$}┐", "", width = LINE_WIDTH);
        println!("│ {:^width$} │", "configuration", width = LINE_WIDTH - 2);
        println!("├{:─<width$}┤", "", width = LINE_WIDTH);

        println!(
            "│ {:<width$} │",
            format!("serial_number      {}", self.serial_number()?),
            width = LINE_WIDTH - 2
        );
        println!(
            "│ {:<width$} │",
            format!("device_id          {}", self.device_id()?),
            width = LINE_WIDTH - 2
        );
        println!(
            "│ {:<width$} │",
            format!("wifi_ssid          {}", self.wifi_ssid()?),
            width = LINE_WIDTH - 2
        );
        println!(
            "│ {:<width$} │",
            format!("wifi_password      {}", self.wifi_password()?),
            width = LINE_WIDTH - 2
        );

        println!("└{:─<width$}┘", "", width = LINE_WIDTH);

        Ok(())
    }

    #[cfg(all(feature = "defmt", not(feature = "std")))]
    fn print(&self) -> Result<(), ConfigError> {
        info!(
            "DeviceConfig::SerialNumber           {}",
            &self.serial_number()?
        );
        info!(
            "DeviceConfig::DeviceID               {}",
            &self.device_id()?
        );

        Ok(())
    }

    #[cfg(not(any(feature = "std", feature = "defmt")))]
    fn print(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}
