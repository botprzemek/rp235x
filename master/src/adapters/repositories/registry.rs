use crate::adapters::data::ReDBProvider;
use crate::adapters::repositories::{DeviceRepository, GameRepository};

pub struct Registry {
    game: GameRepository,
    devices: DeviceRepository,
}

pub trait Repository<Entity: ?Sized> {
    type Output;

    fn select(&self) -> Self::Output;

    fn upsert(&self, entity: &Entity);
}

impl Registry {
    #[must_use]
    pub fn new(database: ReDBProvider) -> Self {
        let game = GameRepository::new(database.clone());
        let devices = DeviceRepository::new(database);

        Self { game, devices }
    }

    #[must_use]
    pub fn game(&self) -> &GameRepository {
        &self.game
    }

    #[must_use]
    pub fn devices(&self) -> &DeviceRepository {
        &self.devices
    }
}
