use std::path::PathBuf;

use config::{Config, file::FileReader, print::Print};

use anyhow::Error;
use clap::Args;

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct ViewArgs {
    #[arg(default_value = "config.bin")]
    pub path: PathBuf,
}

pub struct ViewCommand;

impl ViewCommand {
    pub fn handle(args: ViewArgs) -> Result<(), Error> {
        let config = Config::read_bin(&args.path)?;

        config.print()?;

        Ok(())
    }
}
