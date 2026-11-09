//! `reboot` subcommand for resetting and restarting the connected RP2350 target.

use std::time::Duration;

use clap::Args;
use config::ConfigError;
use config::probe_rs::Permissions;
use config::probe_rs::probe::list::Lister;

/// Arguments for the `reboot` subcommand.
#[derive(Args)]
#[command(about = "Reset and reboot the connected RP2350 worker target", long_about = None)]
pub struct RebootArgs {
    /// SWD clock speed in kHz (defaults to 5000 kHz = 5 MHz)
    #[arg(short, long, default_value_t = 5000)]
    pub speed: u32,

    /// Halt core 0 after reset instead of resuming execution
    #[arg(long, default_value_t = false)]
    pub halt: bool,
}

/// Handler for the `reboot` subcommand.
pub struct RebootCommand;

impl RebootCommand {
    /// Connects to the RP2350 target and triggers a core 0 reset and run.
    pub fn handle(args: &RebootArgs) -> Result<(), ConfigError> {
        let lister = Lister::new();
        let probes = lister.list_all();

        if probes.is_empty() {
            eprintln!("Error: no debug probes found");
            return Err(ConfigError::FileNotFound);
        }

        println!("Opening probe: {}", probes[0].identifier);
        let mut probe = lister.open(&probes[0]).map_err(|error| {
            eprintln!("Failed to open debug probe: {error:?}");
            ConfigError::FileNotFound
        })?;

        let _ = probe.set_speed(args.speed);

        println!("Attaching to RP2350 target...");
        let mut session = probe
            .attach("RP2350", Permissions::default())
            .map_err(|error| {
                eprintln!("Failed to attach to RP2350 target: {error:?}");
                ConfigError::FileNotFound
            })?;

        let mut core = session.core(0).map_err(|error| {
            eprintln!("Failed to attach to core 0: {error:?}");
            ConfigError::FileNotFound
        })?;

        println!("Resetting target...");
        core.reset_and_halt(Duration::from_millis(100))
            .map_err(|error| {
                eprintln!("Failed to reset core 0: {error:?}");
                ConfigError::FileNotFound
            })?;

        if args.halt {
            println!("Target reset and halted.");
        } else {
            core.run().map_err(|error| {
                eprintln!("Failed to resume core 0: {error:?}");
                ConfigError::FileNotFound
            })?;
            println!("Target rebooted successfully!");
        }

        Ok(())
    }
}
