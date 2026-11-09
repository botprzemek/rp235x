//! Byte offsets and sizes for the [`super::Device`] wire payload.

/// Total payload size in bytes.
pub(crate) const DATA_SIZE: usize = crate::layout::DATA_SIZE;

/// Maximum byte length of the device serial number field.
pub(crate) const SERIAL_NUMBER_SIZE: usize = 12;

/// Maximum byte length of the device identifier field.
pub(crate) const DEVICE_ID_SIZE: usize = 12;

/// Byte index of the [`super::DeviceType`] discriminant.
pub(crate) const DEVICE_TYPE: usize = 0x00;

/// Start byte index (inclusive) of the assigned game ID.
pub(crate) const GAME_ID_START: usize = 0x01;

/// End byte index (exclusive) of the assigned game ID.
pub(crate) const GAME_ID_END: usize = 0x03;

/// Start byte index (inclusive) of the serial number.
pub(crate) const SERIAL_NUMBER_START: usize = 0x03;

/// End byte index (exclusive) of the serial number.
pub(crate) const SERIAL_NUMBER_END: usize = 0x0f;

/// Start byte index (inclusive) of the device ID.
pub(crate) const DEVICE_ID_START: usize = 0x0f;

/// End byte index (exclusive) of the device ID.
pub(crate) const DEVICE_ID_END: usize = 0x1b;
