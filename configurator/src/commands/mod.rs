mod create;
mod peek;
mod verify;
mod view;
mod write;

use clap::Subcommand;
use config::ConfigError;

use create::{CreateArgs, CreateCommand};
use peek::{PeekArgs, PeekCommand};
use verify::{VerifyArgs, VerifyCommand};
use view::{ViewArgs, ViewCommand};
use write::{WriteArgs, WriteCommand};

#[derive(Subcommand)]
pub enum Commands {
    Create(CreateArgs),
    Verify(VerifyArgs),
    Peek(PeekArgs),
    View(ViewArgs),
    Write(WriteArgs),
}

impl Commands {
    pub fn run(self) -> Result<(), ConfigError> {
        match self {
            Commands::Create(args) => CreateCommand::handle(args),
            Commands::Verify(args) => VerifyCommand::handle(args),
            Commands::Peek(args) => PeekCommand::handle(args),
            Commands::View(args) => ViewCommand::handle(args),
            Commands::Write(args) => WriteCommand::handle(args),
        }
    }
}
