//! File readers and writers for JSON and encrypted binary [`Config`] files.

#[cfg(feature = "serde")]
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::vec::Vec;

use aes_gcm::aead::{Aead, Generate};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};

use crate::{Config, ConfigError, Keys};
#[cfg(feature = "serde")]
use crate::{ConfigBuilder, Input};

/// Reads a [`Config`] from a JSON file on disk.
#[cfg(feature = "serde")]
pub trait JsonReader {
    /// Reads and validates a [`Config`] from the JSON file at `path`.
    fn read(path: &Path) -> Result<Config, ConfigError>;
}

/// Reads an encrypted and signed [`Config`] from a binary file on disk.
pub trait BinReader {
    /// Reads, verifies, and decrypts a [`Config`] from the binary file at `path`.
    fn read(path: &Path) -> Result<Config, ConfigError>;
}

/// Encrypts, signs, and writes a [`Config`] to a binary file on disk.
pub trait BinWriter {
    /// Encrypts, signs, and writes `self` to `path`.
    fn write(&self, path: &Path) -> Result<(), ConfigError>;
}

#[cfg(feature = "serde")]
impl JsonReader for Config {
    fn read(path: &Path) -> Result<Config, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound);
        }

        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or(ConfigError::Serialization)?
            != "json"
        {
            return Err(ConfigError::InvalidEncoding);
        }

        let content = fs::read_to_string(path)?;
        let input: Input =
            serde_json::from_str(&content).map_err(|_| ConfigError::Serialization)?;
        let mut builder = ConfigBuilder::default();

        builder
            .serial_number(&input.serial_number)
            .device_id(&input.device_id)
            .wifi_ssid(&input.wifi_ssid)
            .wifi_password(&input.wifi_password)
            .build()
    }
}

impl BinReader for Config {
    fn read(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound);
        }

        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or(ConfigError::Serialization)?
            != "bin"
        {
            return Err(ConfigError::InvalidEncoding);
        }

        let mut file = File::open(path)?;
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes)?;

        if file_bytes.len() < 80 {
            return Err(ConfigError::IntegrityCheckFailed);
        }

        let (signature_bytes, rest) = file_bytes.split_at(64);
        let (nonce_bytes, ciphertext) = rest.split_at(12);

        let keys = Keys::read().expect("cryptographic environment keys must be configured");
        let verifying_key = SigningKey::from_bytes(keys.ed25519.as_bytes()).verifying_key();

        let signature_array: &[u8; 64] = signature_bytes
            .try_into()
            .map_err(|_| ConfigError::IntegrityCheckFailed)?;
        let signature = Signature::from_bytes(signature_array);

        verifying_key
            .verify(rest, &signature)
            .map_err(|_| ConfigError::IntegrityCheckFailed)?;

        let cipher = Aes256Gcm::new(keys.aes.as_bytes().into());
        let nonce = Nonce::try_from(nonce_bytes).map_err(|_| ConfigError::IntegrityCheckFailed)?;
        let decrypted_payload = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| ConfigError::IntegrityCheckFailed)?;

        if decrypted_payload.len() < 4 {
            return Err(ConfigError::IntegrityCheckFailed);
        }

        let (checksum_bytes, config_bytes) = decrypted_payload.split_at(4);
        let checksum_array: [u8; 4] = checksum_bytes
            .try_into()
            .map_err(|_| ConfigError::IntegrityCheckFailed)?;
        let expected_checksum = u32::from_le_bytes(checksum_array);

        let actual_checksum = crc32fast::hash(config_bytes);
        if expected_checksum != actual_checksum {
            return Err(ConfigError::IntegrityCheckFailed);
        }

        let raw = Config::from_bytes(config_bytes)?;

        raw.verify().cloned()
    }
}

impl BinWriter for Config {
    fn write(&self, path: &Path) -> Result<(), ConfigError> {
        let bytes = self.as_bytes();
        let checksum = crc32fast::hash(bytes);

        let mut plaintext = Vec::with_capacity(4 + bytes.len());
        plaintext.extend_from_slice(&checksum.to_le_bytes());
        plaintext.extend_from_slice(bytes);

        let keys = Keys::read().expect("cryptographic environment keys must be configured");
        let cipher = Aes256Gcm::new(keys.aes.as_bytes().into());
        let nonce = Nonce::generate();
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|_| ConfigError::Serialization)?;

        let mut signed_payload = Vec::new();
        signed_payload.extend_from_slice(&nonce);
        signed_payload.extend_from_slice(&ciphertext);

        let signing_key = SigningKey::from_bytes(keys.ed25519.as_bytes());
        let signature = signing_key.sign(&signed_payload);

        let mut file = File::create(path)?;
        file.write_all(signature.to_bytes().as_ref())?;
        file.write_all(&signed_payload)?;
        file.sync_all()
            .map_err(|error| ConfigError::Io(error.kind()))?;

        Ok(())
    }
}
