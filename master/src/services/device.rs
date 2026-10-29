//! Service for tracking connected devices, game assignments, and disconnections.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use config::{ConfigError, DeviceConfig};
use net::{Device, DeviceType};
use tokio::sync::{broadcast, watch};
use tokio::task;
use tokio::time::{self, MissedTickBehavior};

use crate::adapters::repositories::{DeviceRepository, Repository};
use crate::services::Observable;

/// Notification emitted when a device is assigned to a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceGameAssignment {
    pub serial_number: String,
    pub game_id: u16,
}

/// Manages connected [`Device`]s, their game assignments, and asynchronous persistence.
#[derive(Clone)]
pub struct DeviceService {
    #[allow(dead_code)]
    device_repository: DeviceRepository,
    state: watch::Sender<Vec<Device>>,
    assigned_games: Arc<RwLock<HashMap<String, u16>>>,
    broadcast: broadcast::Sender<Vec<Device>>,
    disconnect_tx: broadcast::Sender<String>,
    game_assign_tx: broadcast::Sender<DeviceGameAssignment>,
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
    /// Creates a new [`DeviceService`] and spawns its background database writer task.
    #[must_use]
    pub fn new(device_repository: DeviceRepository) -> Self {
        device_repository.upsert(&[]);
        let (state, mut state_rx) = watch::channel(device_repository.select());
        let (broadcast, _broadcast_rx) = broadcast::channel(64);
        let (disconnect_tx, _disconnect_rx) = broadcast::channel(64);
        let (game_assign_tx, _game_assign_rx) = broadcast::channel(64);
        let assigned_games = Arc::new(RwLock::new(HashMap::new()));

        // Asynchronous persistence to ReDB in the background without blocking Tokio registration tasks.
        let repository_writer = device_repository.clone();
        tokio::spawn(async move {
            let mut database_interval = time::interval(Duration::from_millis(250));
            database_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                if state_rx.changed().await.is_err() {
                    break;
                }
                database_interval.tick().await;
                let snapshot = state_rx.borrow_and_update().clone();
                let repository = repository_writer.clone();
                let _ = task::spawn_blocking(move || {
                    repository.upsert(&snapshot);
                })
                .await;
            }
        });

        Self {
            device_repository,
            state,
            assigned_games,
            broadcast,
            disconnect_tx,
            game_assign_tx,
        }
    }

    /// Returns a snapshot of all currently registered devices.
    #[must_use]
    pub fn all(&self) -> Vec<Device> {
        self.state.borrow().clone()
    }

    /// Returns the assigned game ID for the device with `serial_number`, if any.
    #[must_use]
    pub fn game_id_by_serial(&self, serial_number: &str) -> Option<u16> {
        if let Some(&game_id) = self
            .assigned_games
            .read()
            .expect("assigned_games lock should not be poisoned")
            .get(serial_number)
            && game_id != 0
        {
            return Some(game_id);
        }

        self.state
            .borrow()
            .iter()
            .find(|device| device.serial_number() == Ok(serial_number))
            .map(Device::game_id)
    }

    /// Subscribes to device disconnection requests by serial number.
    #[must_use]
    pub fn subscribe_disconnect(&self) -> broadcast::Receiver<String> {
        self.disconnect_tx.subscribe()
    }

    /// Subscribes to device-to-game assignment updates.
    #[must_use]
    pub fn subscribe_game_assign(&self) -> broadcast::Receiver<DeviceGameAssignment> {
        self.game_assign_tx.subscribe()
    }

    /// Assigns the device with `serial_number` to `game_id` and broadcasts the update.
    pub fn assign_game_by_serial(&self, serial_number: &str, game_id: u16) -> Option<Device> {
        self.assigned_games
            .write()
            .expect("assigned_games lock should not be poisoned")
            .insert(serial_number.to_string(), game_id);

        let mut updated_device = None;

        self.state.send_modify(|devices| {
            if let Some(existing) = devices
                .iter_mut()
                .find(|device| device.serial_number() == Ok(serial_number))
            {
                existing.set_game_id(game_id);
                updated_device = Some(*existing);
            } else if let Ok(stub) =
                Device::try_new_with_game_id(DeviceType::Worker, game_id, serial_number, "pending")
            {
                devices.push(stub);
                updated_device = Some(stub);
            }
        });

        if updated_device.is_some() {
            let _ = self.game_assign_tx.send(DeviceGameAssignment {
                serial_number: serial_number.to_string(),
                game_id,
            });
        }

        updated_device
    }

    /// Validates `device` and registers or updates it in the active device list.
    pub fn validate_and_register(&self, device: Device) -> Result<Device, ConfigError> {
        device.validate()?;

        let serial_number = device.serial_number()?;
        let device_id = device.device_id()?;

        if serial_number.is_empty() || device_id.is_empty() {
            return Err(ConfigError::Serialization);
        }

        self.state.send_modify(|devices| {
            if let Some(existing) = devices
                .iter_mut()
                .find(|entry| entry.serial_number() == Ok(serial_number))
            {
                *existing = device;
            } else {
                devices.push(device);
            }
        });

        Ok(device)
    }

    /// Unregisters `device` from the active device list.
    pub fn unregister(&self, device: &Device) {
        if let Ok(serial_number) = device.serial_number() {
            self.remove_by_serial(serial_number);
        }
    }

    /// Removes the device with `serial_number` and notifies listeners to close its connection.
    pub fn disconnect_by_serial(&self, serial_number: &str) {
        self.remove_by_serial(serial_number);
        let _ = self.disconnect_tx.send(serial_number.to_string());
    }

    fn remove_by_serial(&self, serial_number: &str) {
        self.state.send_modify(|devices| {
            devices.retain(|device| device.serial_number() != Ok(serial_number));
        });
    }
}
