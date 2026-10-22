pub mod layout;

use config::{ConfigError, DeviceConfig};

use crate::{ClientPacket, ServerPacket};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeviceType {
    Worker = 0x01,
    WebBrowser = 0x02,
}

impl TryFrom<u8> for DeviceType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Worker),
            0x02 => Ok(Self::WebBrowser),
            _ => Err("device_type"),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Device {
    device_type: DeviceType,
    game_id: u16,
    serial_number: [u8; layout::SERIAL_NUMBER_SIZE],
    device_id: [u8; layout::DEVICE_ID_SIZE],
}

impl Device {
    pub fn new(
        device_type: DeviceType,
        serial_number: &str,
        device_id: &str,
    ) -> Result<Self, ConfigError> {
        let default_game_id = match device_type {
            DeviceType::Worker => 0,
            DeviceType::WebBrowser => 1,
        };
        Self::with_game_id(device_type, default_game_id, serial_number, device_id)
    }

    pub fn with_game_id(
        device_type: DeviceType,
        game_id: u16,
        serial_number: &str,
        device_id: &str,
    ) -> Result<Self, ConfigError> {
        let mut sn_buf = [0u8; layout::SERIAL_NUMBER_SIZE];
        let mut id_buf = [0u8; layout::DEVICE_ID_SIZE];

        Self::write_field(&mut sn_buf, serial_number)?;
        Self::write_field(&mut id_buf, device_id)?;

        let device = Self {
            device_type,
            game_id,
            serial_number: sn_buf,
            device_id: id_buf,
        };

        device.validate()?;
        Ok(device)
    }

    pub fn from_config(
        device_type: DeviceType,
        config: &impl DeviceConfig,
    ) -> Result<Self, ConfigError> {
        Self::new(
            device_type,
            config.get_serial_number()?,
            config.get_device_id()?,
        )
    }

    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    pub fn game_id(&self) -> u16 {
        self.game_id
    }

    pub fn set_game_id(&mut self, game_id: u16) {
        self.game_id = game_id;
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        let serial_number = self.get_serial_number()?;
        let device_id = self.get_device_id()?;

        if serial_number.is_empty() || device_id.is_empty() {
            return Err(ConfigError::Serialization);
        }

        Ok(())
    }

    pub fn to_bytes(&self) -> [u8; layout::DATA_SIZE] {
        let mut bytes = [0u8; layout::DATA_SIZE];

        bytes[layout::DEVICE_TYPE] = self.device_type as u8;
        bytes[layout::GAME_ID_START..layout::GAME_ID_END]
            .copy_from_slice(&self.game_id.to_be_bytes());
        bytes[layout::SERIAL_NUMBER_START..layout::SERIAL_NUMBER_END]
            .copy_from_slice(&self.serial_number);
        bytes[layout::DEVICE_ID_START..layout::DEVICE_ID_END].copy_from_slice(&self.device_id);

        bytes
    }

    pub fn from_bytes(buffer: &[u8]) -> Result<Self, &'static str> {
        if buffer.len() < layout::DATA_SIZE {
            return Err("device_length");
        }

        let device_type = DeviceType::try_from(buffer[layout::DEVICE_TYPE])?;

        let mut game_id_bytes = [0u8; 2];
        let mut serial_number = [0u8; layout::SERIAL_NUMBER_SIZE];
        let mut device_id = [0u8; layout::DEVICE_ID_SIZE];

        game_id_bytes.copy_from_slice(&buffer[layout::GAME_ID_START..layout::GAME_ID_END]);
        serial_number
            .copy_from_slice(&buffer[layout::SERIAL_NUMBER_START..layout::SERIAL_NUMBER_END]);
        device_id.copy_from_slice(&buffer[layout::DEVICE_ID_START..layout::DEVICE_ID_END]);

        let game_id = u16::from_be_bytes(game_id_bytes);

        let device = Self {
            device_type,
            game_id,
            serial_number,
            device_id,
        };

        device.validate().map_err(|_| "device_validation")?;

        Ok(device)
    }

    fn write_field<const N: usize>(buf: &mut [u8; N], src: &str) -> Result<(), ConfigError> {
        let bytes = src.as_bytes();
        if bytes.len() > N {
            return Err(ConfigError::StringTooLong);
        }

        buf.fill(0);
        buf[..bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    fn parse_field(buf: &[u8]) -> Result<&str, ConfigError> {
        let length = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());

        core::str::from_utf8(&buf[..length]).map_err(|_| ConfigError::InvalidEncoding)
    }
}

impl DeviceConfig for Device {
    fn get_serial_number(&self) -> Result<&str, ConfigError> {
        Self::parse_field(&self.serial_number)
    }

    fn get_device_id(&self) -> Result<&str, ConfigError> {
        Self::parse_field(&self.device_id)
    }
}

impl TryFrom<ClientPacket> for Device {
    type Error = &'static str;

    fn try_from(packet: ClientPacket) -> Result<Self, Self::Error> {
        Self::from_bytes(&packet.data())
    }
}

impl TryFrom<ServerPacket> for Device {
    type Error = &'static str;

    fn try_from(packet: ServerPacket) -> Result<Self, Self::Error> {
        Self::from_bytes(&packet.data())
    }
}
