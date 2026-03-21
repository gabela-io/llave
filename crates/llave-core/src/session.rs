use crate::error::Result;
use crate::storage::storage;
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::Zeroizing;

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub device_id: String,
    pub nif: String,
    pub device_password: Zeroizing<String>,
    pub firebase_token: Option<String>,
    pub created_at: String,
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("device_id", &self.device_id)
            .field("nif", &self.nif)
            .field("device_password", &"[REDACTED]")
            .field("firebase_token", &self.firebase_token)
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl Session {
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string(self)?;
        storage()?.save(&json)
    }

    pub fn load() -> Result<Self> {
        let json = storage()?.load()?;
        let session: Session = serde_json::from_str(&json)?;
        Ok(session)
    }

    pub fn delete() -> Result<()> {
        crate::auth::reset_poll_state();
        crate::api::reset_shared_client();
        storage()?.delete()
    }

    /// Hydrate session from a JSON string (used by FFI layer).
    ///
    /// Flutter loads the JSON from its platform secure storage and passes it
    /// here so the Rust core can use the session for subsequent API calls.
    pub fn set_session_data(json: &str) -> Result<()> {
        // Validate that the JSON is a valid session before storing
        let _: Session = serde_json::from_str(json)?;
        storage()?.save(json)
    }

    /// Export session as JSON (used by FFI layer).
    ///
    /// Flutter calls this after operations that mutate the session (e.g.
    /// activation) to persist the result in platform secure storage.
    pub fn get_session_data() -> Result<String> {
        storage()?.load()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zeroize::Zeroizing;

    fn sample_session() -> Session {
        Session {
            device_id: "dev-123".into(),
            nif: "12345678Z".into(),
            device_password: Zeroizing::new("secret-pass".into()),
            firebase_token: Some("fb-token".into()),
            created_at: "2024-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn serde_roundtrip() {
        let session = sample_session();
        let json = serde_json::to_string(&session).unwrap();
        let restored: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.device_id, "dev-123");
        assert_eq!(restored.nif, "12345678Z");
        assert_eq!(&*restored.device_password, "secret-pass");
        assert_eq!(restored.firebase_token.as_deref(), Some("fb-token"));
        assert_eq!(restored.created_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn zeroizing_serializes_transparently() {
        let session = sample_session();
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("secret-pass"));
    }

    #[test]
    fn none_firebase_token_roundtrip() {
        let mut session = sample_session();
        session.firebase_token = None;
        let json = serde_json::to_string(&session).unwrap();
        let restored: Session = serde_json::from_str(&json).unwrap();
        assert!(restored.firebase_token.is_none());
    }

    #[test]
    fn debug_redacts_device_password() {
        let session = sample_session();
        let debug = format!("{:?}", session);
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("secret-pass"));
    }

    #[test]
    fn debug_shows_other_fields() {
        let session = sample_session();
        let debug = format!("{:?}", session);
        assert!(debug.contains("dev-123"));
        assert!(debug.contains("12345678Z"));
        assert!(debug.contains("fb-token"));
        assert!(debug.contains("2024-01-01T00:00:00Z"));
    }
}
