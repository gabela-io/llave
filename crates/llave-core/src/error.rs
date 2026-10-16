use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlaveError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error: status={status}, code={code}, message={message}")]
    Api {
        status: String,
        code: String,
        message: String,
    },

    #[error("Session not found or expired. Run `llave activate` first.")]
    NoSession,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("DNI/NIE authentication failed — the server did not redirect, which usually means the credentials (NIF, fecha, or soporte) are incorrect")]
    DniAuthFailed,

    #[error("Server returned HTML instead of JSON at {endpoint} — session may have expired or DNI auth may have failed")]
    HtmlResponse { endpoint: String },

    #[error("This device is no longer active in Cl@ve (code {code}). Run `llave activate` again.")]
    DeviceNotActive { code: String, message: String },

    #[error("This NIF is not registered in Cl@ve. Register first at the AEAT website or an office.")]
    NotRegistered,

    #[error("Your Cl@ve registration has no mobile phone. Update your phone at the AEAT website first.")]
    NoRegisteredPhone,

    #[error("The AEAT server requires a newer app version: {message}")]
    AppVersionOutdated { message: String },

    #[error("Invalid Cl@ve QR code (expected the code shown by the AEAT website)")]
    InvalidQr,

    #[error("DNI/NIE check needs your date of birth: {message}")]
    DniBirthDateRequired {
        challenge: crate::api::DniChallenge,
        message: String,
    },

    #[error("No pending Cl@ve PIN. Start a PIN request on the AEAT website (NIF + date) first, then retry.")]
    NoPendingPin,

    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("Invalid NIF format: {0}")]
    InvalidNif(String),

    #[error("IP rate-limited by AEAT: {message}")]
    IpRateLimited { message: String },

    #[error("Too many redirects ({count}) at {endpoint}")]
    TooManyRedirects { endpoint: String, count: u32 },
}

pub type Result<T> = std::result::Result<T, LlaveError>;
