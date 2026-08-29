use config::{CONFIG_ADDR, CONFIG_SIZE, Config, ConfigError, print::Print};

use anyhow::Error;
use clap::Args;

#[derive(Args)]
#[command(about="TODO", long_about = None)]
pub struct PeekArgs {}

pub struct PeekCommand;

impl PeekCommand {
    pub fn handle(_args: PeekArgs) -> Result<(), Error> {
        let config = read_flash()?;

        config.print()?;

        Ok(())
    }
}

fn read_flash() -> Result<Config, ConfigError> {
    use probe_rs::MemoryInterface;
    use probe_rs::Permissions;
    use probe_rs::probe;

    let lister = probe::list::Lister::new();
    let probes = lister.list_all();

    let probe = probes[0].open().unwrap();

    let mut session = probe.attach("RP2350", Permissions::default()).unwrap();
    let mut core = session.core(0).unwrap();

    let mut buffer = [0u8; CONFIG_SIZE];
    core.read_8(CONFIG_ADDR.try_into().unwrap(), &mut buffer)
        .unwrap();

    Config::from_bytes(&buffer).cloned()
}
