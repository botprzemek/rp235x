//! In-Application Programming (IAP) flash writer for RP2350.

use config::{CONFIG_ADDR, Config, Verified};
use embassy_rp::Peri;
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::peripherals::FLASH;

/// Flash base address for RP2350 XIP.
const FLASH_BASE: usize = 0x1000_0000;
/// Sector size (4KB) for erase operations on RP2350 flash.
const FLASH_SECTOR_SIZE: u32 = 4096;

/// Flash programming page size (256 bytes) on RP2350 flash.
const FLASH_PAGE_SIZE: usize = 256;

/// Total flash size for RP2350 (typically 4MB or 16MB).
const FLASH_CAPACITY: usize = 4 * 1024 * 1024;

/// Writes the provided verified [`Config`] to flash at [`CONFIG_ADDR`].
///
/// # Safety / Execution Context
///
/// This function executes flash erase and program routines in RAM with interrupts disabled.
/// All other cores must not execute from XIP flash while this operation is in progress.
#[allow(clippy::result_unit_err)]
pub fn write_config_to_flash<'d>(
    flash_peri: Peri<'d, FLASH>,
    config: &Config<Verified>,
) -> Result<(), ()> {
    let bytes = config.as_bytes();
    if bytes.len() > FLASH_PAGE_SIZE {
        return Err(());
    }

    let flash_offset = (CONFIG_ADDR - FLASH_BASE) as u32;

    // Buffer padded to 256-byte page alignment
    let mut page_buffer = [0xFFu8; FLASH_PAGE_SIZE];
    page_buffer[..bytes.len()].copy_from_slice(bytes);

    defmt::info!(
        "Flashing config: erasing sector at offset {=u32:#x}, length {=u32}",
        flash_offset,
        FLASH_SECTOR_SIZE
    );

    let mut flash: Flash<'_, FLASH, Blocking, FLASH_CAPACITY> = Flash::new_blocking(flash_peri);

    if let Err(e) = flash.blocking_erase(flash_offset, flash_offset + FLASH_SECTOR_SIZE) {
        defmt::error!("Flash erase failed: {:?}", e);
        return Err(());
    }

    if let Err(e) = flash.blocking_write(flash_offset, &page_buffer) {
        defmt::error!("Flash write failed: {:?}", e);
        return Err(());
    }

    defmt::info!("Flash write complete. Verifying...");

    // Verify written data by reading memory-mapped flash
    let read_ptr = CONFIG_ADDR as *const u8;
    let written = unsafe { core::slice::from_raw_parts(read_ptr, bytes.len()) };
    if written == bytes {
        defmt::info!("Flash verification passed.");
        Ok(())
    } else {
        defmt::error!("Flash verification failed! Memory mismatch.");
        Err(())
    }
}
