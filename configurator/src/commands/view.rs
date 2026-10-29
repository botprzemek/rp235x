//! `view` subcommand for decrypting and printing a binary configuration file.

use std::path::PathBuf;

use clap::Args;
use config::file::BinReader;
use config::print::Print;
use config::{Config, ConfigError};

/// Arguments for the `view` subcommand.
#[derive(Args)]
#[command(about = "Decrypt and display a binary config file", long_about = None)]
pub struct ViewArgs {
    #[arg(default_value = "config.bin")]
    pub path: PathBuf,
}

/// Handler for the `view` subcommand.
pub struct ViewCommand;

impl ViewCommand {
    /// Reads, verifies, and prints the binary configuration at `args.path`.
    pub fn handle(args: &ViewArgs) -> Result<(), ConfigError> {
        let config = Config::read(&args.path)?;

        config.print()?;

        Ok(())
    }
}
