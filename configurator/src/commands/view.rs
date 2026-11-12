use std::path::PathBuf;

use clap::Args;
use config::file::BinReader;
use config::print::Print;
use config::{Config, ConfigError};

#[derive(Args)]
#[command(about = "Decrypt and display a binary config file", long_about = None)]
pub struct ViewArgs {
    #[arg(default_value = "config.bin")]
    pub path: PathBuf,
}

pub struct ViewCommand;

impl ViewCommand {

    pub fn handle(args: &ViewArgs) -> Result<(), ConfigError> {
        let config = Config::read(&args.path)?;

        config.print()?;

        Ok(())
    }
}
