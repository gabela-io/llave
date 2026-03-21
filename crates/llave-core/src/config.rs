use crate::error::{LlaveError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use parking_lot::Mutex;

/// Global proxy URL. Set via [`set_proxy`] before creating any [`LlaveClient`].
/// Checked by `LlaveClient::new()` — if set, all requests go through this proxy.
/// Supports HTTP, HTTPS, and SOCKS5 URLs (e.g. `socks5://127.0.0.1:1080`).
static PROXY_URL: Mutex<Option<String>> = Mutex::new(None);

/// Set the global proxy URL used by all subsequent `LlaveClient` instances.
///
/// Resets the shared client so the next call picks up the new proxy.
pub fn set_proxy(url: Option<String>) {
    *PROXY_URL.lock() = url;
    crate::api::reset_shared_client();
}

/// Get the currently configured proxy URL.
pub fn get_proxy() -> Option<String> {
    PROXY_URL.lock().clone()
}

/// XDG-compliant configuration directory
pub fn config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| LlaveError::Config("Cannot determine config directory".into()))?
        .join("llave-cli");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// XDG-compliant data directory
pub fn data_dir() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .ok_or_else(|| LlaveError::Config("Cannot determine data directory".into()))?
        .join("llave-cli");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub nif: Option<String>,
    pub device_id: Option<String>,
    /// HTTP/HTTPS/SOCKS5 proxy URL (e.g. `socks5://127.0.0.1:1080`).
    pub proxy: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_dir()?.join("config.json");
        if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config_dir()?.join("config.json");
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}

/// Validate Spanish NIF/NIE format and check digit.
///
/// DNI format: 8 digits + 1 letter (e.g. `12345678Z`).
/// NIE format: `X`/`Y`/`Z` + 7 digits + 1 letter (e.g. `X1234567L`).
///
/// The check letter is computed as `number % 23` mapped to the sequence
/// `TRWAGMYFPDXBNJZSQVHLCKE`.
pub fn validate_nif(nif: &str) -> Result<String> {
    const CHECK_LETTERS: &[u8; 23] = b"TRWAGMYFPDXBNJZSQVHLCKE";

    let nif = nif.trim().to_uppercase();
    if nif.len() != 9 {
        return Err(LlaveError::InvalidNif(nif));
    }

    let bytes = nif.as_bytes();
    let check_letter = bytes[8];
    if !check_letter.is_ascii_alphabetic() {
        return Err(LlaveError::InvalidNif(nif));
    }

    // Replace NIE prefix with its numeric equivalent for the modulo calculation.
    let numeric_part: &str = match bytes[0] {
        b'0'..=b'9' => &nif[..8],
        b'X' => &nif[1..8], // X = 0, so just the 7 digits
        b'Y' | b'Z' => {
            // Handled below via prefix replacement
            &nif[1..8]
        }
        _ => return Err(LlaveError::InvalidNif(nif)),
    };

    // All characters in the numeric portion must be digits.
    if !numeric_part.bytes().all(|b| b.is_ascii_digit()) {
        return Err(LlaveError::InvalidNif(nif));
    }

    let number: u32 = match bytes[0] {
        b'0'..=b'9' => nif[..8].parse().map_err(|_| LlaveError::InvalidNif(nif.clone()))?,
        b'X' => numeric_part.parse().map_err(|_| LlaveError::InvalidNif(nif.clone()))?,
        b'Y' => {
            let n: u32 = numeric_part.parse().map_err(|_| LlaveError::InvalidNif(nif.clone()))?;
            10_000_000 + n
        }
        b'Z' => {
            let n: u32 = numeric_part.parse().map_err(|_| LlaveError::InvalidNif(nif.clone()))?;
            20_000_000 + n
        }
        _ => return Err(LlaveError::InvalidNif(nif)),
    };

    let expected = CHECK_LETTERS[(number % 23) as usize];
    if check_letter != expected {
        return Err(LlaveError::InvalidNif(nif));
    }

    Ok(nif)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_dni() {
        // 12345678 % 23 = 14 → CHECK_LETTERS[14] = 'Z'
        assert_eq!(validate_nif("12345678Z").unwrap(), "12345678Z");
    }

    #[test]
    fn valid_nie_x() {
        // X prefix = 0, so 1234567 % 23 = 19 → 'L'
        assert_eq!(validate_nif("X1234567L").unwrap(), "X1234567L");
    }

    #[test]
    fn valid_nie_y() {
        // Y prefix = 10_000_000, so 11234567 % 23 = 10 → 'X'
        assert_eq!(validate_nif("Y1234567X").unwrap(), "Y1234567X");
    }

    #[test]
    fn valid_nie_z() {
        // Z prefix = 20_000_000, so 21234567 % 23 = 1 → 'R'
        assert_eq!(validate_nif("Z1234567R").unwrap(), "Z1234567R");
    }

    #[test]
    fn lowercase_normalised() {
        assert_eq!(validate_nif("12345678z").unwrap(), "12345678Z");
    }

    #[test]
    fn whitespace_trimmed() {
        assert_eq!(validate_nif("  12345678Z  ").unwrap(), "12345678Z");
    }

    #[test]
    fn wrong_check_letter() {
        assert!(validate_nif("12345678A").is_err());
    }

    #[test]
    fn too_short() {
        assert!(validate_nif("1234567Z").is_err());
    }

    #[test]
    fn too_long() {
        assert!(validate_nif("123456789ZZ").is_err());
    }

    #[test]
    fn non_digit_in_body() {
        assert!(validate_nif("1234A678Z").is_err());
    }

    #[test]
    fn invalid_prefix() {
        assert!(validate_nif("A1234567L").is_err());
    }

    #[test]
    fn empty_input() {
        assert!(validate_nif("").is_err());
    }

    #[test]
    fn check_letter_not_alphabetic() {
        assert!(validate_nif("123456781").is_err());
    }
}
