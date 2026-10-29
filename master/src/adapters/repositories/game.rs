//! ReDB-backed repository for [`Game`] records.

use basketball::Game;

use crate::adapters::data::{GAME_TABLE, ReDBProvider};
use crate::adapters::repositories::Repository;

/// Persistence adapter for [`Game`] records in ReDB.
#[derive(Clone)]
pub struct GameRepository {
    database: ReDBProvider,
}

impl Repository<[Game]> for GameRepository {
    type Output = Vec<Game>;

    fn select(&self) -> Vec<Game> {
        use redb::{ReadableDatabase, ReadableTable};

        let transaction = self
            .database
            .connection()
            .begin_read()
            .expect("failed to begin ReDB read transaction for games");
        let table = transaction
            .open_table(GAME_TABLE)
            .expect("failed to open games table");
        let mut games: Vec<Game> = Vec::new();

        let entries = table
            .iter()
            .expect("failed to iterate over games table")
            .flatten();
        for (key, value) in entries {
            if let Ok(mut game) = Game::from_bytes(value.value()) {
                if let Ok(parsed_id) = key.value().parse::<u16>()
                    && parsed_id > 0
                {
                    game.set_id(parsed_id);
                }
                if let Some(existing) = games.iter_mut().find(|entry| entry.id() == game.id()) {
                    *existing = game;
                } else {
                    games.push(game);
                }
            }
        }

        games.sort_by_key(Game::id);
        games
    }

    fn upsert(&self, entities: &[Game]) {
        let transaction = self
            .database
            .connection()
            .begin_write()
            .expect("failed to begin ReDB write transaction for games");
        let _ = transaction.delete_table(GAME_TABLE);
        let mut table = transaction
            .open_table(GAME_TABLE)
            .expect("failed to open games table for writing");

        for game in entities {
            let key = game.id().to_string();
            let bytes = game.to_bytes();
            table
                .insert(key.as_str(), bytes.as_slice())
                .expect("failed to insert game into games table");
        }

        drop(table);
        transaction
            .commit()
            .expect("failed to commit games transaction");
    }
}

impl GameRepository {
    /// Creates a new [`GameRepository`] backed by `database`.
    #[must_use]
    pub fn new(database: ReDBProvider) -> Self {
        Self { database }
    }
}
