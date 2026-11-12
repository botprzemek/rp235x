use core::fmt;
use core::ptr;
use core::sync::atomic::{self, Ordering};
use std::ffi::CStr;

const AES_ENV_KEY: &[u8] = b"AES_SECRET_KEY\0";
const ED25519_ENV_KEY: &[u8] = b"ED25519_SECRET_KEY\0";

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyError {
    MissingEnvironmentVariable,
    InvalidHexEncoding,
    InvalidKeyLength,
    MemoryLockFailed,
}

impl fmt::Display for KeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnvironmentVariable => {
                write!(formatter, "Brak wymaganej zmiennej środowiskowej")
            }
            Self::InvalidHexEncoding => write!(formatter, "Niepoprawne kodowanie heksadecymalne"),
            Self::InvalidKeyLength => {
                write!(
                    formatter,
                    "Błędna długość klucza (wymagane dokładnie 64 znaki Hex)"
                )
            }
            Self::MemoryLockFailed => {
                write!(
                    formatter,
                    "Nie udało się zablokować strony w pamięci RAM (mlock)"
                )
            }
        }
    }
}

pub struct SecureKey {
    bytes: [u8; 32],
}

impl SecureKey {

    #[must_use]
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl Drop for SecureKey {
    fn drop(&mut self) {

        unsafe {
            ptr::write_volatile(&mut self.bytes, [0u8; 32]);
        }
        atomic::compiler_fence(Ordering::SeqCst);

        #[cfg(unix)]

        unsafe {
            libc::munlock(self.bytes.as_ptr().cast::<libc::c_void>(), 32);
        }
    }
}

#[non_exhaustive]
pub struct KeyPair {
    pub ed25519: SecureKey,
    pub aes: SecureKey,
}

pub struct Keys;

impl Keys {

    pub fn read() -> Result<KeyPair, KeyError> {
        let mut aes = SecureKey { bytes: [0u8; 32] };
        let mut ed25519 = SecureKey { bytes: [0u8; 32] };

        Self::lock_memory(&mut aes.bytes)?;
        Self::lock_memory(&mut ed25519.bytes)?;

        Self::read_env_key(AES_ENV_KEY.as_ptr().cast::<libc::c_char>(), &mut aes.bytes)?;
        Self::read_env_key(
            ED25519_ENV_KEY.as_ptr().cast::<libc::c_char>(),
            &mut ed25519.bytes,
        )?;

        Ok(KeyPair { ed25519, aes })
    }

    #[inline(always)]
    fn lock_memory(bytes: &mut [u8; 32]) -> Result<(), KeyError> {
        #[cfg(unix)]

        unsafe {
            if libc::mlock(bytes.as_ptr().cast::<libc::c_void>(), 32) != 0 {
                return Err(KeyError::MemoryLockFailed);
            }
        }

        Ok(())
    }

    fn read_env_key(
        env_var_c_str: *const libc::c_char,
        output: &mut [u8; 32],
    ) -> Result<(), KeyError> {

        let raw_ptr = unsafe { libc::getenv(env_var_c_str) };
        if raw_ptr.is_null() {
            return Err(KeyError::MissingEnvironmentVariable);
        }

        let bytes = unsafe { CStr::from_ptr(raw_ptr) }.to_bytes();
        if bytes.len() != 64 {
            return Err(KeyError::InvalidKeyLength);
        }

        Self::decode(bytes, output)?;

        Ok(())
    }

    #[inline(always)]
    fn decode(bytes: &[u8], output: &mut [u8; 32]) -> Result<(), KeyError> {
        let mut error: u8 = 0;

        for index in 0..32 {
            let high = Self::char_to_nibble(bytes[index * 2]);
            let low = Self::char_to_nibble(bytes[index * 2 + 1]);

            error |= high | low;

            output[index] = (high << 4) | (low & 0x0f);
        }

        if (error & 0x80) != 0 {

            unsafe {
                ptr::write_volatile(output, [0u8; 32]);
            }

            return Err(KeyError::InvalidHexEncoding);
        }

        Ok(())
    }

    #[inline(always)]
    fn char_to_nibble(character: u8) -> u8 {
        let digit = character.wrapping_sub(b'0');
        let lower = character | 0x20;

        let is_digit = u8::from(character.is_ascii_digit());
        let is_alpha = u8::from((b'a'..=b'f').contains(&lower));

        let value_digit = digit;
        let value_alpha = lower.wrapping_sub(b'a').wrapping_add(10);

        let result = (is_digit * value_digit) | (is_alpha * value_alpha);
        let valid = is_digit | is_alpha;

        (result & valid.wrapping_neg()) | ((!valid & 1) << 7)
    }
}
