//! `flash` subcommand for flashing an ELF or binary firmware file onto the RP2350 device.

use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::Args;
use config::ConfigError;
use config::probe_rs::Permissions;
use config::probe_rs::flashing::{
    BinLoader, BinOptions, DownloadOptions, ElfLoader, ElfOptions, HexLoader, ImageLoader,
    Uf2Loader,
};
use config::probe_rs::probe::list::Lister;

/// Arguments for the `flash` subcommand.
#[derive(Args)]
#[command(about = "Flash an ELF or binary firmware file onto the connected RP2350 target", long_about = None)]
pub struct FlashArgs {
    /// Path to the firmware file (ELF, bin, uf2, or hex)
    pub path: PathBuf,

    /// SWD clock speed in kHz (defaults to 5000 kHz = 5 MHz)
    #[arg(short, long, default_value_t = 5000)]
    pub speed: u32,

    /// Reset and resume core 0 after flashing
    #[arg(short, long, default_value_t = true)]
    pub run: bool,
}

/// Handler for the `flash` subcommand.
pub struct FlashCommand;

impl FlashCommand {
    /// Flashes the firmware file at `args.path` to the connected RP2350 target.
    pub fn handle(args: &FlashArgs) -> Result<(), ConfigError> {
        if !args.path.exists() {
            eprintln!("Error: file not found: {}", args.path.display());
            return Err(ConfigError::FileNotFound);
        }

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

        println!("Flashing {} onto target...", args.path.display());

        let loader = select_loader(&args.path);
        let mut options = DownloadOptions::default();
        options.keep_unwritten_bytes = false;
        options.do_chip_erase = false;

        config::probe_rs::flashing::download_file_with_options(
            &mut session,
            &args.path,
            loader,
            options,
        )
        .map_err(|error| {
            eprintln!("Failed to flash firmware: {error:?}");
            ConfigError::Serialization
        })?;

        println!("Flashing successful!");

        if args.run {
            println!("Resetting and running core 0...");
            let mut core = session.core(0).map_err(|error| {
                eprintln!("Failed to attach to core 0: {error:?}");
                ConfigError::FileNotFound
            })?;

            core.reset_and_halt(Duration::from_millis(100))
                .map_err(|error| {
                    eprintln!("Failed to reset core 0: {error:?}");
                    ConfigError::FileNotFound
                })?;

            core.run().map_err(|error| {
                eprintln!("Failed to resume core 0: {error:?}");
                ConfigError::FileNotFound
            })?;
            println!("Target running.");
        }

        Ok(())
    }
}

fn select_loader(path: &Path) -> Box<dyn ImageLoader> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("hex") | Some("ihex") => Box::new(HexLoader),
        Some("uf2") => Box::new(Uf2Loader),
        Some("bin") => Box::new(BinLoader(BinOptions {
            base_address: Some(0x1000_0000),
            skip: 0,
        })),
        _ => Box::new(ElfLoader(ElfOptions::default())),
    }
}
