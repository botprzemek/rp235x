use basketball::Game;
use tokio::sync::{broadcast, watch};

use crate::{
    adapters::repositories::{GameRepository, Repository},
    services::Observable,
};

#[derive(Clone)]
pub struct GameService {
    game_repository: GameRepository,
    state: watch::Sender<Game>,
    broadcast: broadcast::Sender<Game>,
}

impl Observable<Game> for GameService {
    fn subscribe(&self) -> broadcast::Receiver<Game> {
        self.broadcast.subscribe()
    }

    fn watch(&self) -> watch::Receiver<Game> {
        self.state.subscribe()
    }
}

impl GameService {
    pub fn get(&self) -> Game {
        self.game_repository.select()
    }

    pub fn update(&self, game: Game) {
        self.game_repository.upsert(game);
        self.state.send(game).unwrap();
        self.broadcast.send(game).unwrap();
    }
}

impl GameService {
    pub fn new(game_repository: GameRepository) -> Self {
        let game = basketball::Game::new(basketball::Discipline::Fiba5vs5);
        game_repository.upsert(game);
        let (state, _state_rx) = watch::channel(game_repository.select());
        let (broadcast, _broadcast_rx) = broadcast::channel(100);

        Self {
            game_repository,
            state,
            broadcast,
        }
    }
}
