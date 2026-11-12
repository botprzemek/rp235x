pub mod flash;
mod net;
mod split;
mod trng;

pub use hub75::LedPeripherals;
pub use net::NetPeripherals;
pub use split::split;
pub use trng::TrngPeripherals;
