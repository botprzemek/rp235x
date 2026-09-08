use std::path::PathBuf;

use config::{
    Config,
    file::{FileReader, FileWriter},
    print::Print,
};

use anyhow::Error;
use clap::Args;

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct CreateArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long, default_value = "config.json")]
    pub input: PathBuf,

    #[arg(default_value = "config.bin")]
    pub output: PathBuf,
}

pub struct CreateCommand;

impl CreateCommand {
    pub fn handle(args: CreateArgs) -> Result<(), Error> {
        let config = Config::read_json(&args.input)?;

        if args.verbose {
            config.print()?;
        }

        config.write_file(&args.output)?;

        Ok(())
    }
}
