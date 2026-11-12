mod commands;

use crate::commands::Commands;

use clap::Parser;

#[derive(Parser)]
#[command(name = "configurator")]
#[command(about = "RP235x device configuration tool")]
#[command(
    long_about = "Creates, verifies, views, and flashes encrypted RP235x device configurations."
)]
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

        println!("Error: {error}");
    }
}

fn main() {
    Cli::parse().run();
}
