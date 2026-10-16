use crate::error::{LlaveError, Result};
use std::sync::OnceLock;
use zeroize::Zeroizing;

/// Platform-agnostic secure credential storage.
///
/// Each platform provides its own implementation:
/// - **Desktop (CLI)**: [`KeyringStorage`] — delegates to the OS secret store
///   (GNOME Keyring, macOS Keychain, Windows Credential Manager) via the
///   `keyring` crate.
/// - **Mobile (FFI)**: [`MemoryStorage`] — holds credentials in-memory while
///   the app is running. Flutter hydrates from `flutter_secure_storage` on
///   startup and persists back after mutations.
pub trait SecureStorage: Send + Sync {
    fn save(&self, data: &str) -> Result<()>;
    fn load(&self) -> Result<String>;
    fn delete(&self) -> Result<()>;

    /// Get ready to store a new session, before anything irreversible
    /// happens (for example, ask for the PIN that will seal it).
    fn prepare_new(&self) -> Result<()> {
        Ok(())
    }
}

/// Get the storage backend ready for a new session. Call it before
/// activating, so a failure here cannot lose credentials the server has
/// already issued.
pub fn prepare_new_session() -> Result<()> {
    storage()?.prepare_new()
}

static STORAGE: OnceLock<Box<dyn SecureStorage>> = OnceLock::new();

/// Register the storage backend. Must be called once at startup.
pub fn init_storage(backend: Box<dyn SecureStorage>) {
    let _ = STORAGE.set(backend);
}

/// Get the registered storage backend.
pub(crate) fn storage() -> Result<&'static dyn SecureStorage> {
    STORAGE
        .get()
        .map(|b| b.as_ref())
        .ok_or_else(|| LlaveError::Config("Storage backend not initialized".into()))
}

// ---------------------------------------------------------------------------
// Desktop: OS keyring (not available on Android)
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "android"))]
/// Stores credentials in the OS secret store via the `keyring` crate.
pub struct KeyringStorage {
    service: String,
    /// A fixed account identifier (the NIF is unknown at init time, so we use a
    /// well-known key and store the full session JSON as the "password").
    account: String,
}

#[cfg(not(target_os = "android"))]
impl KeyringStorage {
    pub fn new() -> Self {
        Self {
            service: "llave-cli".into(),
            account: "session".into(),
        }
    }
}

#[cfg(not(target_os = "android"))]
impl Default for KeyringStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "android"))]
impl SecureStorage for KeyringStorage {
    fn save(&self, data: &str) -> Result<()> {
        tracing::debug!(backend = "keyring", "session save");
        let entry = keyring::Entry::new(&self.service, &self.account)
            .map_err(|e| LlaveError::Keyring(e.to_string()))?;
        entry
            .set_password(data)
            .map_err(|e| {
                tracing::error!(backend = "keyring", err = %e, "save failed");
                LlaveError::Keyring(e.to_string())
            })?;
        Ok(())
    }

    fn load(&self) -> Result<String> {
        tracing::debug!(backend = "keyring", "session load");
        let entry = keyring::Entry::new(&self.service, &self.account)
            .map_err(|e| LlaveError::Keyring(e.to_string()))?;
        entry
            .get_password()
            .map_err(|e| {
                tracing::debug!(backend = "keyring", err = %e, "load failed");
                LlaveError::Keyring(e.to_string())
            })
    }

    fn delete(&self) -> Result<()> {
        tracing::debug!(backend = "keyring", "session delete");
        let entry = keyring::Entry::new(&self.service, &self.account)
            .map_err(|e| LlaveError::Keyring(e.to_string()))?;
        let _ = entry.delete_credential();
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Desktop CLI: PIN-sealed file (durable) + optional in-memory cache
// ---------------------------------------------------------------------------

/// Why the storage needs the user's PIN.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinRequest {
    /// No sealed file yet: choose a new PIN (the caller should confirm it).
    Create,
    /// Unlock the existing sealed file; `attempt` starts at 1.
    Unlock { attempt: u32 },
}

/// Asks the user for a PIN.
pub type PinPrompt = Box<dyn Fn(PinRequest) -> Result<Zeroizing<String>> + Send + Sync>;

/// Wrong-PIN attempts before [`SealedFileStorage::load`] gives up.
pub const UNLOCK_ATTEMPTS: u32 = 3;

/// Minimum PIN length (same rule as the Flutter app).
pub const MIN_PIN_LEN: usize = 4;

/// Durable session store for the CLI.
///
/// The session is sealed with the user's PIN (Argon2id + AES-256-GCM, see
/// [`crate::crypto::seal_session`]) and written to a 0600 file, so it
/// survives reboots — unlike the kernel keyring, which is memory-only.
/// An optional `cache` (the kernel keyring) holds the unlocked session for
/// the current login so the PIN is only asked when the cache is empty.
///
/// Credentials change only on activation, which starts from no session, so
/// the first save always seals with a new PIN. Later saves in a process that
/// never asked for the PIN (e.g. polling cursor updates) refresh only the
/// cache; the sealed file keeps the credentials.
pub struct SealedFileStorage {
    path: std::path::PathBuf,
    cache: Option<Box<dyn SecureStorage>>,
    prompt: PinPrompt,
    pin: parking_lot::Mutex<Option<Zeroizing<String>>>,
}

impl SealedFileStorage {
    pub fn new(
        path: impl Into<std::path::PathBuf>,
        cache: Option<Box<dyn SecureStorage>>,
        prompt: PinPrompt,
    ) -> Self {
        Self {
            path: path.into(),
            cache,
            prompt,
            pin: parking_lot::Mutex::new(None),
        }
    }

    fn write_sealed(&self, data: &str, pin: &str) -> Result<()> {
        let sealed = crate::crypto::seal_session(data, pin)?;
        write_owner_only(&self.path, sealed.as_bytes())
    }
}

impl SecureStorage for SealedFileStorage {
    fn save(&self, data: &str) -> Result<()> {
        if let Some(cache) = &self.cache {
            if let Err(e) = cache.save(data) {
                tracing::warn!(err = %e, "session cache save failed");
            }
        }

        if !self.path.exists() {
            self.prepare_new()?;
        }
        match self.pin.lock().as_deref() {
            Some(p) => self.write_sealed(data, p),
            None => {
                tracing::debug!("PIN not unlocked in this process; updated cache only");
                Ok(())
            }
        }
    }

    /// Ask for the PIN that will seal the next session, unless one is
    /// already known in this process.
    fn prepare_new(&self) -> Result<()> {
        let mut pin = self.pin.lock();
        if pin.is_some() {
            return Ok(());
        }
        let new_pin = (self.prompt)(PinRequest::Create)?;
        if new_pin.chars().count() < MIN_PIN_LEN {
            return Err(LlaveError::Config(format!(
                "PIN must have at least {MIN_PIN_LEN} characters"
            )));
        }
        *pin = Some(new_pin);
        Ok(())
    }

    fn load(&self) -> Result<String> {
        if let Some(cache) = &self.cache {
            if let Ok(data) = cache.load() {
                return Ok(data);
            }
        }
        if !self.path.exists() {
            return Err(LlaveError::NoSession);
        }
        let sealed = std::fs::read_to_string(&self.path)?;

        let mut last_err = LlaveError::Crypto("wrong PIN".into());
        for attempt in 1..=UNLOCK_ATTEMPTS {
            let pin = (self.prompt)(PinRequest::Unlock { attempt })?;
            match crate::crypto::open_session(&sealed, &pin) {
                Ok(data) => {
                    *self.pin.lock() = Some(pin);
                    if let Some(cache) = &self.cache {
                        if let Err(e) = cache.save(&data) {
                            tracing::warn!(err = %e, "session cache save failed");
                        }
                    }
                    return Ok(data);
                }
                Err(e @ LlaveError::Crypto(_)) => last_err = e,
                Err(e) => return Err(e),
            }
        }
        Err(last_err)
    }

    fn delete(&self) -> Result<()> {
        *self.pin.lock() = None;
        if let Some(cache) = &self.cache {
            let _ = cache.delete();
        }
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

/// Atomically write `bytes` to `path` readable only by the owner (0600; the
/// parent directory is created 0700).
fn write_owner_only(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;

    let dir = path
        .parent()
        .ok_or_else(|| LlaveError::Config("session path has no parent".into()))?;
    std::fs::create_dir_all(dir)?;
    let tmp = path.with_extension("tmp");

    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }

    std::fs::rename(&tmp, path)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Mobile / FFI: in-memory store
// ---------------------------------------------------------------------------

/// In-memory credential store for mobile platforms.
///
/// Flutter is responsible for persisting to / loading from the platform's
/// native secure storage (`flutter_secure_storage`). On app startup, Flutter
/// calls [`crate::session::Session::set_session_data`] to hydrate this store;
/// after mutations (activate, logout) the FFI layer returns session data for
/// Flutter to persist.
pub struct MemoryStorage {
    data: parking_lot::RwLock<Option<Zeroizing<String>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            data: parking_lot::RwLock::new(None),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SecureStorage for MemoryStorage {
    fn save(&self, data: &str) -> Result<()> {
        *self.data.write() = Some(Zeroizing::new(data.to_owned()));
        Ok(())
    }

    fn load(&self) -> Result<String> {
        let guard = self.data.read();
        guard.as_deref().cloned().ok_or(LlaveError::NoSession)
    }

    fn delete(&self) -> Result<()> {
        *self.data.write() = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_before_save_returns_no_session() {
        let store = MemoryStorage::new();
        let err = store.load().unwrap_err();
        assert!(matches!(err, LlaveError::NoSession));
    }

    #[test]
    fn save_then_load() {
        let store = MemoryStorage::new();
        store.save("session-data").unwrap();
        assert_eq!(store.load().unwrap(), "session-data");
    }

    #[test]
    fn save_overwrites() {
        let store = MemoryStorage::new();
        store.save("first").unwrap();
        store.save("second").unwrap();
        assert_eq!(store.load().unwrap(), "second");
    }

    #[test]
    fn delete_then_load_fails() {
        let store = MemoryStorage::new();
        store.save("data").unwrap();
        store.delete().unwrap();
        let err = store.load().unwrap_err();
        assert!(matches!(err, LlaveError::NoSession));
    }

    #[test]
    fn delete_when_empty_succeeds() {
        let store = MemoryStorage::new();
        store.delete().unwrap();
    }

    // -- SealedFileStorage --

    use std::sync::{Arc, Mutex as StdMutex};

    /// Storage under a temp dir whose prompt always answers `pin` and records
    /// every request.
    fn sealed(dir: &tempfile::TempDir, pin: &str) -> (SealedFileStorage, Arc<StdMutex<Vec<PinRequest>>>) {
        let calls = Arc::new(StdMutex::new(Vec::new()));
        let seen = calls.clone();
        let pin = pin.to_string();
        let store = SealedFileStorage::new(
            dir.path().join("session.sealed"),
            None,
            Box::new(move |req| {
                seen.lock().unwrap().push(req);
                Ok(Zeroizing::new(pin.clone()))
            }),
        );
        (store, calls)
    }

    #[test]
    fn sealed_load_without_file_is_no_session() {
        let dir = tempfile::tempdir().unwrap();
        let (store, _) = sealed(&dir, "1234");
        assert!(matches!(store.load().unwrap_err(), LlaveError::NoSession));
    }

    #[test]
    fn sealed_first_save_asks_to_create_pin() {
        let dir = tempfile::tempdir().unwrap();
        let (store, calls) = sealed(&dir, "1234");
        store.save("session").unwrap();
        assert_eq!(*calls.lock().unwrap(), vec![PinRequest::Create]);
    }

    #[test]
    fn sealed_prepared_pin_is_used_by_the_first_save() {
        let dir = tempfile::tempdir().unwrap();
        let (store, calls) = sealed(&dir, "1234");
        store.prepare_new().unwrap();
        store.save("session").unwrap();
        assert_eq!(*calls.lock().unwrap(), vec![PinRequest::Create]);
        assert_eq!(sealed(&dir, "1234").0.load().unwrap(), "session");
    }

    #[test]
    fn sealed_prepare_rejects_a_short_pin() {
        let dir = tempfile::tempdir().unwrap();
        let (store, _) = sealed(&dir, "12");
        assert!(store.prepare_new().is_err());
    }

    #[test]
    fn sealed_session_survives_a_new_process() {
        let dir = tempfile::tempdir().unwrap();
        sealed(&dir, "1234").0.save("session").unwrap();
        let (reopened, _) = sealed(&dir, "1234");
        assert_eq!(reopened.load().unwrap(), "session");
    }

    #[test]
    fn sealed_file_is_not_plaintext() {
        let dir = tempfile::tempdir().unwrap();
        sealed(&dir, "1234").0.save("secret-session").unwrap();
        let raw = std::fs::read_to_string(dir.path().join("session.sealed")).unwrap();
        assert!(!raw.contains("secret-session"));
    }

    #[cfg(unix)]
    #[test]
    fn sealed_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        sealed(&dir, "1234").0.save("session").unwrap();
        let mode = std::fs::metadata(dir.path().join("session.sealed")).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn sealed_wrong_pin_fails_after_all_attempts() {
        let dir = tempfile::tempdir().unwrap();
        sealed(&dir, "1234").0.save("session").unwrap();
        let (wrong, calls) = sealed(&dir, "9999");
        assert!(matches!(wrong.load().unwrap_err(), LlaveError::Crypto(_)));
        assert_eq!(calls.lock().unwrap().len(), UNLOCK_ATTEMPTS as usize);
    }

    #[test]
    fn sealed_short_pin_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let (store, _) = sealed(&dir, "12");
        assert!(store.save("session").is_err());
    }

    #[test]
    fn sealed_save_after_unlock_reuses_pin() {
        let dir = tempfile::tempdir().unwrap();
        sealed(&dir, "1234").0.save("v1").unwrap();
        let (store, calls) = sealed(&dir, "1234");
        store.load().unwrap();
        store.save("v2").unwrap();
        assert_eq!(calls.lock().unwrap().len(), 1);
    }

    #[test]
    fn sealed_save_without_pin_keeps_file_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        sealed(&dir, "1234").0.save("v1").unwrap();
        let before = std::fs::read(dir.path().join("session.sealed")).unwrap();
        let (store, calls) = sealed(&dir, "1234");
        store.save("v2").unwrap();
        assert_eq!(std::fs::read(dir.path().join("session.sealed")).unwrap(), before);
        assert!(calls.lock().unwrap().is_empty());
    }

    #[test]
    fn sealed_delete_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let (store, _) = sealed(&dir, "1234");
        store.save("session").unwrap();
        store.delete().unwrap();
        assert!(!dir.path().join("session.sealed").exists());
    }

    #[test]
    fn sealed_cache_hit_needs_no_pin() {
        let dir = tempfile::tempdir().unwrap();
        let cache = MemoryStorage::new();
        cache.save("cached").unwrap();
        let calls = Arc::new(StdMutex::new(Vec::new()));
        let seen = calls.clone();
        let store = SealedFileStorage::new(
            dir.path().join("session.sealed"),
            Some(Box::new(cache)),
            Box::new(move |req| {
                seen.lock().unwrap().push(req);
                Ok(Zeroizing::new("1234".into()))
            }),
        );
        assert_eq!(store.load().unwrap(), "cached");
        assert!(calls.lock().unwrap().is_empty());
    }

    #[test]
    fn full_lifecycle() {
        let store = MemoryStorage::new();
        assert!(store.load().is_err());
        store.save("v1").unwrap();
        assert_eq!(store.load().unwrap(), "v1");
        store.save("v2").unwrap();
        assert_eq!(store.load().unwrap(), "v2");
        store.delete().unwrap();
        assert!(store.load().is_err());
        store.save("v3").unwrap();
        assert_eq!(store.load().unwrap(), "v3");
    }
}
