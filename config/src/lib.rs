//! Configuration management, cryptographic verification, and flash/file storage for RP235x.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod config;
pub mod flash;
pub mod print;

#[cfg(feature = "std")]
pub mod file;
#[cfg(feature = "std")]
pub mod keys;

#[cfg(feature = "std")]
pub use config::Input;
pub use config::{
    CONFIG_ADDR, CONFIG_SIZE, Config, ConfigBuilder, ConfigError, ConfigField, DeviceConfig,
    NetworkConfig, Raw, Verified,
};
#[cfg(feature = "serde")]
pub use file::JsonReader;
#[cfg(feature = "std")]
pub use file::{BinReader, BinWriter};
#[cfg(feature = "std")]
pub use keys::{KeyError, KeyPair, Keys, SecureKey};
pub use print::Print;
