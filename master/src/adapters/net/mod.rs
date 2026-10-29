//! Network adapters for HTTP, WebSocket, and UDP worker communication.

pub mod connection;
mod handler;
mod http;
mod udp;

pub use connection::{Session, run};
pub use handler::MasterServerHandler;
pub use http::WebDriver;
pub use udp::WorkerDriver;
