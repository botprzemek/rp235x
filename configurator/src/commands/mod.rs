//! Subcommand definitions and dispatch for the `configurator` CLI.

mod cli;
mod create;
mod flash;
mod peek;
mod verify;
mod view;
mod write;

pub use cli::Commands;
pub use create::{CreateArgs, CreateCommand};
pub use flash::{FlashArgs, FlashCommand};
pub use peek::{PeekArgs, PeekCommand};
pub use verify::{VerifyArgs, VerifyCommand};
pub use view::{ViewArgs, ViewCommand};
pub use write::{WriteArgs, WriteCommand};
