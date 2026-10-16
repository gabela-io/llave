use crate::error::{LlaveError, Result};
use base64::Engine;
use sha2::{Sha512, Digest};
use zeroize::Zeroizing;

// ---------------------------------------------------------------------------
// PIN-based session encryption (Argon2id + AES-256-GCM)
// ---------------------------------------------------------------------------

/// Current envelope version byte.
const SEALED_VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12; // AES-GCM standard

/// Encrypt session JSON with a user-chosen PIN.
///
/// Format: `base64( version(1) || salt(16) || nonce(12) || ciphertext+tag )`
///
/// The key is derived via Argon2id so offline brute-force is expensive.
pub fn seal_session(session_json: &str, pin: &str) -> Result<String> {
    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
    use rand::RngCore;

    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key = derive_key(pin, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&*key)
        .map_err(|e| LlaveError::Crypto(format!("AES-GCM init: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, session_json.as_bytes())
        .map_err(|e| LlaveError::Crypto(format!("AES-GCM encrypt: {e}")))?;

    // Assemble envelope
    let mut envelope = Vec::with_capacity(1 + SALT_LEN + NONCE_LEN + ciphertext.len());
    envelope.push(SEALED_VERSION);
    envelope.extend_from_slice(&salt);
    envelope.extend_from_slice(&nonce_bytes);
    envelope.extend_from_slice(&ciphertext);

    Ok(base64::engine::general_purpose::STANDARD.encode(&envelope))
}

/// Decrypt a sealed session blob using the user's PIN.
///
/// Returns the original session JSON, or an error if the PIN is wrong
/// (AES-GCM authentication will fail).
pub fn open_session(sealed_b64: &str, pin: &str) -> Result<String> {
    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};

    let envelope = base64::engine::general_purpose::STANDARD
        .decode(sealed_b64)
        .map_err(|e| LlaveError::Crypto(format!("base64 decode: {e}")))?;

    let min_len = 1 + SALT_LEN + NONCE_LEN + 16; // 16 = GCM tag
    if envelope.len() < min_len {
        return Err(LlaveError::Crypto("sealed data too short".into()));
    }

    let version = envelope[0];
    if version != SEALED_VERSION {
        return Err(LlaveError::Crypto(format!(
            "unsupported sealed version: {version}"
        )));
    }

    let salt = &envelope[1..1 + SALT_LEN];
    let nonce_bytes = &envelope[1 + SALT_LEN..1 + SALT_LEN + NONCE_LEN];
    let ciphertext = &envelope[1 + SALT_LEN + NONCE_LEN..];

    let key = derive_key(pin, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&*key)
        .map_err(|e| LlaveError::Crypto(format!("AES-GCM init: {e}")))?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| LlaveError::Crypto("wrong PIN or corrupted data".into()))?;

    String::from_utf8(plaintext)
        .map_err(|e| LlaveError::Crypto(format!("invalid UTF-8 after decrypt: {e}")))
}

/// Derive a 256-bit key from a PIN and salt using Argon2id.
fn derive_key(pin: &str, salt: &[u8]) -> Result<Zeroizing<[u8; 32]>> {
    use argon2::Argon2;

    let mut key = Zeroizing::new([0u8; 32]);
    // Default Argon2id params: 19 MiB memory, 2 iterations, 1 lane.
    // Good balance between mobile performance and brute-force resistance.
    Argon2::default()
        .hash_password_into(pin.as_bytes(), salt, &mut *key)
        .map_err(|e| LlaveError::Crypto(format!("Argon2id: {e}")))?;
    Ok(key)
}

/// Generate a device password matching the Android app's format.
///
/// The Android app generates 256 random alphanumeric characters, then
/// SHA-512 hashes them, producing a 128-char uppercase hex string.
pub fn generate_device_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();
    let random_string: Zeroizing<String> = Zeroizing::new(
        (0..256)
            .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
            .collect(),
    );
    let hash = Sha512::digest(random_string.as_bytes());
    hash.iter()
        .map(|b| format!("{:02X}", b))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    // -- seal_session / open_session --

    #[test]
    fn seal_open_roundtrip() {
        let json = r#"{"device_id":"abc","nif":"12345678Z"}"#;
        let pin = "1234";
        let sealed = seal_session(json, pin).unwrap();
        let opened = open_session(&sealed, pin).unwrap();
        assert_eq!(opened, json);
    }

    #[test]
    fn seal_open_wrong_pin_fails() {
        let json = "hello";
        let sealed = seal_session(json, "correct").unwrap();
        let err = open_session(&sealed, "wrong").unwrap_err();
        assert!(err.to_string().contains("wrong PIN"));
    }

    #[test]
    fn seal_produces_different_ciphertext() {
        let json = "same-input";
        let pin = "pin";
        let a = seal_session(json, pin).unwrap();
        let b = seal_session(json, pin).unwrap();
        assert_ne!(a, b, "each seal should use a random salt+nonce");
    }

    #[test]
    fn seal_open_empty_plaintext() {
        let sealed = seal_session("", "pin").unwrap();
        let opened = open_session(&sealed, "pin").unwrap();
        assert_eq!(opened, "");
    }

    #[test]
    fn seal_open_empty_pin() {
        let json = "data";
        let sealed = seal_session(json, "").unwrap();
        let opened = open_session(&sealed, "").unwrap();
        assert_eq!(opened, json);
    }

    // -- open_session error paths --

    #[test]
    fn open_session_invalid_base64() {
        let err = open_session("not-valid-base64!!!", "pin").unwrap_err();
        assert!(err.to_string().contains("base64"));
    }

    #[test]
    fn open_session_truncated_data() {
        // Valid base64 but too short to contain version+salt+nonce+tag
        let short = base64::engine::general_purpose::STANDARD.encode([1u8; 10]);
        let err = open_session(&short, "pin").unwrap_err();
        assert!(err.to_string().contains("too short"));
    }

    #[test]
    fn open_session_wrong_version() {
        let mut envelope = vec![99u8]; // wrong version
        envelope.extend_from_slice(&[0u8; SALT_LEN]);
        envelope.extend_from_slice(&[0u8; NONCE_LEN]);
        envelope.extend_from_slice(&[0u8; 16]); // fake GCM tag
        let b64 = base64::engine::general_purpose::STANDARD.encode(&envelope);
        let err = open_session(&b64, "pin").unwrap_err();
        assert!(err.to_string().contains("unsupported sealed version"));
    }

    // -- generate_device_password --

    #[test]
    fn generate_device_password_format() {
        let pw = generate_device_password();
        assert_eq!(pw.len(), 128);
        assert!(pw.chars().all(|c| c.is_ascii_hexdigit() && (c.is_ascii_digit() || c.is_ascii_uppercase())));
    }

    #[test]
    fn generate_device_password_randomness() {
        let a = generate_device_password();
        let b = generate_device_password();
        assert_ne!(a, b);
    }
}
