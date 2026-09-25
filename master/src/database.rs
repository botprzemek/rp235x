use bincode::{deserialize, serialize};
use redb::{Database, ReadableDatabase, TableDefinition};

use net::data::snapshot::Snapshot;

pub const SNAPSHOT_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("snapshot_store");

pub fn save(database: &Database, snapshot: &Snapshot) {
    let bytes = match serialize(snapshot) {
        Ok(bytes) => bytes,
        Err(_) => return,
    };

    let tx = match database.begin_write() {
        Ok(tx) => tx,
        Err(_) => return,
    };

    let mut table = match tx.open_table(SNAPSHOT_TABLE) {
        Ok(table) => table,
        Err(_) => return,
    };

    let _ = table.insert("current", bytes.as_slice());

    drop(table);

    let _ = tx.commit();
}

pub fn load(database: &Database) -> Option<Snapshot> {
    let transaction = database.begin_read().unwrap();
    let table = transaction.open_table(SNAPSHOT_TABLE).unwrap();
    let bytes = table.get("current").unwrap().unwrap();

    deserialize(bytes.value()).unwrap()
}
