use std::path::PathBuf;

use config::{CONFIG_ADDR, Config, file::FileReader, print::Print};

use anyhow::{Error, anyhow};
use clap::Args;

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct WriteArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(default_value = "config.bin")]
    pub file: PathBuf,
}

pub struct WriteCommand;

impl WriteCommand {
    pub fn handle(args: WriteArgs) -> Result<(), Error> {
        let config = Config::read_bin(&args.file)?;

        write_flash(&config)?;

        if args.verbose {
            config.print()?;
        }

        Ok(())
    }
}

fn write_flash(config: &Config) -> Result<(), Error> {
    use probe_rs::{Permissions, flashing, probe};
    use std::time::Duration;

    let lister = probe::list::Lister::new();
    let probes = lister.list_all();

    let probe = probes[0].open()?;

    let mut session = probe.attach("RP2350", Permissions::default())?;

    let mut loader = session.target().flash_loader();
    loader.add_data(CONFIG_ADDR.try_into()?, config.as_bytes())?;

    loader.commit(&mut session, flashing::DownloadOptions::default())?;

    let mut core = session.core(0)?;
    core.reset_and_halt(Duration::from_millis(500))?;
    core.run().map_err(|error| anyhow!(error))
}
