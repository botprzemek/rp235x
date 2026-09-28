mod game;

pub use game::GameRepository;

use crate::adapters::data::ReDBProvider;

pub struct Registry {
    game: GameRepository,
    _game: GameRepository,
}

pub trait Repository<Entity> {
    fn upsert(&self, entity: &Entity);
    fn select(&self) -> Entity;
}

impl Registry {
    pub fn new(database: ReDBProvider) -> Self {
        let game = GameRepository::new(database.clone());
        let _game = GameRepository::new(database.clone());

        Self { game, _game }
    }

    pub fn game(&self) -> &GameRepository {
        &self.game
    }
}
