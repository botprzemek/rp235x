use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ConfigInput {
    serial_number: String,
    device_id: String,
    wifi_ssid: String,
    wifi_pass: String,
    firmware_ver: String,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DeviceConfig {
    serial_number: [u8; 32],
    device_id: [u8; 32],
    wifi_ssid: [u8; 32],
    wifi_pass: [u8; 64],
    firmware_ver: [u8; 16],
}

impl DeviceConfig {
    fn new(
        sn: &str,
        dev_id: &str,
        ssid: &str,
        pass: &str,
        ver: &str,
    ) -> Result<Self, &'static str> {
        let mut config = Self {
            serial_number: [0; 32],
            device_id: [0; 32],
            wifi_ssid: [0; 32],
            wifi_pass: [0; 64],
            firmware_ver: [0; 16],
        };

        copy_to_array(sn, &mut config.serial_number)?;
        copy_to_array(dev_id, &mut config.device_id)?;
        copy_to_array(ssid, &mut config.wifi_ssid)?;
        copy_to_array(pass, &mut config.wifi_pass)?;
        copy_to_array(ver, &mut config.firmware_ver)?;

        Ok(config)
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < std::mem::size_of::<Self>() {
            return Err("");
        }
        let config = unsafe { *(bytes.as_ptr() as *const Self) };
        Ok(config)
    }

    fn get_serial_string(&self) -> String {
        String::from_utf8_lossy(&self.serial_number)
            .trim_end_matches('\0')
            .to_string()
    }
    fn get_device_id_string(&self) -> String {
        String::from_utf8_lossy(&self.device_id)
            .trim_end_matches('\0')
            .to_string()
    }
    fn get_wifi_ssid_string(&self) -> String {
        String::from_utf8_lossy(&self.wifi_ssid)
            .trim_end_matches('\0')
            .to_string()
    }
    fn get_firmware_ver_string(&self) -> String {
        String::from_utf8_lossy(&self.firmware_ver)
            .trim_end_matches('\0')
            .to_string()
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
