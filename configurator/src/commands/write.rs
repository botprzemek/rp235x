//! `write` subcommand for flashing a verified binary configuration onto an RP2350 device.

use std::path::PathBuf;

use clap::Args;
use config::file::BinReader;
use config::flash::FlashWriter;
use config::print::Print;
use config::{Config, ConfigError};

/// Arguments for the `write` subcommand.
#[derive(Args)]
#[command(about = "Flash a binary config file to the target device", long_about = None)]
pub struct WriteArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(default_value = "config.bin")]
    pub path: PathBuf,
}

/// Handler for the `write` subcommand.
pub struct WriteCommand;

impl WriteCommand {
    /// Reads the configuration from `args.path` and flashes it to the connected RP2350 target.
    pub fn handle(args: &WriteArgs) -> Result<(), ConfigError> {
        let config = Config::read(&args.path)?;

        if args.verbose {
            config.print()?;
        }

        config.write();

        Ok(())
    }
}
