//! Peripheral partitioning for the RP235x worker firmware.

mod net;
mod split;
mod trng;

pub use hub75::LedPeripherals;
pub use net::NetPeripherals;
pub use split::split;
pub use trng::TrngPeripherals;
