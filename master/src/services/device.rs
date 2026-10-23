use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use config::{ConfigError, DeviceConfig};
use net::{Device, DeviceType};
use tokio::sync::{broadcast, watch};

use crate::{
    adapters::repositories::{DeviceRepository, Repository},
    services::Observable,
};

#[derive(Clone)]
pub struct DeviceService {
    #[allow(dead_code)]
    device_repository: DeviceRepository,
    state: watch::Sender<Vec<Device>>,
    assigned_games: Arc<RwLock<HashMap<String, u16>>>,
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
        let (state, mut state_rx) = watch::channel(device_repository.select());
        let (broadcast, _broadcast_rx) = broadcast::channel(4096);
        let (disconnect_tx, _disconnect_rx) = broadcast::channel(4096);
        let (game_assign_tx, _game_assign_rx) = broadcast::channel(4096);
        let assigned_games = Arc::new(RwLock::new(HashMap::new()));

        // Asynchroniczny zapis do ReDB w tle (bez blokowania wątków Tokio podczas rejestracji)
        let repo_writer = device_repository.clone();
        tokio::spawn(async move {
            let mut db_interval = tokio::time::interval(std::time::Duration::from_millis(250));
            db_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                if state_rx.changed().await.is_err() {
                    break;
                }
                db_interval.tick().await;
                let snapshot = state_rx.borrow_and_update().clone();
                let repo = repo_writer.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    repo.upsert(snapshot);
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

    pub fn get_all(&self) -> Vec<Device> {
        self.state.borrow().clone()
    }

    pub fn get_game_id_by_serial(&self, serial_number: &str) -> Option<u16> {
        if let Some(&gid) = self.assigned_games.read().unwrap().get(serial_number)
            && gid != 0
        {
            return Some(gid);
        }

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
        self.assigned_games
            .write()
            .unwrap()
            .insert(serial_number.to_string(), game_id);

        let mut updated_device = None;

        self.state.send_modify(|devices| {
            if let Some(existing) = devices
                .iter_mut()
                .find(|d| d.get_serial_number() == Ok(serial_number))
            {
                existing.set_game_id(game_id);
                updated_device = Some(*existing);
            } else if let Ok(stub) =
                Device::with_game_id(DeviceType::Worker, game_id, serial_number, "pending")
            {
                devices.push(stub);
                updated_device = Some(stub);
            }
        });

        if updated_device.is_some() {
            let _ = self
                .game_assign_tx
                .send((serial_number.to_string(), game_id));
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

        self.state.send_modify(|devices| {
            if let Some(existing) = devices
                .iter_mut()
                .find(|d| d.get_serial_number() == Ok(serial_number))
            {
                *existing = device;
            } else {
                devices.push(device);
            }
        });

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
        self.state.send_modify(|devices| {
            devices.retain(|d| d.get_serial_number() != Ok(serial_number));
        });
    }
}
