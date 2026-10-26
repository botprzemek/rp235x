mod device;
mod game;

pub use device::DeviceService;
pub use game::{GameFrame, GameService};
use tokio::sync::{broadcast, watch};

use crate::adapters::repositories::Registry;

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
    pub fn new(registry: Registry) -> Self {
        let game = GameService::new(registry.game().clone());
        let devices = DeviceService::new(registry.devices().clone());

        Self { game, devices }
    }

    pub fn game(&self) -> &GameService {
        &self.game
    }

    pub fn devices(&self) -> &DeviceService {
        &self.devices
    }
}
