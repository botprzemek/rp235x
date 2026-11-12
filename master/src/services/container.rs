use crate::adapters::repositories::Registry;
use crate::services::{DeviceService, GameService};

use tokio::sync::{broadcast, watch};

#[derive(Clone)]
pub struct Services {
    game: GameService,
    devices: DeviceService,
}

pub trait Observable<E> {

    fn subscribe(&self) -> broadcast::Receiver<E>;

    fn watch(&self) -> watch::Receiver<E>;
}

impl Services {

    #[must_use]
    pub fn new(registry: &Registry) -> Self {
        let game = GameService::new(registry.game().clone());
        let devices = DeviceService::new(registry.devices().clone());

        Self { game, devices }
    }

    #[must_use]
    pub fn game(&self) -> &GameService {
        &self.game
    }

    #[must_use]
    pub fn devices(&self) -> &DeviceService {
        &self.devices
    }
}
