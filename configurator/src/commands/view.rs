use std::path::PathBuf;

use clap::Args;
use config::{Config, ConfigError, file::BinReader, print::Print};

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct ViewArgs {
    #[arg(default_value = "config.bin")]
    pub path: PathBuf,
}

pub struct ViewCommand;

impl ViewCommand {
    pub fn handle(args: ViewArgs) -> Result<(), ConfigError> {
        let config = Config::read(&args.path)?;

        config.print()?;

        Ok(())
    }
}
