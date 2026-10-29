//! Diagnostic printing for [`Config`].

#[cfg(feature = "std")]
use std::{format, println};

#[cfg(feature = "defmt")]
use defmt::info;

use crate::Config;
#[cfg(feature = "std")]
use crate::NetworkConfig;
#[cfg(any(feature = "std", feature = "defmt"))]
use crate::{ConfigError, DeviceConfig};

/// Formats and outputs a [`Config`] to the console or `defmt` logger.
pub trait Print {
    /// Outputs the configuration details.
    #[cfg(feature = "defmt")]
    fn print(&self) -> Result<(), ConfigError>;

    /// Outputs the configuration details in a formatted table.
    #[cfg(feature = "std")]
    fn print(&self) -> Result<(), ConfigError>;
}

#[cfg(feature = "std")]
const LINE_WIDTH: usize = 42;

impl Print for Config {
    #[cfg(feature = "defmt")]
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
}
