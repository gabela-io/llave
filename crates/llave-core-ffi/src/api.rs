use flutter_rust_bridge::frb;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// ---------------------------------------------------------------------------
// FFI-safe types (no lifetimes, simple owned data)
// ---------------------------------------------------------------------------

/// Session information returned after activation.
#[frb]
pub struct FfiSession {
    pub device_id: String,
    pub nif: String,
    pub created_at: String,
    pub has_firebase_token: bool,
}

/// Result of a PIN request.
#[frb]
pub struct FfiPinResult {
    pub pin: String,
    pub time_to_live_seconds: String,
    pub nif: String,
}

/// Status of the current session.
#[frb]
pub struct FfiStatus {
    pub active: bool,
    pub nif: Option<String>,
    pub device_id: Option<String>,
    pub created_at: Option<String>,
    pub has_firebase_token: bool,
}

/// Generic API result carrying JSON data.
#[frb]
pub struct FfiApiResult {
    pub ok: bool,
    pub data: String, // JSON string
    /// English error text, for logs.
    pub error: Option<String>,
    /// [`LlaveError::kind`](llave_core::LlaveError::kind), for a localised message.
    pub error_code: Option<String>,
    /// The AEAT server's own message for a `KO`, shown as is.
    pub server_message: Option<String>,
}

impl FfiApiResult {
    fn success(data: String) -> Self {
        Self { ok: true, data, error: None, error_code: None, server_message: None }
    }

    fn failure(e: &llave_core::LlaveError) -> Self {
        Self {
            ok: false,
            data: String::new(),
            error: Some(e.to_string()),
            error_code: Some(e.kind().to_owned()),
            server_message: e.server_message().map(str::to_owned),
        }
    }
}

/// NIF check result.
#[frb]
pub struct FfiNifCheckResult {
    pub nif: String,
    pub status: String,
    pub response_json: String,
}

// ---------------------------------------------------------------------------
// Storage initialisation — must be called before any other function.
// ---------------------------------------------------------------------------

/// Initialise the Rust core storage backend.
///
/// The FFI layer always uses [`MemoryStorage`]; Flutter is responsible for
/// persisting via `flutter_secure_storage` and passing the saved JSON here
/// on startup.  (The CLI uses [`KeyringStorage`] directly — see
/// `llave-cli/src/main.rs`.)
#[frb]
pub fn init_core(session_json: Option<String>) -> Result<bool, String> {
    // Initialise logfmt tracing for Rust core (only once; ignore if already set).
    let _ = tracing_subscriber::Registry::default()
        .with(tracing_logfmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "llave_core=info,llave_core_ffi=info,warn".parse().unwrap()),
        )
        .try_init();

    // FFI is always called from Flutter, which manages persistence via
    // flutter_secure_storage.  Use MemoryStorage on all platforms — the
    // KeyringStorage backend is reserved for the standalone CLI.
    tracing::info!(backend = "memory", "init storage");
    llave_core::init_storage(Box::new(llave_core::MemoryStorage::new()));
    if let Some(json) = session_json {
        llave_core::Session::set_session_data(&json).map_err(|e| e.to_string())?;
        tracing::debug!("session hydrated from flutter");
    }
    Ok(true)
}

/// Set the proxy URL for all subsequent API calls.
///
/// Supports HTTP, HTTPS, and SOCKS5 URLs (e.g. `socks5://127.0.0.1:1080`).
/// Pass `None` to disable the proxy.
#[frb]
pub fn set_proxy(url: Option<String>) {
    tracing::info!(proxy = url.as_deref().unwrap_or("none"), "set proxy");
    llave_core::set_proxy(url);
}

/// Export the current session as a JSON string.
///
/// Flutter should call this after operations that create or mutate the session
/// (e.g. `dni_complete_activation`) and persist the returned JSON in
/// `flutter_secure_storage`.  Returns `null` if there is no active session.
#[frb]
pub fn export_session() -> Option<String> {
    llave_core::Session::get_session_data().ok()
}

// ---------------------------------------------------------------------------
// Bridge helpers (internal)
// ---------------------------------------------------------------------------

/// Load session and shared client — used by functions that need session data.
fn session_and_client(
) -> Result<(llave_core::Session, std::sync::Arc<llave_core::LlaveClient>), String> {
    let session = llave_core::Session::load().map_err(|e| e.to_string())?;
    let client = llave_core::shared_client().map_err(|e| e.to_string())?;
    Ok((session, client))
}

/// Convert a `Result<T, E>` into an `FfiApiResult`.
fn to_ffi_result<T: serde::Serialize>(
    result: llave_core::error::Result<T>,
) -> Result<FfiApiResult, String> {
    Ok(match result {
        Ok(data) => FfiApiResult::success(serde_json::to_string(&data).unwrap_or_default()),
        Err(e) => FfiApiResult::failure(&e),
    })
}

/// Persist NIF and device ID in config, logging on failure.
fn save_session_config(nif: &str, device_id: &str) {
    if let Ok(mut cfg) = llave_core::Config::load() {
        cfg.nif = Some(nif.to_owned());
        cfg.device_id = Some(device_id.to_owned());
        if let Err(e) = cfg.save() {
            tracing::warn!(err = %e, "failed to save config");
        }
    }
}

// ---------------------------------------------------------------------------
// Bridge functions
// ---------------------------------------------------------------------------

/// Request a Llave PIN.
#[frb]
pub async fn request_pin() -> Result<FfiPinResult, String> {
    let (session, client) = session_and_client()?;
    let (pin, ttl) = llave_core::auth::request_pin(&client, &session)
        .await
        .map_err(|e| e.to_string())?;

    Ok(FfiPinResult {
        pin,
        time_to_live_seconds: ttl,
        nif: session.nif,
    })
}

/// Get current session status.
#[frb]
pub fn get_status() -> FfiStatus {
    match llave_core::Session::load() {
        Ok(session) => FfiStatus {
            active: true,
            nif: Some(session.nif),
            device_id: Some(session.device_id),
            created_at: Some(session.created_at),
            has_firebase_token: session.firebase_token.is_some(),
        },
        Err(_) => FfiStatus {
            active: false,
            nif: None,
            device_id: None,
            created_at: None,
            has_firebase_token: false,
        },
    }
}

/// Check if a NIF is registered in Llave.
#[frb]
pub async fn check_nif(nif: Option<String>) -> Result<FfiNifCheckResult, String> {
    let nif = if let Some(n) = nif {
        llave_core::config::validate_nif(&n).map_err(|e| e.to_string())?
    } else {
        let cfg = llave_core::Config::load().map_err(|e| e.to_string())?;
        cfg.nif.ok_or("No NIF configured")?
    };

    let client = llave_core::shared_client().map_err(|e| e.to_string())?;
    // Like the app: the stored device id (empty if not activated).
    let device_id = llave_core::Session::load()
        .map(|s| s.device_id)
        .unwrap_or_default();

    let resp = client
        .clave_is_nif_activated(&device_id, &nif)
        .await
        .map_err(|e| e.to_string())?;

    Ok(FfiNifCheckResult {
        nif,
        status: resp.status,
        response_json: serde_json::to_string(&resp.respuesta).unwrap_or_default(),
    })
}

/// Get account data.
///
/// Mirrors the Android app flow: ClaveIsNifActivatedSv → ClaveCheckMyDataSv.
/// The response contains `email`, `numTelefono`, and `nivelRegistro`.
#[frb]
pub async fn get_my_data() -> Result<FfiApiResult, String> {
    let (session, client) = session_and_client()?;
    // The NIF is not returned by the server; add it from the session so the
    // UI can display it.
    to_ffi_result(
        llave_core::auth::my_data(&client, &session)
            .await
            .map(|d| {
                serde_json::json!({
                    "nif": session.nif,
                    "email": d.email,
                    "numTelefono": d.num_telefono,
                    "nivelRegistro": d.nivel_registro,
                })
            }),
    )
}

/// Get operations history.
#[frb]
pub async fn get_history() -> Result<FfiApiResult, String> {
    let (session, client) = session_and_client()?;
    // The history screen shows a month of confirmed and rejected requests
    // (the app's own default is 7 days, confirmed only).
    to_ffi_result(
        llave_core::auth::history(&client, &session, 30, llave_core::auth::HistoryResult::All)
            .await,
    )
}

/// Poll for pending authentication requests (single poll).
#[frb]
pub async fn get_pending_requests() -> Result<FfiApiResult, String> {
    let (session, client) = session_and_client()?;
    // `data` is the pending `peticion` object, or `null` when there is
    // nothing yet (codes meaning "keep polling" are folded in by the core).
    to_ffi_result(llave_core::auth::next_pending_request(&client, &session).await)
}

/// Confirm a pending Llave Móvil authentication request.
#[frb]
pub async fn confirm_request(token: String, idp_code: String) -> Result<FfiApiResult, String> {
    let (session, client) = session_and_client()?;
    to_ffi_result(
        llave_core::auth::confirm_authentication(&client, &session, &token, &idp_code).await,
    )
}

/// Reject a pending Llave Móvil authentication request.
#[frb]
pub async fn reject_request(token: String, idp_code: String) -> Result<FfiApiResult, String> {
    let (session, client) = session_and_client()?;
    to_ffi_result(
        llave_core::auth::reject_authentication(&client, &session, &token, &idp_code).await,
    )
}

/// QR code authentication.
#[frb]
pub async fn qr_authenticate(value: String) -> Result<FfiApiResult, String> {
    let (session, client) = session_and_client()?;
    to_ffi_result(llave_core::auth::qr_authenticate(&client, &session, &value).await)
}

/// Deactivate this device.
#[frb]
pub async fn deactivate() -> Result<FfiApiResult, String> {
    let (session, client) = session_and_client()?;
    to_ffi_result(
        llave_core::auth::deactivate(&client, &session)
            .await
            .map(|()| serde_json::json!({"status": "OK", "message": "Device deactivated"})),
    )
}

/// Log out and clear saved credentials.
#[frb]
pub fn logout() -> Result<bool, String> {
    llave_core::Session::delete().map_err(|e| e.to_string())?;
    Ok(true)
}

/// Validate a NIF format without making any API calls.
#[frb]
pub fn validate_nif(nif: String) -> Result<String, String> {
    llave_core::config::validate_nif(&nif).map_err(|e| e.to_string())
}

/// Which document field the DNI/NIE authentication needs for `nif`:
/// `"validity_date"`, `"birth_date"` or `"support_number"`.
#[frb]
pub fn dni_input(nif: String) -> Result<String, String> {
    use llave_core::config::DniInput;
    let nif = llave_core::config::validate_nif(&nif).map_err(|e| e.to_string())?;
    Ok(match llave_core::config::required_dni_input(&nif) {
        DniInput::ValidityDate => "validity_date",
        DniInput::BirthDate => "birth_date",
        DniInput::SupportNumber => "support_number",
    }
    .into())
}

/// Wire name of a [`DniChallenge`](llave_core::api::DniChallenge), as
/// passed between Flutter and [`dni_authenticate`].
fn challenge_name(challenge: llave_core::api::DniChallenge) -> &'static str {
    match challenge {
        llave_core::api::DniChallenge::Azul => "azul",
        llave_core::api::DniChallenge::FechaNie => "fecha_nie",
    }
}

fn challenge_from_name(name: &str) -> Result<Option<llave_core::api::DniChallenge>, String> {
    match name {
        "" => Ok(None),
        "azul" => Ok(Some(llave_core::api::DniChallenge::Azul)),
        "fecha_nie" => Ok(Some(llave_core::api::DniChallenge::FechaNie)),
        other => Err(format!("unknown DNI challenge: {other}")),
    }
}

// ---------------------------------------------------------------------------
// PIN-based session encryption
// ---------------------------------------------------------------------------

/// Encrypt the current in-memory session with a PIN.
///
/// Returns a base64-encoded sealed blob. Flutter should store this in
/// `flutter_secure_storage` instead of raw JSON.
#[frb]
pub fn encrypt_session(pin: String) -> Result<String, String> {
    let json = llave_core::Session::get_session_data().map_err(|e| e.to_string())?;
    llave_core::crypto::seal_session(&json, &pin).map_err(|e| e.to_string())
}

/// Decrypt a sealed session blob and load it into the Rust core.
///
/// On success the session is available for all subsequent API calls.
/// Returns an error if the PIN is wrong (AES-GCM auth fails).
#[frb]
pub fn decrypt_and_load_session(sealed: String, pin: String) -> Result<bool, String> {
    let json =
        llave_core::crypto::open_session(&sealed, &pin).map_err(|e| e.to_string())?;
    llave_core::Session::set_session_data(&json).map_err(|e| e.to_string())?;
    tracing::debug!("session decrypted and loaded");
    Ok(true)
}

/// Re-encrypt the current session with a new PIN.
///
/// Requires the old PIN to decrypt first (validates the caller knows it),
/// then re-encrypts with the new PIN and returns the new sealed blob.
#[frb]
pub fn change_session_pin(
    sealed: String,
    old_pin: String,
    new_pin: String,
) -> Result<String, String> {
    let json =
        llave_core::crypto::open_session(&sealed, &old_pin).map_err(|e| e.to_string())?;
    llave_core::crypto::seal_session(&json, &new_pin).map_err(|e| e.to_string())
}

/// Phase 1: DNI/NIE auth → registration check → request SMS code.
///
/// Returns the registration state, masked phone number, SMS tokens,
/// and serialised cookies for phase 2.
///
/// After this call, the user must enter the SMS PIN they receive,
/// then call [`dni_complete_activation`] with the PIN and cookies.
///
/// `challenge` is empty on the first call. When the server asks for the
/// date of birth the result is `ok: false` with `data = {"challenge": …}`;
/// call again with the date of birth as `fecha` and that challenge.
#[frb]
pub async fn dni_authenticate(
    nif: String,
    fecha: String,
    soporte: String,
    challenge: String,
) -> Result<FfiApiResult, String> {
    let nif = llave_core::config::validate_nif(&nif).map_err(|e| e.to_string())?;
    let client = llave_core::shared_client().map_err(|e| e.to_string())?;

    let challenge = challenge_from_name(&challenge)?;

    match llave_core::auth::dni_request_sms(&client, &nif, &fecha, &soporte, challenge).await {
        Ok(phase1) => {
            tracing::info!(
                movil = %phase1.sms.movil,
                "DNI/NIE auth succeeded — SMS sent"
            );
            Ok(FfiApiResult::success(
                serde_json::to_string(&serde_json::json!({
                    "registrado": phase1.state.registrado,
                    "nivel_registro": phase1.state.nivel_registro,
                    "telefono": phase1.state.telefono,
                    "movil": phase1.sms.movil,
                    "hora_peticion": phase1.sms.hora_peticion,
                    "timestamp_alta_sms": phase1.sms.timestamp_alta_sms,
                    "token_clave_movil_sms": phase1.sms.token_clave_movil_sms,
                    "cookies_json": phase1.cookies_json,
                }))
                .unwrap_or_default(),
            ))
        }
        // Like the app, the caller asks for the date of birth and calls again
        // with it as `fecha`, an empty `soporte` and this challenge.
        Err(e @ llave_core::LlaveError::DniBirthDateRequired { challenge, .. }) => Ok(FfiApiResult {
            data: serde_json::json!({ "challenge": challenge_name(challenge) }).to_string(),
            ..FfiApiResult::failure(&e)
        }),
        Err(e) => {
            tracing::warn!(err = %e, "DNI/NIE auth + SMS request failed");
            Ok(FfiApiResult::failure(&e))
        }
    }
}

/// Phase 2: Validate SMS code + activate device.
///
/// Takes the SMS tokens and cookies from [`dni_authenticate`] (phase 1),
/// validates the user-entered SMS PIN, then activates the device on www6.
#[frb]
pub async fn dni_complete_activation(
    nif: String,
    cookies_json: String,
    timestamp_alta_sms: String,
    token_clave_movil_sms: String,
    sms_pin: String,
) -> Result<FfiApiResult, String> {
    let nif = llave_core::config::validate_nif(&nif).map_err(|e| e.to_string())?;
    let client = llave_core::shared_client().map_err(|e| e.to_string())?;
    let device_password = llave_core::crypto::generate_device_password();

    match llave_core::auth::dni_validate_and_activate(
        &client,
        &cookies_json,
        &nif,
        &timestamp_alta_sms,
        &token_clave_movil_sms,
        &sms_pin,
        &device_password,
    )
    .await
    {
        Ok(session) => {
            save_session_config(&nif, &session.device_id);
            tracing::info!(nif = %session.nif, "DNI/NIE + SMS activation succeeded");
            Ok(FfiApiResult::success(
                serde_json::to_string(&serde_json::json!({
                    "device_id": session.device_id,
                    "nif": session.nif,
                }))
                .unwrap_or_default(),
            ))
        }
        Err(e) => {
            tracing::warn!(err = %e, "DNI/NIE SMS validation + activation failed");
            Ok(FfiApiResult::failure(&e))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dni_input_follows_document_type() {
        assert_eq!(dni_input("12345678Z".into()).unwrap(), "validity_date");
        assert_eq!(dni_input("X1234567L".into()).unwrap(), "support_number");
        assert_eq!(dni_input("M1234567L".into()).unwrap(), "birth_date");
    }

    #[test]
    fn dni_input_rejects_invalid_nif() {
        assert!(dni_input("123".into()).is_err());
    }

    #[test]
    fn challenge_names_round_trip() {
        use llave_core::api::DniChallenge;
        for c in [DniChallenge::Azul, DniChallenge::FechaNie] {
            assert_eq!(challenge_from_name(challenge_name(c)).unwrap(), Some(c));
        }
        assert_eq!(challenge_from_name("").unwrap(), None);
        assert!(challenge_from_name("bogus").is_err());
    }
}
