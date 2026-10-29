//! Embedded ReDB database provider and table definitions.

use std::path::Path;
use std::sync::Arc;

use redb::{Builder, Database, TableDefinition};

const DATABASE_CACHE_SIZE: usize = 4 * 1024 * 1024;

/// ReDB table storing serialized [`basketball::Game`] entries keyed by game ID string.
pub const GAME_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("games");

/// ReDB table storing serialized [`net::Device`] entries keyed by serial number.
pub const DEVICES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("devices");

/// Shared connection provider for the local `master.redb` database.
#[derive(Clone)]
pub struct ReDBProvider {
    connection: Arc<Database>,
}

impl Default for ReDBProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ReDBProvider {
    /// Opens or creates `master.redb` and ensures the required tables exist.
    #[must_use]
    pub fn new() -> Self {
        let database_path = Path::new("master.redb");
        let connection = Builder::new()
            .set_cache_size(DATABASE_CACHE_SIZE)
            .create(database_path)
            .expect("failed to open or create master.redb database");
        let connection = Arc::new(connection);

        {
            let write_transaction = connection
                .begin_write()
                .expect("failed to begin initial ReDB write transaction");
            write_transaction
                .open_table(GAME_TABLE)
                .expect("failed to initialize games table");
            write_transaction
                .open_table(DEVICES_TABLE)
                .expect("failed to initialize devices table");
            write_transaction
                .commit()
                .expect("failed to commit initial ReDB table creation");
        }

        Self { connection }
    }

    /// Returns a cloned [`Arc`] handle to the underlying [`Database`].
    #[must_use]
    pub fn connection(&self) -> Arc<Database> {
        Arc::clone(&self.connection)
    }
}
