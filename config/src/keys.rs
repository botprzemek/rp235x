use core::fmt;
use core::ptr;
use core::sync::atomic::{Ordering, compiler_fence};

const AES_ENV_KEY: &[u8] = b"AES_SECRET_KEY\0";
const ED_ENV_KEY: &[u8] = b"ED25519_SECRET_KEY\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyError {
    MissingEnvironmentVariable,
    InvalidHexEncoding,
    InvalidKeyLength,
    MemoryLockFailed,
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyError::MissingEnvironmentVariable => {
                write!(f, "Brak wymaganej zmiennej środowiskowej")
            }
            KeyError::InvalidHexEncoding => write!(f, "Niepoprawne kodowanie heksadecymalne"),
            KeyError::InvalidKeyLength => {
                write!(f, "Błędna długość klucza (wymagane dokładnie 64 znaki Hex)")
            }
            KeyError::MemoryLockFailed => {
                write!(f, "Nie udało się zablokować strony w pamięci RAM (mlock)")
            }
        }
    }
}

pub struct SecureKey {
    bytes: [u8; 32],
}

impl SecureKey {
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
        compiler_fence(Ordering::SeqCst);

        #[cfg(unix)]
        unsafe {
            libc::munlock(self.bytes.as_ptr() as *const libc::c_void, 32);
        }
    }
}

pub struct Keys;

impl Keys {
    pub fn read() -> Result<(SecureKey, SecureKey), KeyError> {
        let mut aes_key = SecureKey { bytes: [0u8; 32] };
        let mut ed_key = SecureKey { bytes: [0u8; 32] };

        Self::lock_memory(&mut aes_key.bytes)?;
        Self::lock_memory(&mut ed_key.bytes)?;

        Self::read_env_key(
            AES_ENV_KEY.as_ptr() as *const libc::c_char,
            &mut aes_key.bytes,
        )?;
        Self::read_env_key(
            ED_ENV_KEY.as_ptr() as *const libc::c_char,
            &mut ed_key.bytes,
        )?;

        Ok((aes_key, ed_key))
    }

    #[inline(always)]
    fn lock_memory(bytes: &mut [u8; 32]) -> Result<(), KeyError> {
        #[cfg(unix)]
        unsafe {
            if libc::mlock(bytes.as_ptr() as *const libc::c_void, 32) != 0 {
                return Err(KeyError::MemoryLockFailed);
            }
        }

        Ok(())
    }

    fn read_env_key(
        env_var_c_str: *const libc::c_char,
        out: &mut [u8; 32],
    ) -> Result<(), KeyError> {
        let raw_ptr = unsafe { libc::getenv(env_var_c_str) };
        if raw_ptr.is_null() {
            return Err(KeyError::MissingEnvironmentVariable);
        }

        let bytes = unsafe { std::ffi::CStr::from_ptr(raw_ptr) }.to_bytes();
        if bytes.len() != 64 {
            return Err(KeyError::InvalidKeyLength);
        }

        Self::decode(bytes, out)?;

        Ok(())
    }

    #[inline(always)]
    fn decode(bytes: &[u8], out: &mut [u8; 32]) -> Result<(), KeyError> {
        let mut error: u8 = 0;

        for i in 0..32 {
            let high = Self::char_to_nibble(bytes[i * 2]);
            let low = Self::char_to_nibble(bytes[i * 2 + 1]);

            error |= high | low;

            out[i] = (high << 4) | (low & 0x0F);
        }

        if (error & 0x80) != 0 {
            unsafe {
                ptr::write_volatile(out, [0u8; 32]);
            }

            return Err(KeyError::InvalidHexEncoding);
        }

        Ok(())
    }

    #[inline(always)]
    fn char_to_nibble(char: u8) -> u8 {
        let digit = char.wrapping_sub(b'0');
        let lower = char | 0x20;

        let is_digit = char.is_ascii_digit() as u8;
        let is_alpha = (b'a'..=b'f').contains(&lower) as u8;

        let value_digit = digit;
        let value_alpha = lower.wrapping_sub(b'a').wrapping_add(10);

        let result = (is_digit * value_digit) | (is_alpha * value_alpha);
        let valid = is_digit | is_alpha;

        (result & (valid.wrapping_neg())) | ((!valid & 1) << 7)
    }
}
