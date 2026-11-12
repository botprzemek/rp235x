#[cfg(feature = "std")]
use std::eprintln;
#[cfg(feature = "std")]
use std::time::Duration;

use crate::{CONFIG_ADDR, CONFIG_SIZE, Config, ConfigError, Raw};

#[cfg(feature = "std")]
use probe_rs::Permissions;
#[cfg(feature = "std")]
use probe_rs::flashing::DownloadOptions;
#[cfg(feature = "std")]
use probe_rs::probe::list::Lister;

pub trait FlashReader<'flash> {

    #[cfg(feature = "std")]
    fn read() -> Result<Config, ConfigError>;

    #[cfg(not(feature = "std"))]
    fn read() -> Result<&'flash Config, ConfigError>;
}

pub trait FlashWriter {

    #[cfg(feature = "std")]
    fn write(self);
}

impl<'flash> FlashReader<'flash> for Config {
    #[cfg(feature = "std")]
    fn read() -> Result<Self, ConfigError> {
        use probe_rs::MemoryInterface;

        let mut buffer = [0u8; CONFIG_SIZE];
        let lister = Lister::new();
        let probes = lister.list_all();

        if probes.is_empty() {
            return Err(ConfigError::FileNotFound);
        }

        let mut probe = lister.open(&probes[0]).map_err(|error| {
            eprintln!("failed to open debug probe: {error:?}");
            ConfigError::FileNotFound
        })?;

        let _ = probe.set_speed(5000);

        let mut session = probe
            .attach("RP2350", Permissions::default())
            .expect("failed to attach to RP2350 target");
        let mut core = session.core(0).expect("failed to attach to core 0");
        let address: u64 = CONFIG_ADDR.try_into().expect("CONFIG_ADDR fits into u64");
        core.read_8(address, &mut buffer)
            .expect("failed to read configuration from flash");
        let raw = Config::<Raw>::from_bytes(&buffer)?;

        raw.verify().cloned()
    }

    #[cfg(not(feature = "std"))]
    fn read() -> Result<&'flash Self, ConfigError> {

        unsafe {
            let bytes = core::slice::from_raw_parts(CONFIG_ADDR as *const u8, CONFIG_SIZE);
            let raw = Config::<Raw>::from_bytes(bytes)?;

            raw.verify()
        }
    }
}

#[cfg(feature = "std")]
impl FlashWriter for Config {
    fn write(self) {
        let lister = Lister::new();
        let probes = lister.list_all();

        assert!(!probes.is_empty(), "no debug probes found");

        let mut probe = lister.open(&probes[0]).expect("failed to open debug probe");

        let _ = probe.set_speed(5000);

        let mut session = probe
            .attach("RP2350", Permissions::default())
            .expect("failed to attach to RP2350 target");

        let mut loader = session.target().flash_loader();
        let address: u64 = CONFIG_ADDR.try_into().expect("CONFIG_ADDR fits into u64");
        loader
            .add_data(address, self.as_bytes())
            .expect("failed to stage configuration data for flash loader");
        loader
            .commit(&mut session, DownloadOptions::default())
            .expect("failed to commit configuration data to flash");

        let mut core = session.core(0).expect("failed to attach to core 0");
        core.reset_and_halt(Duration::from_millis(500))
            .expect("failed to reset and halt core 0");
        core.run().expect("failed to resume core 0 execution");
    }
}
