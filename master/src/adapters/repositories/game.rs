use basketball::Game;
use redb::ReadableDatabase;

use crate::adapters::data::{GAME_TABLE, ReDBProvider};
use crate::adapters::repositories::Repository;

#[derive(Clone)]
pub struct GameRepository {
    database: ReDBProvider,
}

impl Repository<Game> for GameRepository {
    fn upsert(&self, game: &Game) {
        let bytes = game.to_bytes();
        let transaction = self.database.get_connection().begin_write().unwrap();
        let mut table = transaction.open_table(GAME_TABLE).unwrap();

        table.insert("current", bytes.as_slice()).unwrap();

        drop(table);

        transaction.commit().unwrap();
    }

    fn select(&self) -> Game {
        let transaction = self.database.get_connection().begin_read().unwrap();
        let table = transaction.open_table(GAME_TABLE).unwrap();
        let bytes = table.get("current").unwrap().unwrap();

        Game::from_bytes(bytes.value()).unwrap()
    }
}

impl GameRepository {
    pub fn new(database: ReDBProvider) -> Self {
        Self { database }
    }
}
