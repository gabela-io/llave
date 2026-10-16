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

impl LlaveError {
    /// Stable identifier of the error variant, for frontends that show
    /// their own (localised) message instead of the English `Display` text.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Http(_) => "http",
            Self::Api { .. } => "api",
            Self::NoSession => "no_session",
            Self::Config(_) => "config",
            Self::Crypto(_) => "crypto",
            Self::Io(_) => "io",
            Self::Json(_) => "json",
            Self::DniAuthFailed => "dni_auth_failed",
            Self::HtmlResponse { .. } => "html_response",
            // 105/109: the device was revoked; like the app, forget it locally.
            Self::DeviceNotActive { code, .. } if matches!(code.as_str(), "105" | "109") => {
                "device_revoked"
            }
            Self::DeviceNotActive { .. } => "device_not_active",
            Self::NotRegistered => "not_registered",
            Self::NoRegisteredPhone => "no_registered_phone",
            Self::AppVersionOutdated { .. } => "app_version_outdated",
            Self::InvalidQr => "invalid_qr",
            Self::DniBirthDateRequired { .. } => "dni_birth_date_required",
            Self::NoPendingPin => "no_pending_pin",
            Self::Keyring(_) => "keyring",
            Self::InvalidNif(_) => "invalid_nif",
            Self::IpRateLimited { .. } => "ip_rate_limited",
            Self::TooManyRedirects { .. } => "too_many_redirects",
        }
    }

    /// The message the AEAT server sent with a `KO`, when there is one.
    /// The app shows it as is.
    pub fn server_message(&self) -> Option<&str> {
        match self {
            Self::Api { message, .. }
            | Self::DeviceNotActive { message, .. }
            | Self::AppVersionOutdated { message }
            | Self::DniBirthDateRequired { message, .. }
            | Self::IpRateLimited { message } => Some(message.as_str()).filter(|m| !m.trim().is_empty()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_identifies_the_variant() {
        assert_eq!(LlaveError::NoPendingPin.kind(), "no_pending_pin");
        let api = LlaveError::Api { status: "KO".into(), code: "416".into(), message: "x".into() };
        assert_eq!(api.kind(), "api");
    }

    #[test]
    fn revoked_device_has_its_own_kind() {
        let err = |code: &str| LlaveError::DeviceNotActive { code: code.into(), message: String::new() };
        assert_eq!(err("105").kind(), "device_revoked");
        assert_eq!(err("109").kind(), "device_revoked");
        assert_eq!(err("416").kind(), "device_not_active");
    }

    #[test]
    fn server_message_is_the_ko_text() {
        let api = LlaveError::Api {
            status: "KO".into(),
            code: "300".into(),
            message: "Datos incorrectos".into(),
        };
        assert_eq!(api.server_message(), Some("Datos incorrectos"));
    }

    #[test]
    fn blank_server_message_is_none() {
        let api = LlaveError::Api { status: "KO".into(), code: "300".into(), message: " ".into() };
        assert_eq!(api.server_message(), None);
        assert_eq!(LlaveError::InvalidQr.server_message(), None);
    }
}
