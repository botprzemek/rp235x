//! Command-line interface for creating, verifying, viewing, and flashing RP235x configurations.

mod commands;

use clap::Parser;

use crate::commands::Commands;

/// Top-level CLI parser for the `configurator` binary.
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
    /// Executes the selected CLI subcommand.
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
