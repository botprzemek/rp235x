use config::DeviceConfig;
use net::Device;
use redb::{ReadableDatabase, ReadableTable};

use crate::adapters::data::{DEVICES_TABLE, ReDBProvider};
use crate::adapters::repositories::Repository;

#[derive(Clone)]
pub struct DeviceRepository {
    database: ReDBProvider,
}

impl Repository<Vec<Device>> for DeviceRepository {
    fn select(&self) -> Vec<Device> {
        let transaction = self.database.get_connection().begin_read().unwrap();
        let table = transaction.open_table(DEVICES_TABLE).unwrap();
        let mut devices = Vec::new();

        for entry in table.iter().unwrap().flatten() {
            if let Ok(device) = Device::from_bytes(entry.1.value()) {
                devices.push(device);
            }
        }

        devices
    }

    fn upsert(&self, entities: Vec<Device>) {
        let transaction = self.database.get_connection().begin_write().unwrap();
        let _ = transaction.delete_table(DEVICES_TABLE);
        let mut table = transaction.open_table(DEVICES_TABLE).unwrap();

        for device in entities {
            if let Ok(serial_number) = device.get_serial_number() {
                let bytes = device.to_bytes();
                table.insert(serial_number, bytes.as_slice()).unwrap();
            }
        }

        drop(table);
        transaction.commit().unwrap();
    }
}

impl DeviceRepository {
    pub fn new(database: ReDBProvider) -> Self {
        Self { database }
    }
}

