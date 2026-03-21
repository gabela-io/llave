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

/// Encrypt data for browser handoff using server-provided AES-CBC parameters.
///
/// Mirrors the Android app's `KeyChainManager.encryptTextForBrowser()`:
/// - Key derivation: PBKDF2-HMAC-SHA1, 1000 iterations, 128-bit key
/// - Encryption: AES-128-CBC with PKCS5 padding
/// - Output: Base64 → URL-encoded
///
/// # Security note
///
/// These parameters are weak by modern standards (OWASP recommends 600,000+
/// iterations for PBKDF2-SHA1). However, they **cannot be changed** — the
/// AEAT server expects exactly this format for the browser handoff protocol.
/// The PIN-based session encryption ([`seal_session`]) uses Argon2id +
/// AES-256-GCM instead.
pub fn encrypt_for_browser(
    passphrase: &str,
    iv_hex: &str,
    salt_hex: &str,
    plaintext: &str,
) -> Result<String> {
    use hmac::Hmac;
    use sha1::Sha1;

    // Decode hex salt and IV
    let salt = hex_to_bytes(salt_hex)?;
    let iv = hex_to_bytes(iv_hex)?;

    // PBKDF2 key derivation
    let mut key = Zeroizing::new([0u8; 16]); // 128 bits
    pbkdf2::pbkdf2::<Hmac<Sha1>>(passphrase.as_bytes(), &salt, 1000, &mut *key)
        .map_err(|e| LlaveError::Crypto(format!("PBKDF2 failed: {e}")))?;

    // AES-128-CBC encryption with PKCS7 padding
    use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
    type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

    let encryptor = Aes128CbcEnc::new_from_slices(&*key, &iv)
        .map_err(|e| LlaveError::Crypto(format!("AES init failed: {e}")))?;

    let plaintext_bytes = plaintext.as_bytes();
    // Allocate buffer with padding space
    let mut buf = vec![0u8; plaintext_bytes.len() + 16];
    buf[..plaintext_bytes.len()].copy_from_slice(plaintext_bytes);

    let ciphertext = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext_bytes.len())
        .map_err(|e| LlaveError::Crypto(format!("AES encrypt failed: {e}")))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(ciphertext);
    Ok(urlencoding::encode(&b64).into_owned())
}

/// Build the plaintext payload for browser encryption
pub fn build_browser_payload(
    device_id: &str,
    device_password: &str,
    target_url: &str,
    language: &str,
    nif: Option<&str>,
    contraste: Option<(&str, &str)>, // (dato_contraste, tipo_autenticacion)
    pin_data: Option<(&str, &str)>,  // (codigo, pin)
) -> String {
    let lang_suffix = if language == "en" {
        format!("{language}_GB")
    } else {
        format!("{language}_ES")
    };

    let mut payload = format!(
        "$id_dispositivo:{device_id}$password_dispositivo:{device_password}$url_destino:{target_url}$idioma:{lang_suffix}"
    );

    if let Some(nif) = nif {
        if !nif.is_empty() {
            payload.push_str(&format!("$NIF:{nif}"));
        }
    }

    if let Some((dato, tipo)) = contraste {
        if !dato.is_empty() {
            payload.push_str(&format!("$datoContraste:{dato}$tipoAutenticacion:{tipo}"));
        }
    }

    if let Some((codigo, pin)) = pin_data {
        if !codigo.is_empty() {
            payload.push_str(&format!("$codigo:{codigo}$pin:{pin}"));
        }
    }

    payload
}

pub(crate) fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return Err(LlaveError::Crypto(format!("Odd-length hex string: {}", hex.len())));
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| LlaveError::Crypto(format!("Invalid hex: {e}")))
        })
        .collect()
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

    // -- hex_to_bytes --

    #[test]
    fn hex_to_bytes_valid() {
        assert_eq!(hex_to_bytes("deadbeef").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn hex_to_bytes_empty() {
        assert_eq!(hex_to_bytes("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn hex_to_bytes_uppercase() {
        assert_eq!(hex_to_bytes("DEADBEEF").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn hex_to_bytes_odd_length_rejected() {
        let err = hex_to_bytes("abc").unwrap_err();
        assert!(err.to_string().contains("Odd-length"));
    }

    #[test]
    fn hex_to_bytes_invalid_chars() {
        let err = hex_to_bytes("zzzz").unwrap_err();
        assert!(err.to_string().contains("Invalid hex"));
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

    // -- encrypt_for_browser --

    #[test]
    fn encrypt_for_browser_deterministic() {
        let passphrase = "testpass";
        let iv = "00112233445566778899aabbccddeeff"; // 16 bytes
        let salt = "0011223344556677"; // 8 bytes
        let plaintext = "hello";
        let a = encrypt_for_browser(passphrase, iv, salt, plaintext).unwrap();
        let b = encrypt_for_browser(passphrase, iv, salt, plaintext).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn encrypt_for_browser_invalid_hex_iv() {
        // Valid salt, invalid IV (even-length but non-hex chars)
        let err = encrypt_for_browser("pass", "zzzz", "aabb", "text").unwrap_err();
        assert!(err.to_string().contains("Invalid hex"));
    }

    #[test]
    fn encrypt_for_browser_invalid_hex_salt() {
        // Odd-length salt
        let err = encrypt_for_browser("pass", "aabb", "xyz", "text").unwrap_err();
        assert!(err.to_string().contains("Odd-length"));
    }

    // -- build_browser_payload --

    #[test]
    fn build_browser_payload_basic() {
        let p = build_browser_payload("dev1", "pass1", "https://example.com", "es", None, None, None);
        assert!(p.contains("$id_dispositivo:dev1"));
        assert!(p.contains("$password_dispositivo:pass1"));
        assert!(p.contains("$url_destino:https://example.com"));
        assert!(p.contains("$idioma:es_ES"));
    }

    #[test]
    fn build_browser_payload_english() {
        let p = build_browser_payload("d", "p", "u", "en", None, None, None);
        assert!(p.contains("$idioma:en_GB"));
    }

    #[test]
    fn build_browser_payload_with_nif() {
        let p = build_browser_payload("d", "p", "u", "es", Some("12345678Z"), None, None);
        assert!(p.contains("$NIF:12345678Z"));
    }

    #[test]
    fn build_browser_payload_empty_nif_omitted() {
        let p = build_browser_payload("d", "p", "u", "es", Some(""), None, None);
        assert!(!p.contains("$NIF:"));
    }

    #[test]
    fn build_browser_payload_with_contraste() {
        let p = build_browser_payload("d", "p", "u", "es", None, Some(("dato1", "tipo1")), None);
        assert!(p.contains("$datoContraste:dato1"));
        assert!(p.contains("$tipoAutenticacion:tipo1"));
    }

    #[test]
    fn build_browser_payload_with_pin_data() {
        let p = build_browser_payload("d", "p", "u", "es", None, None, Some(("code1", "pin1")));
        assert!(p.contains("$codigo:code1"));
        assert!(p.contains("$pin:pin1"));
    }
}
