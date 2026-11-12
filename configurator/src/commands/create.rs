use std::path::PathBuf;

use clap::Args;
use config::file::{BinWriter, JsonReader};
use config::print::Print;
use config::{Config, ConfigError};

#[derive(Args)]
#[command(about = "Create an encrypted binary config from JSON", long_about = None)]
pub struct CreateArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(default_value = "config.json")]
    pub input: PathBuf,

    #[arg(short, long, default_value = "config.bin")]
    pub output: PathBuf,
}

pub struct CreateCommand;

impl CreateCommand {

    pub fn handle(args: &CreateArgs) -> Result<(), ConfigError> {
        let config = Config::read(&args.input)?;

        if args.verbose {
            config.print()?;
        }

        config.write(&args.output)?;

        Ok(())
    }
}
