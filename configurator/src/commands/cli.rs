//! [`Commands`] enum and subcommand dispatcher.

use clap::Subcommand;
use config::ConfigError;

use crate::commands::{
    CreateArgs, CreateCommand, PeekArgs, PeekCommand, VerifyArgs, VerifyCommand, ViewArgs,
    ViewCommand, WriteArgs, WriteCommand,
};

/// Supported CLI subcommands.
#[derive(Subcommand)]
pub enum Commands {
    Create(CreateArgs),
    Verify(VerifyArgs),
    Peek(PeekArgs),
    View(ViewArgs),
    Write(WriteArgs),
}

impl Commands {
    /// Dispatches the subcommand to its handler.
    pub fn run(self) -> Result<(), ConfigError> {
        use Commands::*;

        match self {
            Create(args) => CreateCommand::handle(&args),
            Verify(args) => VerifyCommand::handle(&args),
            Peek(args) => PeekCommand::handle(&args),
            View(args) => ViewCommand::handle(&args),
            Write(args) => WriteCommand::handle(&args),
        }
    }
}
