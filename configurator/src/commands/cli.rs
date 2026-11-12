use crate::commands::{
    AttachArgs, AttachCommand, CreateArgs, CreateCommand, FlashArgs, FlashCommand, PeekArgs,
    PeekCommand, RebootArgs, RebootCommand, VerifyArgs, VerifyCommand, ViewArgs, ViewCommand,
    WriteArgs, WriteCommand,
};

use clap::Subcommand;
use config::ConfigError;

#[derive(Subcommand)]
pub enum Commands {
    Create(CreateArgs),
    Verify(VerifyArgs),
    Peek(PeekArgs),
    View(ViewArgs),
    Write(WriteArgs),
    Flash(FlashArgs),
    Reboot(RebootArgs),
    Attach(AttachArgs),
}

impl Commands {

    pub fn run(self) -> Result<(), ConfigError> {
        use Commands::*;

        match self {
            Create(args) => CreateCommand::handle(&args),
            Verify(args) => VerifyCommand::handle(&args),
            Peek(args) => PeekCommand::handle(&args),
            View(args) => ViewCommand::handle(&args),
            Write(args) => WriteCommand::handle(&args),
            Flash(args) => FlashCommand::handle(&args),
            Reboot(args) => RebootCommand::handle(&args),
            Attach(args) => AttachCommand::handle(&args),
        }
    }
}
