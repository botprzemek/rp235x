use crate::{Config, ConfigError, DeviceConfig};

#[cfg(feature = "std")]
use crate::NetworkConfig;

#[cfg(feature = "std")]
use std::{format, println};

#[cfg(feature = "defmt")]
use defmt::info;

pub trait Print {
    fn print(&self) -> Result<(), ConfigError>;
}

#[cfg(feature = "std")]
const LINE_WIDTH: usize = 42;

impl Print for Config {
    #[cfg(feature = "defmt")]
    fn print(&self) -> Result<(), ConfigError> {
        info!(
            "DeviceConfig::SerialNumber           {}",
            &self.get_serial_number()?
        );
        info!(
            "DeviceConfig::DeviceID               {}",
            &self.get_device_id()?
        );

        Ok(())
    }

    #[cfg(feature = "std")]
    fn print(&self) -> Result<(), ConfigError> {
        println!("┌{:─<width$}┐", "", width = LINE_WIDTH);
        println!("│ {:^width$} │", "configuration", width = LINE_WIDTH - 2);
        println!("├{:─<width$}┤", "", width = LINE_WIDTH);

        println!(
            "│ {:<width$} │",
            format!("serial_number      {}", self.get_serial_number()?),
            width = LINE_WIDTH - 2
        );
        println!(
            "│ {:<width$} │",
            format!("device_id          {}", self.get_device_id()?),
            width = LINE_WIDTH - 2
        );
        println!(
            "│ {:<width$} │",
            format!("wifi_ssid          {}", self.get_wifi_ssid()?),
            width = LINE_WIDTH - 2
        );
        println!(
            "│ {:<width$} │",
            format!("wifi_password      {}", self.get_wifi_password()?),
            width = LINE_WIDTH - 2
        );

        println!("└{:─<width$}┘", "", width = LINE_WIDTH);

        Ok(())
    }
}
