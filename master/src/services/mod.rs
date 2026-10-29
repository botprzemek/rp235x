//! Application services for managing games and connected devices.

mod container;
mod device;
mod game;

pub use container::{Observable, Services};
pub use device::{DeviceGameAssignment, DeviceService};
pub use game::{GameFrame, GameService};
