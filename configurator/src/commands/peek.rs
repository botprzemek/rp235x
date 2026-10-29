//! `peek` subcommand for reading and displaying the configuration from target flash memory.

use clap::Args;
use config::flash::FlashReader;
use config::print::Print;
use config::{Config, ConfigError};

/// Arguments for the `peek` subcommand.
#[derive(Args)]
#[command(about = "Read configuration from target flash memory", long_about = None)]
pub struct PeekArgs {
    #[arg(short, long, default_value = "true")]
    pub verbose: bool,
}

/// Handler for the `peek` subcommand.
pub struct PeekCommand;

impl PeekCommand {
    /// Reads the configuration from the attached RP2350 probe and optionally prints it.
    pub fn handle(args: &PeekArgs) -> Result<(), ConfigError> {
        let config = Config::read()?;

        if args.verbose {
            config.print()?;
        }

        Ok(())
    }
}
