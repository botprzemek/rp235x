//! Wire packet trait and client/server packet implementations.

pub mod base;
pub mod client;
pub mod server;

pub use base::Packet;
#[cfg(feature = "serde")]
pub use base::serialize_data;
pub use client::ClientPacket;
pub use server::ServerPacket;
