#[cfg(feature = "std")]
use std::time::Duration;

use crate::{CONFIG_ADDR, CONFIG_SIZE, Config, ConfigError, Raw};
#[cfg(feature = "std")]
use probe_rs::{Permissions, flashing, probe};

pub trait FlashReader<'a> {
    #[cfg(feature = "std")]
    fn read() -> Result<Config, ConfigError>;
    #[cfg(not(feature = "std"))]
    fn read() -> Result<&'a Config, ConfigError>;
}

pub trait FlashWriter {
    #[cfg(feature = "std")]
    fn write(self);
}

impl<'a> FlashReader<'a> for Config {
    #[cfg(feature = "std")]
    fn read() -> Result<Self, ConfigError> {
        use probe_rs::{MemoryInterface, Permissions, probe};

        let mut buffer = [0u8; CONFIG_SIZE];
        let lister = probe::list::Lister::new();
        let probes = lister.list_all();

        let probe = probes[0].open().unwrap();

        let mut session = probe.attach("RP2350", Permissions::default()).unwrap();
        let mut core = session.core(0).unwrap();
        core.read_8(CONFIG_ADDR.try_into().unwrap(), &mut buffer)
            .unwrap();
        let raw = Config::<Raw>::from_bytes(&buffer)?;

        raw.verify().cloned()
    }

    #[cfg(not(feature = "std"))]
    fn read() -> Result<&'a Self, ConfigError> {
        unsafe {
            let bytes = core::slice::from_raw_parts(CONFIG_ADDR as *const u8, CONFIG_SIZE);
            let raw = Config::<Raw>::from_bytes(&bytes)?;

            raw.verify()
        }
    }
}

#[cfg(feature = "std")]
impl FlashWriter for Config {
    fn write(self) {
        let lister = probe::list::Lister::new();
        let probes = lister.list_all();
        let probe = probes[0].open().unwrap();
        let mut session = probe.attach("RP2350", Permissions::default()).unwrap();

        let mut loader = session.target().flash_loader();
        loader
            .add_data(CONFIG_ADDR.try_into().unwrap(), self.as_bytes())
            .unwrap();
        loader
            .commit(&mut session, flashing::DownloadOptions::default())
            .unwrap();

        let mut core = session.core(0).unwrap();
        core.reset_and_halt(Duration::from_millis(500)).unwrap();
        core.run().unwrap();
    }
}
