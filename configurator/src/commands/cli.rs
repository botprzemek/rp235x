//! [`Commands`] enum and subcommand dispatcher.

use crate::commands::{
    CreateArgs, CreateCommand, FlashArgs, FlashCommand, PeekArgs, PeekCommand, VerifyArgs,
    VerifyCommand, ViewArgs, ViewCommand, WriteArgs, WriteCommand,
};

use clap::Subcommand;
use config::ConfigError;

/// Supported CLI subcommands.
#[derive(Subcommand)]
pub enum Commands {
    Create(CreateArgs),
    Verify(VerifyArgs),
    Peek(PeekArgs),
    View(ViewArgs),
    Write(WriteArgs),
    Flash(FlashArgs),
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
            Flash(args) => FlashCommand::handle(&args),
        }
    }
}
