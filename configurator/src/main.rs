mod commands;

use clap::Parser;

use commands::Commands;

#[derive(Parser)]
#[command(name = "configurator")]
#[command(about = "TODO")]
#[command(long_about = "TODO")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn run(self) {
        let error = match self.command.run() {
            Ok(()) => return,
            Err(error) => error,
        };

        println!("Error: {}", error);
    }
}

pub fn main() {
    Cli::parse().run()
}
