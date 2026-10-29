//! [`Repository`] trait and [`Registry`] container for persistence adapters.

use crate::adapters::data::ReDBProvider;
use crate::adapters::repositories::{DeviceRepository, GameRepository};

/// Container holding all persistence repositories.
pub struct Registry {
    game: GameRepository,
    devices: DeviceRepository,
}

/// Generic persistence interface for loading and storing `Entity` collections.
pub trait Repository<Entity: ?Sized> {
    /// Owned representation returned by [`Repository::select`].
    type Output;

    /// Selects and returns the stored entities.
    fn select(&self) -> Self::Output;

    /// Replaces the stored entities with `entity`.
    fn upsert(&self, entity: &Entity);
}

impl Registry {
    /// Creates a new [`Registry`] backed by `database`.
    #[must_use]
    pub fn new(database: ReDBProvider) -> Self {
        let game = GameRepository::new(database.clone());
        let devices = DeviceRepository::new(database);

        Self { game, devices }
    }

    /// Returns a reference to the [`GameRepository`].
    #[must_use]
    pub fn game(&self) -> &GameRepository {
        &self.game
    }

    /// Returns a reference to the [`DeviceRepository`].
    #[must_use]
    pub fn devices(&self) -> &DeviceRepository {
        &self.devices
    }
}
