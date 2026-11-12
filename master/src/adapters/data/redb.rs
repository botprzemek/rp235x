use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use redb::{Builder, Database, TableDefinition};

const DATABASE_CACHE_SIZE: usize = 4 * 1024 * 1024;

pub const GAME_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("games");

pub const DEVICES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("devices");

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

    #[must_use]
    pub fn default_path() -> PathBuf {
        if let Ok(path_str) = env::var("MASTER_DB_PATH") {
            return PathBuf::from(path_str);
        }

        if Path::new("master/master.redb").exists() {
            return PathBuf::from("master/master.redb");
        }
        if Path::new("master.redb").exists() {
            return PathBuf::from("master.redb");
        }

        if let Ok(home) = env::var("HOME") {
            let data_dir = PathBuf::from(home).join(".local/share/scoreboard-master");
            let _ = fs::create_dir_all(&data_dir);
            return data_dir.join("master.redb");
        }

        PathBuf::from("master.redb")
    }

    #[must_use]
    pub fn new() -> Self {
        Self::open(&Self::default_path())
    }

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

    #[must_use]
    pub fn connection(&self) -> Arc<Database> {
        Arc::clone(&self.connection)
    }
}
