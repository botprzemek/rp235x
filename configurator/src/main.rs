mod commands;

use commands::Commands;

use clap::Parser;

const COMMAND_NAME: &str = "configurator";
const COMMAND_ABOUT: &str = "TODO";

#[derive(Parser)]
#[command(name = COMMAND_NAME)]
#[command(about = COMMAND_ABOUT, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn run(self) {
        match self.command.run() {
            Ok(()) => (),
            Err(error) => println!("Error: {}", error),
        }
    }
}

pub fn main() {
    Cli::parse().run()
}
