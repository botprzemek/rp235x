//! Wire packet trait and client/server packet implementations.

pub mod base;
pub mod client;
pub mod server;

#[cfg(feature = "serde")]
pub(crate) use base::serialize_data;
pub use base::{Packet, crc16_ccitt};
pub use client::ClientPacket;
pub use server::ServerPacket;
