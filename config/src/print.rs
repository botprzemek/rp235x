use crate::{Config, ConfigError};

pub trait Print {
    fn print(&self) -> Result<(), ConfigError>;
}

impl Print for Config {
    #[cfg(feature = "defmt")]
    fn print(&self) -> Result<(), ConfigError> {
        use crate::DeviceConfig;
        use defmt::info;

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
        use crate::{DeviceConfig, NetworkConfig};
        use std::{format, println};

        let width = 42;

        println!("┌{:─<width$}┐", "", width = width);
        println!("│ {:^width$} │", "configuration", width = width - 2);
        println!("├{:─<width$}┤", "", width = width);

        println!(
            "│ {:<width$} │",
            format!("serial_number      {}", self.get_serial_number()?),
            width = width - 2
        );
        println!(
            "│ {:<width$} │",
            format!("device_id          {}", self.get_device_id()?),
            width = width - 2
        );
        println!(
            "│ {:<width$} │",
            format!("wifi_ssid          {}", self.get_wifi_ssid()?),
            width = width - 2
        );
        println!(
            "│ {:<width$} │",
            format!("wifi_password      {}", self.get_wifi_password()?),
            width = width - 2
        );

        println!("└{:─<width$}┘", "", width = width);

        Ok(())
    }
}
