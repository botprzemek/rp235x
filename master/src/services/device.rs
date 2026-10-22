use config::{ConfigError, DeviceConfig};
use net::Device;
use tokio::sync::{broadcast, watch};

use crate::{
    adapters::repositories::{DeviceRepository, Repository},
    services::Observable,
};

#[derive(Clone)]
pub struct DeviceService {
    device_repository: DeviceRepository,
    state: watch::Sender<Vec<Device>>,
    broadcast: broadcast::Sender<Vec<Device>>,
    disconnect_tx: broadcast::Sender<String>,
    game_assign_tx: broadcast::Sender<(String, u16)>,
}

impl Observable<Vec<Device>> for DeviceService {
    fn subscribe(&self) -> broadcast::Receiver<Vec<Device>> {
        self.broadcast.subscribe()
    }

    fn watch(&self) -> watch::Receiver<Vec<Device>> {
        self.state.subscribe()
    }
}

impl DeviceService {
    pub fn new(device_repository: DeviceRepository) -> Self {
        device_repository.upsert(Vec::new());
        let (state, _state_rx) = watch::channel(device_repository.select());
        let (broadcast, _broadcast_rx) = broadcast::channel(100);
        let (disconnect_tx, _disconnect_rx) = broadcast::channel(100);
        let (game_assign_tx, _game_assign_rx) = broadcast::channel(100);

        Self {
            device_repository,
            state,
            broadcast,
            disconnect_tx,
            game_assign_tx,
        }
    }

    pub fn get_all(&self) -> Vec<Device> {
        self.state.borrow().clone()
    }

    pub fn get_game_id_by_serial(&self, serial_number: &str) -> Option<u16> {
        self.state
            .borrow()
            .iter()
            .find(|d| d.get_serial_number() == Ok(serial_number))
            .map(|d| d.game_id())
    }

    pub fn subscribe_disconnect(&self) -> broadcast::Receiver<String> {
        self.disconnect_tx.subscribe()
    }

    pub fn subscribe_game_assign(&self) -> broadcast::Receiver<(String, u16)> {
        self.game_assign_tx.subscribe()
    }

    pub fn assign_game_by_serial(&self, serial_number: &str, game_id: u16) -> Option<Device> {
        let mut updated_device = None;
        let mut updated_list = Vec::new();

        self.state.send_modify(|devices| {
            if let Some(existing) = devices
                .iter_mut()
                .find(|d| d.get_serial_number() == Ok(serial_number))
            {
                existing.set_game_id(game_id);
                updated_device = Some(*existing);
            }
            updated_list = devices.clone();
        });

        if updated_device.is_some() {
            let repo = self.device_repository.clone();
            let snapshot = updated_list.clone();
            tokio::task::spawn_blocking(move || {
                repo.upsert(snapshot);
            });

            if self.broadcast.receiver_count() > 0 {
                let _ = self.broadcast.send(updated_list);
            }

            let _ = self.game_assign_tx.send((serial_number.to_string(), game_id));
        }

        updated_device
    }

    pub fn validate_and_register(&self, device: Device) -> Result<Device, ConfigError> {
        device.validate()?;

        let serial_number = device.get_serial_number()?;
        let device_id = device.get_device_id()?;

        if serial_number.is_empty() || device_id.is_empty() {
            return Err(ConfigError::Serialization);
        }

        let mut updated = Vec::new();
        self.state.send_modify(|devices| {
            if let Some(existing) = devices
                .iter_mut()
                .find(|d| d.get_serial_number() == Ok(serial_number))
            {
                *existing = device;
            } else {
                devices.push(device);
            }
            updated = devices.clone();
        });

        let repo = self.device_repository.clone();
        let snapshot = updated.clone();
        tokio::task::spawn_blocking(move || {
            repo.upsert(snapshot);
        });

        if self.broadcast.receiver_count() > 0
            && let Err(error) = self.broadcast.send(updated)
        {
            eprintln!("ERROR: Failed to broadcast devices: {}", error);
        }

        Ok(device)
    }

    pub fn unregister(&self, device: &Device) {
        if let Ok(serial_number) = device.get_serial_number() {
            self.remove_by_serial(serial_number);
        }
    }

    pub fn disconnect_by_serial(&self, serial_number: &str) {
        self.remove_by_serial(serial_number);
        let _ = self.disconnect_tx.send(serial_number.to_string());
    }

    fn remove_by_serial(&self, serial_number: &str) {
        let mut updated = Vec::new();
        self.state.send_modify(|devices| {
            devices.retain(|d| d.get_serial_number() != Ok(serial_number));
            updated = devices.clone();
        });

        let repo = self.device_repository.clone();
        let snapshot = updated.clone();
        tokio::task::spawn_blocking(move || {
            repo.upsert(snapshot);
        });

        if self.broadcast.receiver_count() > 0 {
            let _ = self.broadcast.send(updated);
        }
    }
}
