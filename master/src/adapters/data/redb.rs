use std::path::Path;
use std::sync::Arc;

use redb::{Database, TableDefinition};

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
    pub fn new() -> Self {
        let database_path = Path::new("master.redb");
        let connection = Database::create(database_path).unwrap();
        let connection = Arc::new(connection);

        {
            let write_tx = connection.begin_write().unwrap();
            write_tx.open_table(GAME_TABLE).unwrap();
            write_tx.open_table(DEVICES_TABLE).unwrap();
            write_tx.commit().unwrap();
        }

        Self { connection }
    }

    pub fn get_connection(&self) -> Arc<Database> {
        Arc::clone(&self.connection)
    }
}
