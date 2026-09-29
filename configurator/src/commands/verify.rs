use std::path::PathBuf;

use clap::Args;
use config::{Config, ConfigError, file::BinReader, print::Print};

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct VerifyArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(default_value = "config.bin")]
    pub path: PathBuf,
}

pub struct VerifyCommand;

impl VerifyCommand {
    pub fn handle(args: VerifyArgs) -> Result<(), ConfigError> {
        let config = Config::read(&args.path)?;

        if args.verbose {
            config.print()?;
        }

        Ok(())
    }
}
