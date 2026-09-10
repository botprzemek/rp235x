use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::vec::Vec;

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, Generate},
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

#[cfg(feature = "serde")]
use crate::ConfigInput;
use crate::{Config, ConfigError};

pub trait FileReader {
    #[cfg(feature = "serde")]
    fn read_json(path: &Path) -> Result<Config, ConfigError>;
    fn read_bin(
        path: &Path,
        public_key: &[u8; 32],
        decryption_key: &[u8; 32],
    ) -> Result<Config, ConfigError>;
}

pub trait FileWriter {
    fn write_bin(
        &self,
        path: &Path,
        signing_key: &[u8; 32],
        encryption_key: &[u8; 32],
    ) -> Result<(), ConfigError>;
}

impl FileReader for Config {
    #[cfg(feature = "serde")]
    fn read_json(path: &Path) -> Result<Self, ConfigError> {
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
        };

        let content = std::fs::read_to_string(path)?;
        let input: ConfigInput =
            serde_json::from_str(&content).map_err(|_| ConfigError::Serialization)?;

        Config::new(
            &input.serial_number,
            &input.device_id,
            &input.wifi_ssid,
            &input.wifi_password,
        )
    }

    fn read_bin(
        path: &Path,
        public_key: &[u8; 32],
        decryption_key: &[u8; 32],
    ) -> Result<Self, ConfigError> {
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
        };

        let mut file = File::open(path)?;
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes)?;

        if file_bytes.len() < 76 {
            return Err(ConfigError::IntegrityCheckFailed);
        }

        let (sig_bytes, rest) = file_bytes.split_at(64);
        let (nonce_bytes, ciphertext) = rest.split_at(12);

        let signature = Signature::from_bytes(
            sig_bytes
                .try_into()
                .map_err(|_| ConfigError::IntegrityCheckFailed)?,
        );

        let verifying_key =
            VerifyingKey::from_bytes(public_key).map_err(|_| ConfigError::IntegrityCheckFailed)?;

        verifying_key
            .verify(rest, &signature)
            .map_err(|_| ConfigError::IntegrityCheckFailed)?;

        let cipher = Aes256Gcm::new(decryption_key.into());
        let nonce = Nonce::try_from(nonce_bytes).map_err(|_| ConfigError::IntegrityCheckFailed)?;
        let bytes = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| ConfigError::IntegrityCheckFailed)?;

        Config::from_bytes(&bytes).cloned()
    }
}

impl FileWriter for Config {
    fn write_bin(
        &self,
        path: &Path,
        signing_key: &[u8; 32],
        encryption_key: &[u8; 32],
    ) -> Result<(), ConfigError> {
        let bytes = self.as_bytes();
        let cipher = Aes256Gcm::new(encryption_key.into());
        let nonce = Nonce::generate();
        let ciphertext = cipher
            .encrypt(&nonce, bytes.as_ref())
            .map_err(|_| ConfigError::Serialization)?;

        let mut signed_payload = Vec::new();
        signed_payload.extend_from_slice(&nonce);
        signed_payload.extend_from_slice(&ciphertext);

        let signing_key = SigningKey::from_bytes(signing_key);
        let signature: Signature = signing_key.sign(&signed_payload);

        let mut file = File::create(path)?;
        file.write_all(signature.to_bytes().as_ref())?;
        file.write_all(&signed_payload)?;
        file.sync_all().map_err(|e| ConfigError::Io(e.kind()))?;

        Ok(())
    }
}
