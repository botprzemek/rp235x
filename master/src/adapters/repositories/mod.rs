mod device;
mod game;

pub use device::DeviceRepository;
pub use game::GameRepository;

use crate::adapters::data::ReDBProvider;

pub struct Registry {
    game: GameRepository,
    devices: DeviceRepository,
}

pub trait Repository<Entity> {
    fn select(&self) -> Entity;
    fn upsert(&self, entity: Entity);
}

impl Registry {
    pub fn new(database: ReDBProvider) -> Self {
        let game = GameRepository::new(database.clone());
        let devices = DeviceRepository::new(database);

        Self { game, devices }
    }

    pub fn game(&self) -> &GameRepository {
        &self.game
    }

    pub fn devices(&self) -> &DeviceRepository {
        &self.devices
    }
}
