use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use redb::Database;
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, broadcast, watch};
use tokio::time::{self};

use net::{
    ClientPacket, ServerPacket,
    data::snapshot::{Discipline, Snapshot},
    event::{ClientEvent, ServerEvent},
};

mod database;
mod http;
mod udp;
mod websocket;

const FRAME_DURATION: Duration = Duration::from_millis(64);

#[derive(Clone)]
pub struct AppState {
    tx: broadcast::Sender<String>,
    state_tx: watch::Sender<Snapshot>,
    db: Arc<Database>,
}

impl AppState {
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let database_path = std::path::Path::new("master_state.redb");
    let database = Arc::new(Database::create(database_path).map_err(std::io::Error::other)?);

    {
        let write_tx = database.begin_write().map_err(std::io::Error::other)?;
        let _ = write_tx
            .open_table(database::SNAPSHOT_TABLE)
            .map_err(std::io::Error::other)?;
        write_tx.commit().map_err(std::io::Error::other)?;
    }

    let initial_snapshot =
        database::load(&database).unwrap_or_else(|| Snapshot::new(Discipline::FIBA5V5));

    let (state_tx, state_rx) = watch::channel(initial_snapshot);
    let (tx, _rx) = broadcast::channel(100);

    let udp_socket = Arc::new(UdpSocket::bind("0.0.0.0:9000").await?);
    udp_socket.set_broadcast(true)?;

    let state = AppState {
        tx: tx.clone(),
        state_tx: state_tx.clone(),
        db: Arc::clone(&database),
    };

    websocket::run(state.clone()).await;
    http::run(state.clone()).await;

    Ok(())
}
