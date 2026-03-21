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
