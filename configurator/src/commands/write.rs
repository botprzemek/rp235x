use std::path::PathBuf;

use clap::Args;
use config::{Config, ConfigError, file::BinReader, flash::FlashWriter, print::Print};

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct WriteArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(default_value = "config.bin")]
    pub path: PathBuf,
}

pub struct WriteCommand;

impl WriteCommand {
    pub fn handle(args: WriteArgs) -> Result<(), ConfigError> {
        let config = Config::read(&args.path)?;

        if args.verbose {
            config.print()?;
        }

        config.write();

        Ok(())
    }
}
