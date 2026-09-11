use clap::Args;
use config::{Config, ConfigError, flash::FlashReader, print::Print};

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct PeekArgs {
    #[arg(short, long, default_value = "true")]
    pub verbose: bool,
}

pub struct PeekCommand;

impl PeekCommand {
    pub fn handle(args: PeekArgs) -> Result<(), ConfigError> {
        let config = Config::read()?;

        if args.verbose {
            config.print()?;
        }

        Ok(())
    }
}
