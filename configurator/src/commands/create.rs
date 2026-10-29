//! `create` subcommand for compiling a JSON configuration into an encrypted binary file.

use std::path::PathBuf;

use clap::Args;
use config::file::{BinWriter, JsonReader};
use config::print::Print;
use config::{Config, ConfigError};

/// Arguments for the `create` subcommand.
#[derive(Args)]
#[command(about = "Create an encrypted binary config from JSON", long_about = None)]
pub struct CreateArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(default_value = "config.json")]
    pub input: PathBuf,

    #[arg(short, long, default_value = "config.bin")]
    pub output: PathBuf,
}

/// Handler for the `create` subcommand.
pub struct CreateCommand;

impl CreateCommand {
    /// Reads a JSON configuration from `args.input` and writes the encrypted binary to `args.output`.
    pub fn handle(args: &CreateArgs) -> Result<(), ConfigError> {
        let config = Config::read(&args.input)?;

        if args.verbose {
            config.print()?;
        }

        config.write(&args.output)?;

        Ok(())
    }
}
