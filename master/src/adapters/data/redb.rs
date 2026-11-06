//! Embedded ReDB database provider and table definitions.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use redb::{Builder, Database, TableDefinition};

const DATABASE_CACHE_SIZE: usize = 4 * 1024 * 1024;

/// ReDB table storing serialized [`basketball::Game`] entries keyed by game ID string.
pub const GAME_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("games");

/// ReDB table storing serialized [`net::Device`] entries keyed by serial number.
pub const DEVICES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("devices");

/// Shared connection provider for the local ReDB database.
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
    /// Determines the default database path using environment variables or standard local storage.
    #[must_use]
    pub fn default_path() -> PathBuf {
        if let Ok(path_str) = env::var("MASTER_DB_PATH") {
            return PathBuf::from(path_str);
        }

        // Prioritize existing local database in workspace or current working directory
        if Path::new("master/master.redb").exists() {
            return PathBuf::from("master/master.redb");
        }
        if Path::new("master.redb").exists() {
            return PathBuf::from("master.redb");
        }

        // Standard user data directory fallback
        if let Ok(home) = env::var("HOME") {
            let data_dir = PathBuf::from(home).join(".local/share/scoreboard-master");
            let _ = fs::create_dir_all(&data_dir);
            return data_dir.join("master.redb");
        }

        PathBuf::from("master.redb")
    }

    /// Opens or creates the database at the default path and ensures required tables exist.
    #[must_use]
    pub fn new() -> Self {
        Self::open(&Self::default_path())
    }

    /// Opens or creates ReDB at the specified `database_path` and ensures required tables exist.
    pub fn open(database_path: &Path) -> Self {
        if let Some(parent) = database_path.parent()
            && !parent.as_os_str().is_empty()
        {
            let _ = fs::create_dir_all(parent);
        }

        let connection = Builder::new()
            .set_cache_size(DATABASE_CACHE_SIZE)
            .create(database_path)
            .unwrap_or_else(|err| {
                panic!(
                    "failed to open or create ReDB database at {}: {err}",
                    database_path.display()
                )
            });
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
