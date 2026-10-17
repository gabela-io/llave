pub mod api;
pub mod auth;
pub mod config;
pub mod crypto;
pub mod error;
pub mod session;
pub mod storage;

pub use api::{reset_shared_client, shared_client, LlaveClient};
pub use config::{get_proxy, set_proxy, Config};
pub use error::LlaveError;
pub use session::Session;
#[cfg(not(target_os = "android"))]
pub use storage::KeyringStorage;
pub use storage::{init_storage, MemoryStorage, PinPrompt, PinRequest, SealedFileStorage};

/// llave's version and the official Cl@ve app version it matches, e.g.
/// `0.1.0 (Cl@ve 6.2.10, build 294)`. Shown by the CLI and the app.
pub fn version() -> String {
    format!(
        "{} (Cl@ve {}, build {})",
        env!("CARGO_PKG_VERSION"),
        api::CLAVE_APP_VERSION,
        api::CLAVE_APP_BUILD
    )
}
