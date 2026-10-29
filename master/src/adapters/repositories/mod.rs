//! Repository abstractions and concrete persistence implementations.

mod device;
mod game;
mod registry;

pub use device::DeviceRepository;
pub use game::GameRepository;
pub use registry::{Registry, Repository};
