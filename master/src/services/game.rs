use basketball::{Game, Scorable, Score, Team};
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

    pub fn get(&self) -> Game {
        self.game_repository.select()
    }

    pub fn update(&self, game: Game) {
        self.game_repository.upsert(game);

        if self.broadcast.receiver_count() > 0
            && let Err(error) = self.broadcast.send(game)
        {
            eprintln!("ERROR: Failed to broadcast: {}", error);
        };

        if self.state.receiver_count() > 0
            && let Err(error) = self.state.send(game)
        {
            eprintln!("ERROR: Failed to send update: {}", error);
        };
    }

    pub async fn get_scores(&self) -> (u16, u16) {
        let game = self.get();

        game.get_scores()
    }

    pub async fn get_score(&self, team: Team) -> u16 {
        let game = self.get();

        game.get_score(team)
    }

    pub async fn add_score(&self, team: Team, score: Score) {
        let mut game = self.get();

        game.score(team, score);

        self.update(game);
    }

    pub async fn resume(&self) {
        let mut game = self.get();

        game.resume();

        self.update(game);
    }

    pub async fn pause(&self) {
        let mut game = self.get();

        game.pause();

        self.update(game);
    }
}
