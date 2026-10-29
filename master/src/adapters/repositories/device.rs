//! ReDB-backed repository for [`Device`] records.

use config::DeviceConfig;
use net::Device;

use crate::adapters::data::{DEVICES_TABLE, ReDBProvider};
use crate::adapters::repositories::Repository;

/// Persistence adapter for [`Device`] records in ReDB.
#[derive(Clone)]
pub struct DeviceRepository {
    database: ReDBProvider,
}

impl Repository<[Device]> for DeviceRepository {
    type Output = Vec<Device>;

    fn select(&self) -> Vec<Device> {
        use redb::{ReadableDatabase, ReadableTable};

        let transaction = self
            .database
            .connection()
            .begin_read()
            .expect("failed to begin ReDB read transaction for devices");
        let table = transaction
            .open_table(DEVICES_TABLE)
            .expect("failed to open devices table");
        let mut devices = Vec::new();

        let entries = table
            .iter()
            .expect("failed to iterate over devices table")
            .flatten();
        for (_key, value) in entries {
            if let Ok(device) = Device::from_bytes(value.value()) {
                devices.push(device);
            }
        }

        devices
    }

    fn upsert(&self, entities: &[Device]) {
        let transaction = self
            .database
            .connection()
            .begin_write()
            .expect("failed to begin ReDB write transaction for devices");
        let _ = transaction.delete_table(DEVICES_TABLE);
        let mut table = transaction
            .open_table(DEVICES_TABLE)
            .expect("failed to open devices table for writing");

        for device in entities {
            if let Ok(serial_number) = device.serial_number() {
                let bytes = device.to_bytes();
                table
                    .insert(serial_number, bytes.as_slice())
                    .expect("failed to insert device into devices table");
            }
        }

        drop(table);
        transaction
            .commit()
            .expect("failed to commit devices transaction");
    }
}

impl DeviceRepository {
    /// Creates a new [`DeviceRepository`] backed by `database`.
    #[must_use]
    pub fn new(database: ReDBProvider) -> Self {
        Self { database }
    }
}
