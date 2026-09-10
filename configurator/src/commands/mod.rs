mod create;
// mod peek;
// mod verify;
// mod view;
// mod write;

use create::{CreateArgs, CreateCommand};
// use peek::{PeekArgs, PeekCommand};
// use verify::{VerifyArgs, VerifyCommand};
// use view::{ViewArgs, ViewCommand};
// use write::{WriteArgs, WriteCommand};

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    Create(CreateArgs),
    // Peek(PeekArgs),
    // Verify(VerifyArgs),
    // View(ViewArgs),
    // Write(WriteArgs),
}

impl Commands {
    pub fn run(self) -> std::result::Result<(), anyhow::Error> {
        match self {
            Commands::Create(args) => CreateCommand::handle(args),
            // Commands::Peek(args) => PeekCommand::handle(args),
            // Commands::Verify(args) => VerifyCommand::handle(args),
            // Commands::View(args) => ViewCommand::handle(args),
            // Commands::Write(args) => WriteCommand::handle(args),
        }
    }
}
