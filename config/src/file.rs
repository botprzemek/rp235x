use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::vec::Vec;

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, Generate},
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier};

use crate::{Config, ConfigError, Keys};
#[cfg(feature = "serde")]
use crate::{ConfigBuilder, Input};

#[cfg(feature = "serde")]
pub trait JsonReader {
    fn read(path: &Path) -> Result<Config, ConfigError>;
}

pub trait BinReader {
    fn read(path: &Path) -> Result<Config, ConfigError>;
}

pub trait BinWriter {
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
            .and_then(|ext| ext.to_str())
            .ok_or(ConfigError::Serialization)?
            .ne("json")
        {
            return Err(ConfigError::InvalidEncoding);
        }

        let content = std::fs::read_to_string(path)?;
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
            .and_then(|ext| ext.to_str())
            .ok_or(ConfigError::Serialization)?
            .ne("bin")
        {
            return Err(ConfigError::InvalidEncoding);
        }

        let mut file = File::open(path)?;
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes)?;

        if file_bytes.len() < 80 {
            return Err(ConfigError::IntegrityCheckFailed);
        }

        let (sig_bytes, rest) = file_bytes.split_at(64);
        let (nonce_bytes, ciphertext) = rest.split_at(12);

        let (verifying_key, decryption_key) = Keys::read().unwrap();
        let verifying_key = SigningKey::from_bytes(verifying_key.as_bytes()).verifying_key();

        let signature = Signature::from_bytes(
            sig_bytes
                .try_into()
                .map_err(|_| ConfigError::IntegrityCheckFailed)?,
        );

        verifying_key
            .verify(rest, &signature)
            .map_err(|_| ConfigError::IntegrityCheckFailed)?;

        let cipher = Aes256Gcm::new(decryption_key.as_bytes().into());
        let nonce = Nonce::try_from(nonce_bytes).map_err(|_| ConfigError::IntegrityCheckFailed)?;
        let decrypted_payload = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| ConfigError::IntegrityCheckFailed)?;

        if decrypted_payload.len() < 4 {
            return Err(ConfigError::IntegrityCheckFailed);
        }

        let (checksum_bytes, config_bytes) = decrypted_payload.split_at(4);
        let expected_checksum = u32::from_le_bytes(
            checksum_bytes
                .try_into()
                .map_err(|_| ConfigError::IntegrityCheckFailed)?,
        );

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

        let mut plaintext = Vec::with_capacity(4 + bytes.as_ref().len());
        plaintext.extend_from_slice(&checksum.to_le_bytes());
        plaintext.extend_from_slice(bytes.as_ref());

        let (signing_key, encryption_key) = Keys::read().unwrap();
        let cipher = Aes256Gcm::new(encryption_key.as_bytes().into());
        let nonce = Nonce::generate();
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|_| ConfigError::Serialization)?;

        let mut signed_payload = Vec::new();
        signed_payload.extend_from_slice(&nonce);
        signed_payload.extend_from_slice(&ciphertext);

        let signing_key = SigningKey::from_bytes(signing_key.as_bytes());
        let signature = signing_key.sign(&signed_payload);

        let mut file = File::create(path)?;
        file.write_all(signature.to_bytes().as_ref())?;
        file.write_all(&signed_payload)?;
        file.sync_all().map_err(|e| ConfigError::Io(e.kind()))?;

        Ok(())
    }
}
