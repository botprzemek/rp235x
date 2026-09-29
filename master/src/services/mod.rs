mod game;

pub use game::GameService;
use tokio::sync::{broadcast, watch};

use crate::adapters::repositories::Registry;

#[derive(Clone)]
pub struct Services {
    game: GameService,
    _game: GameService,
}

pub trait Observable<E> {
    fn subscribe(&self) -> broadcast::Receiver<E>;
    fn watch(&self) -> watch::Receiver<E>;
}

impl Services {
    pub fn new(registry: Registry) -> Self {
        let game = GameService::new(registry.game().clone());
        let _game = GameService::new(registry.game().clone());

        Self { game, _game }
    }

    pub fn game(&self) -> &GameService {
        &self.game
    }
}
