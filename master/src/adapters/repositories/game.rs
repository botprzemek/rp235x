use basketball::Game;
use redb::{ReadableDatabase, ReadableTable};

use crate::adapters::data::{GAME_TABLE, ReDBProvider};
use crate::adapters::repositories::Repository;

#[derive(Clone)]
pub struct GameRepository {
    database: ReDBProvider,
}

impl Repository<Vec<Game>> for GameRepository {
    fn select(&self) -> Vec<Game> {
        let transaction = self.database.get_connection().begin_read().unwrap();
        let table = transaction.open_table(GAME_TABLE).unwrap();
        let mut games: Vec<Game> = Vec::new();

        for entry in table.iter().unwrap().flatten() {
            if let Ok(mut game) = Game::from_bytes(entry.1.value()) {
                if let Ok(parsed_id) = entry.0.value().parse::<u16>()
                    && parsed_id > 0
                {
                    game.set_id(parsed_id);
                }
                if let Some(existing) = games.iter_mut().find(|g| g.id() == game.id()) {
                    *existing = game;
                } else {
                    games.push(game);
                }
            }
        }

        games.sort_by_key(|g| g.id());
        games
    }

    fn upsert(&self, entities: Vec<Game>) {
        let transaction = self.database.get_connection().begin_write().unwrap();
        let _ = transaction.delete_table(GAME_TABLE);
        let mut table = transaction.open_table(GAME_TABLE).unwrap();

        for game in entities {
            let key = game.id().to_string();
            let bytes = game.to_bytes();
            table.insert(key.as_str(), bytes.as_slice()).unwrap();
        }

        drop(table);
        transaction.commit().unwrap();
    }
}

impl GameRepository {
    pub fn new(database: ReDBProvider) -> Self {
        Self { database }
    }
}
