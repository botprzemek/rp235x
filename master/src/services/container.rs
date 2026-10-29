//! [`Services`] container and [`Observable`] state subscription trait.

use tokio::sync::{broadcast, watch};

use crate::adapters::repositories::Registry;
use crate::services::{DeviceService, GameService};

/// Container holding the application's [`GameService`] and [`DeviceService`].
#[derive(Clone)]
pub struct Services {
    game: GameService,
    devices: DeviceService,
}

/// Provides broadcast and watch subscriptions to state changes of type `E`.
pub trait Observable<E> {
    /// Subscribes to event broadcasts.
    fn subscribe(&self) -> broadcast::Receiver<E>;

    /// Watches the latest value of the state.
    fn watch(&self) -> watch::Receiver<E>;
}

impl Services {
    /// Creates a new [`Services`] instance backed by `registry`.
    #[must_use]
    pub fn new(registry: &Registry) -> Self {
        let game = GameService::new(registry.game().clone());
        let devices = DeviceService::new(registry.devices().clone());

        Self { game, devices }
    }

    /// Returns a reference to the [`GameService`].
    #[must_use]
    pub fn game(&self) -> &GameService {
        &self.game
    }

    /// Returns a reference to the [`DeviceService`].
    #[must_use]
    pub fn devices(&self) -> &DeviceService {
        &self.devices
    }
}
