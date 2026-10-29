//! `verify` subcommand for validating an encrypted binary configuration file.

use std::path::PathBuf;

use clap::Args;
use config::file::BinReader;
use config::print::Print;
use config::{Config, ConfigError};

/// Arguments for the `verify` subcommand.
#[derive(Args)]
#[command(about = "Verify signature and checksum of a binary config", long_about = None)]
pub struct VerifyArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(default_value = "config.bin")]
    pub path: PathBuf,
}

/// Handler for the `verify` subcommand.
pub struct VerifyCommand;

impl VerifyCommand {
    /// Reads and verifies the encrypted binary configuration at `args.path`.
    pub fn handle(args: &VerifyArgs) -> Result<(), ConfigError> {
        let config = Config::read(&args.path)?;

        if args.verbose {
            config.print()?;
        }

        Ok(())
    }
}
