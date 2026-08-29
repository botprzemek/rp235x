use std::path::PathBuf;

use config::{Config, ConfigError, file::FileReader};

use anyhow::{Error, anyhow};
use clap::Args;

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct VerifyArgs {
    #[arg(default_value = "config.bin")]
    pub path: PathBuf,
}

pub struct VerifyCommand;

impl VerifyCommand {
    pub fn handle(args: VerifyArgs) -> Result<(), Error> {
        let config = Config::read_bin(&args.path)?;
        if !config.verify_integrity() {
            return Err(anyhow!(ConfigError::IntegrityCheckFailed));
        }

        Ok(())
    }
}
