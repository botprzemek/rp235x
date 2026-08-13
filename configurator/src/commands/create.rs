use std::path::PathBuf;

use config::DeviceConfig;

use anyhow::Error;
use clap::Args;

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct CreateArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long, default_value = "config.json")]
    pub config: PathBuf,

    #[arg(short, long, default_value = "firmware.bin")]
    pub firmware: PathBuf,

    #[arg(default_value = "firmware_configured.bin")]
    pub output: PathBuf,
}

pub struct CreateCommand;

impl CreateCommand {
    pub fn handle(args: CreateArgs) -> Result<(), Error> {
        let config = DeviceConfig::write_file(&args.config, &args.firmware, &args.output)?;

        if args.verbose {
            config.display();
        }

        Ok(())
    }
}
