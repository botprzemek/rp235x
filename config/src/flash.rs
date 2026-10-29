//! Flash memory reader and writer for [`Config`].

#[cfg(feature = "std")]
use std::time::Duration;

#[cfg(feature = "std")]
use probe_rs::Permissions;
#[cfg(feature = "std")]
use probe_rs::flashing::DownloadOptions;
#[cfg(feature = "std")]
use probe_rs::probe::list::Lister;

use crate::{CONFIG_ADDR, CONFIG_SIZE, Config, ConfigError, Raw};

/// Reads a [`Config`] from target flash memory.
pub trait FlashReader<'flash> {
    /// Reads and verifies the [`Config`] from flash memory.
    #[cfg(feature = "std")]
    fn read() -> Result<Config, ConfigError>;

    /// Reads and verifies a zero-copy reference to [`Config`] in memory-mapped flash.
    #[cfg(not(feature = "std"))]
    fn read() -> Result<&'flash Config, ConfigError>;
}

/// Writes a [`Config`] to target flash memory over a debug probe.
pub trait FlashWriter {
    /// Flashes `self` onto the connected RP2350 target.
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

        let probe = probes[0].open().expect("failed to open debug probe");

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
        // SAFETY: `CONFIG_ADDR` is the reserved flash partition address on RP235x mapped into the
        // address space with at least `CONFIG_SIZE` readable bytes.
        // Justification: Hardware memory-mapped flash access on bare-metal RP235x.
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
        let probe = probes[0].open().expect("failed to open debug probe");
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
