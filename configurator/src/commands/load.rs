use std::path::PathBuf;

use config::DeviceConfig;

use anyhow::Error;
use clap::Args;

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct LoadArgs {
    #[arg(default_value = "firmware_configured.bin")]
    pub path: PathBuf,
}

pub struct LoadCommand;

impl LoadCommand {
    pub fn handle(args: LoadArgs) -> Result<(), Error> {
        let config = DeviceConfig::read_file(&args.path)?;

        config.display();

        Ok(())
    }
}
