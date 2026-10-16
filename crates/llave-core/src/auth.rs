use crate::api::LlaveClient;
use crate::error::Result;
use crate::session::Session;
use parking_lot::Mutex;
use zeroize::Zeroizing;

/// Epoch timestamp used for the first poll — "give me everything since the
/// beginning of time".  Format matches the AEAT APK: `yyyyMMdd'T'HHmmssSSS'Z'`.
const EPOCH_TIMESTAMP: &str = "19700101T000000000Z";

/// Last-seen poll timestamp for this process.  It is also persisted in
/// [`Session::poll_cursor`] so the cursor survives between CLI runs (the app
/// keeps it in its DataStore).
static LAST_POLL_TS: Mutex<Option<String>> = Mutex::new(None);

/// Reset polling state. Call on logout/session change so stale timestamps
/// from a previous session are not reused.
pub fn reset_poll_state() {
    *LAST_POLL_TS.lock() = None;
}

/// Cursor precedence: this process's last value, then the persisted one,
/// then the epoch.
fn pick_cursor<'a>(in_memory: Option<&'a str>, stored: Option<&'a str>) -> &'a str {
    in_memory.or(stored).unwrap_or(EPOCH_TIMESTAMP)
}

/// Cursor to send as `timestamp` in `ClaveRequestAllOperationsSv`.
fn current_cursor(session: &Session) -> String {
    let in_memory = LAST_POLL_TS.lock().clone();
    pick_cursor(in_memory.as_deref(), session.poll_cursor.as_deref()).to_string()
}

/// `respuesta.peticion.timestamp` — the only timestamp the app uses as cursor.
fn peticion_timestamp(respuesta: &serde_json::Value) -> Option<&str> {
    respuesta.get("peticion")?.get("timestamp")?.as_str()
}

/// Timestamps share the fixed `yyyyMMdd'T'HHmmssSSS'Z'` format, so string
/// order is chronological.  The app only shows requests not older than the
/// stored cursor.
fn is_stale(peticion_ts: &str, cursor: &str) -> bool {
    peticion_ts < cursor
}

/// Remove `respuesta.peticion` when it is older than `cursor`.
fn drop_stale_peticion(respuesta: &mut serde_json::Value, cursor: &str) {
    if peticion_timestamp(respuesta).is_some_and(|ts| is_stale(ts, cursor)) {
        if let Some(obj) = respuesta.as_object_mut() {
            obj.insert("peticion".into(), serde_json::Value::Null);
        }
    }
}

/// Advance the cursor from a response and persist it in the stored session.
/// Persistence failures are logged, not fatal.
fn record_cursor(respuesta: &serde_json::Value) {
    let Some(ts) = peticion_timestamp(respuesta) else {
        return;
    };
    *LAST_POLL_TS.lock() = Some(ts.to_string());
    let persisted = Session::load().and_then(|mut stored| {
        stored.poll_cursor = Some(ts.to_string());
        stored.save()
    });
    if let Err(e) = persisted {
        tracing::warn!(err = %e, "could not persist poll cursor");
    }
}

/// Request a Llave PIN using saved session credentials.
///
/// Mirrors the original app's full flow:
/// 1. `ClaveStartingSv` — establish device session (app startup call).
/// 2. `ClaveRequestAllOperationsSv` — validate device registration and check
///    for a pending PIN (`respuesta.pin`).  Error codes 105/109/416 indicate
///    the device is not properly registered.
/// 3. `ClaveRequestPinSv` — reveal the pending PIN.  The app never calls this
///    without a pending PIN; the PIN request itself is started on the web.
///
/// Returns `(pin, time_to_live_seconds)`.  Fails with
/// [`LlaveError::NoPendingPin`](crate::error::LlaveError::NoPendingPin) when
/// there is no pending PIN to reveal.
pub async fn request_pin(client: &LlaveClient, session: &Session) -> Result<(String, String)> {
    tracing::info!(
        device_id = %session.device_id,
        "requesting pin"
    );

    // Step 1: Establish device session with the server (mirrors app startup call).
    match client
        .clave_starting(&session.device_id, &session.nif, "")
        .await
    {
        Ok(resp) => {
            tracing::info!(
                status = %resp.status,
                code = resp.codigo_error.as_deref().unwrap_or(""),
                "ClaveStartingSv before PIN request"
            );
            if resp.status == "OK" {
                ensure_app_movil_cookie(client).await;
            } else {
                // The app stops (and wipes on 105/109) when Starting reports
                // an inactive device.
                check_device_active(resp.codigo_error.as_deref(), resp.mensaje.as_deref())?;
            }
        }
        Err(e) => {
            tracing::warn!(err = %e, "ClaveStartingSv failed (continuing anyway)");
        }
    }

    // Step 2: Call ClaveRequestAllOperationsSv to validate the device session
    // and check for a pending PIN.  The original app does this when the
    // device active screen loads (consultarClaveMovilOPin).
    let poll_timestamp = current_cursor(session);

    let ops = client
        .request_all_operations(&session.device_id, &session.nif, &poll_timestamp)
        .await?;
    tracing::info!(
        status = %ops.status,
        code = ops.codigo_error.as_deref().unwrap_or(""),
        has_respuesta = ops.respuesta.is_some(),
        "ClaveRequestAllOperationsSv before PIN"
    );

    // Device registration errors (the app shows its "not active device" dialog).
    check_device_active(ops.codigo_error.as_deref(), ops.mensaje.as_deref())?;

    let respuesta = ops.into_result()?;
    let has_pin = has_pending_pin(&respuesta);
    tracing::info!(
        has_pin = has_pin,
        has_peticion = respuesta.get("peticion").is_some_and(|v| !v.is_null()),
        "AllOperations response payload"
    );

    record_cursor(&respuesta);

    if !has_pin {
        return Err(crate::error::LlaveError::NoPendingPin);
    }

    // Step 3: Reveal the pending PIN.
    let resp = client
        .request_pin(&session.device_id, &session.device_password, &session.nif)
        .await?;

    // 307 = "no valid PIN available" (the PIN expired between both calls).
    if resp.codigo_error.as_deref() == Some("307") {
        return Err(crate::error::LlaveError::NoPendingPin);
    }
    let pin_data = resp.into_result()?;
    let pin = pin_data.pin.unwrap_or_default();
    let ttl_ms = pin_data.time_to_live.unwrap_or_default();
    let ttl = ttl_ms_to_seconds(&ttl_ms).unwrap_or_else(|| {
        tracing::warn!(ttl_ms = %ttl_ms, "unparseable timeToLive");
        String::new()
    });
    tracing::info!(ttl_seconds = %ttl, "pin revealed");

    Ok((pin, ttl))
}

/// DNI/NIE weak authentication flow.
///
/// This endpoint returns HTML (not JSON).  The only purpose is to establish
/// session cookies that subsequent endpoints (`ClaveRequestStateSv`, etc.)
/// rely on.
pub async fn authenticate_dni(
    client: &LlaveClient,
    nif: &str,
    fecha: &str,
    soporte: &str,
    challenge: Option<crate::api::DniChallenge>,
) -> Result<()> {
    client
        .authenticate_dni_nie(nif, fecha, soporte, challenge)
        .await
}

/// Result of phase 1: DNI auth + registration check + SMS request.
///
/// Contains the registration state, SMS metadata for validation,
/// and serialised cookies to resume the session in phase 2.
pub struct DniSmsPhase1 {
    pub state: crate::api::ClaveRequestStateResponse,
    pub sms: crate::api::ObtenerSmsResponse,
    /// Serialised cookies — pass to [`dni_validate_and_activate`] to resume.
    pub cookies_json: String,
}

/// Phase 1: DNI/NIE auth → registration check → request SMS code.
///
/// Matches the Android app's sequence up to the point where the user
/// must enter the SMS PIN:
/// 1. `AutenticaDniNieContrasteh` (DNI/NIE weak auth on www2)
/// 2. `ClaveRequestStateSv` (registration check on www12)
/// 3. `ObtenerClaveMovilSMS` (trigger SMS on www12)
///
/// Returns [`DniSmsPhase1`] with the SMS metadata and serialised cookies.
/// The caller should display the masked phone number to the user, collect
/// the SMS PIN, then call [`dni_validate_and_activate`] with the PIN.
///
/// On [`LlaveError::DniBirthDateRequired`](crate::error::LlaveError::DniBirthDateRequired)
/// call again with the date of birth as `fecha`, an empty `soporte` and the
/// returned challenge.
pub async fn dni_request_sms(
    client: &LlaveClient,
    nif: &str,
    fecha: &str,
    soporte: &str,
    challenge: Option<crate::api::DniChallenge>,
) -> Result<DniSmsPhase1> {
    // Like the app entering its identification screen: start from a clean
    // cookie jar (resubmissions with a challenge keep the current one).
    if challenge.is_none() {
        client.clear_session_cookies();
    }

    // Step 0: ClaveStartingSv on www2, as the app does at launch (before DNI
    // auth).  An unactivated app has no device id yet and sends it empty.
    tracing::info!("calling ClaveStartingSv (www2) to establish device session");
    match client.clave_starting("", nif, "").await {
        Ok(resp) if resp.status == "OK" => {
            tracing::info!("ClaveStartingSv succeeded");
            ensure_app_movil_cookie(client).await;
        }
        Ok(resp) => {
            tracing::warn!(
                status = %resp.status,
                code = resp.codigo_error.as_deref().unwrap_or("?"),
                message = resp.mensaje.as_deref().unwrap_or("?"),
                "ClaveStartingSv returned non-OK (continuing anyway)"
            );
        }
        Err(e) => {
            tracing::warn!(err = %e, "ClaveStartingSv failed (continuing anyway)");
        }
    }

    // Step 1: DNI/NIE auth (establishes session cookies).
    tracing::info!("authenticating via DNI/NIE");
    client
        .authenticate_dni_nie(nif, fecha, soporte, challenge)
        .await?;

    // Step 2: Check registration state (uses session cookies from step 1).
    tracing::info!("checking registration state");
    let mut state_resp = client.clave_request_state().await?;
    if needs_app_movil_retry(&state_resp.status, state_resp.codigo_error.as_deref()) {
        // The app re-requests the `appmovil` cookie and retries once.
        tracing::info!(
            "ClaveRequestStateSv returned 987 — refreshing appmovil cookie and retrying"
        );
        if let Err(e) = client.set_cookie_app_movil().await {
            tracing::warn!(err = %e, "SetCookieAppMovil failed before retry");
        }
        state_resp = client.clave_request_state().await?;
    }
    if state_resp.status != "OK" {
        return Err(state_resp.ko_error("Registration state check failed"));
    }
    let state = state_resp
        .respuesta
        .ok_or_else(|| crate::error::LlaveError::Api {
            status: "OK".into(),
            code: String::new(),
            message: "Empty ClaveRequestState response".into(),
        })?;
    tracing::info!(
        registrado = state.registrado.as_deref().unwrap_or("?"),
        telefono = state.telefono.as_deref().unwrap_or("?"),
        nivel = state.nivel_registro.as_deref().unwrap_or("?"),
        "registration state"
    );
    registration_gate(&state)?;

    // Step 3: Request SMS code (triggers SMS to user's registered phone).
    tracing::info!("requesting SMS verification code");
    let sms_resp = client.request_sms_code().await?;
    if sms_resp.status != "OK" {
        return Err(sms_resp.ko_error("SMS request failed"));
    }
    let sms = sms_resp
        .respuesta
        .ok_or_else(|| crate::error::LlaveError::Api {
            status: "OK".into(),
            code: String::new(),
            message: "Empty ObtenerClaveMovilSMS response".into(),
        })?;
    tracing::info!(
        movil = %sms.movil,
        hora = %sms.hora_peticion,
        "SMS code sent"
    );

    // Export cookies so they can be restored in phase 2.
    let cookies_json = client.export_cookies();

    Ok(DniSmsPhase1 {
        state,
        sms,
        cookies_json,
    })
}

/// Phase 2: Validate SMS code → activate device on www6.
///
/// Resumes the session from phase 1 using the serialised cookies,
/// validates the user-entered SMS PIN, then activates the device.
///
/// 4. `ValidarClaveMovilSMS` (validate SMS code on www12 → sets pin24H cookie)
/// 5. `ClaveActivateAuthenticationSv` (device activation on www6)
pub async fn dni_validate_and_activate(
    client: &LlaveClient,
    cookies_json: &str,
    nif: &str,
    timestamp_alta_sms: &str,
    token_clave_movil_sms: &str,
    sms_pin: &str,
    device_password: &str,
) -> Result<Session> {
    // Restore session cookies from phase 1.
    client.import_cookies(cookies_json);
    tracing::info!("restored session cookies from phase 1");

    // Step 4: Validate SMS code (sets pin24H cookie on www12).
    tracing::info!("validating SMS code");
    let validate_resp = client
        .validate_sms_code(timestamp_alta_sms, token_clave_movil_sms, sms_pin)
        .await?;
    if validate_resp.status != "OK" {
        return Err(validate_resp.ko_error("SMS validation failed"));
    }
    tracing::info!("SMS code validated (pin24H cookie should be set)");

    // Step 5: Activate device on www6 (now has pin24H cookie).
    tracing::info!("activating device");
    let activate_resp = client.activate_authentication(device_password, "").await?;
    if activate_resp.status != "OK" {
        return Err(activate_resp.ko_error("Device activation failed"));
    }

    let activate_data = activate_resp
        .respuesta
        .ok_or_else(|| crate::error::LlaveError::Api {
            status: "OK".into(),
            code: String::new(),
            message: "Empty activation response".into(),
        })?;

    let session = session_from_activation(activate_data, nif, device_password)?;
    session.save()?;
    tracing::info!("device activated via DNI/NIE + SMS flow");

    Ok(session)
}

/// Poll for pending authentication requests (single poll).
///
/// Uses `ClaveRequestAllOperationsSv` on www2 which only needs device
/// credentials.  The timestamp acts as a cursor — on first call we send
/// the epoch value and on subsequent calls we re-use the timestamp from
/// the previous successful response so the server only returns newer
/// requests.
pub async fn poll_pending_requests(
    client: &LlaveClient,
    session: &Session,
) -> Result<serde_json::Value> {
    let timestamp = current_cursor(session);
    tracing::debug!(timestamp = %timestamp, "polling pending requests");

    let operations = client
        .request_all_operations(&session.device_id, &session.nif, &timestamp)
        .await?;

    // Server returns KO when there are no pending operations (e.g. code 205).
    // Propagate as a proper error so callers can distinguish "no data" from
    // "got data with null fields".
    if operations.status != "OK" {
        check_device_active(
            operations.codigo_error.as_deref(),
            operations.mensaje.as_deref(),
        )?;
        return Err(operations.ko_error("No pending operations"));
    }

    // Like the app: ignore a request older than the cursor, then advance it.
    let mut respuesta = operations.respuesta;
    if let Some(resp) = respuesta.as_mut() {
        drop_stale_peticion(resp, &timestamp);
        record_cursor(resp);
    }

    Ok(serde_json::json!({
        "source": "all_operations",
        "status": operations.status,
        "data": respuesta,
    }))
}

/// Whether a polling error just means "nothing yet, poll again".
fn keep_polling(err: &crate::LlaveError) -> bool {
    matches!(err, crate::LlaveError::Api { code, .. }
        if is_no_pending_code(code) || is_retry_later_code(code))
}

/// One polling step for a "waiting for login" loop driven by the caller.
///
/// Returns `Some(peticion)` when an actionable request is pending, `None`
/// when there is nothing yet (including the codes that mean "keep polling"),
/// and an error for anything that should stop the loop (e.g. device revoked).
pub async fn next_pending_request(
    client: &LlaveClient,
    session: &Session,
) -> Result<Option<serde_json::Value>> {
    match poll_pending_requests(client, session).await {
        Ok(result) => Ok(result
            .get("data")
            .filter(|data| has_pending_request(data))
            .and_then(|data| data.get("peticion").cloned())),
        Err(e) if keep_polling(&e) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Confirm a pending Llave Móvil authentication request.
///
/// Mirrors the official app's `confirmarPeticionAutenticacionAndContinue`:
/// 1. `ClaveAuthenticateSv` on www2 — tells AEAT we accept.
/// 2. If the `WWW12` cookie is present, `ValidarClaveMovil` on www12 —
///    completes the SSO session so the service provider (e.g. Seguridad
///    Social) can finalise its login redirect.
///
/// Note: the official app does **not** call `ClaveStartingSv` before
/// authenticate.  Doing so resets server-side session state and causes
/// error 205.
pub async fn confirm_authentication(
    client: &LlaveClient,
    session: &Session,
    token_clave_movil: &str,
    codigo_idp: &str,
) -> Result<serde_json::Value> {
    tracing::info!("confirming auth request");

    let resp = client
        .authenticate(
            &session.device_id,
            &session.device_password,
            &session.nif,
            token_clave_movil,
            codigo_idp,
        )
        .await?;

    if resp.status != "OK" {
        return Err(resp.ko_error("Confirm failed"));
    }

    // Step 2: validate on www12 to complete the SSO session (matches APK).
    if client.has_cookie("WWW12") {
        tracing::info!("WWW12 cookie present — calling ValidarClaveMovil");
        match client.validate_llave_movil(token_clave_movil).await {
            Ok(v) => {
                tracing::info!(status = %v.status, "ValidarClaveMovil response");
            }
            Err(e) => {
                tracing::warn!(err = %e, "ValidarClaveMovil failed (non-fatal)");
            }
        }
    } else {
        tracing::info!("no WWW12 cookie — skipping ValidarClaveMovil");
    }

    let pending = resp
        .respuesta
        .as_ref()
        .and_then(|r| r.pending_requests.as_deref());
    Ok(serde_json::json!({
        "status": resp.status,
        "pending_requests": pending,
        "more_pending": more_pending(pending),
    }))
}

/// Deep link the app registers for QR codes (`AndroidManifest`, path prefix).
const QR_DEEP_LINK_PREFIX: &str = "https://sede.agenciatributaria.gob.es/appclave/qrscan";

/// Accept either the raw QR text or the app's deep link
/// (`…/appclave/qrscan?qrcode=<value>`).  Like the app, the value after
/// `qrcode=` is taken from the encoded query without percent-decoding.
pub fn qr_value_from_input(input: &str) -> &str {
    let input = input.trim();
    if input.starts_with(QR_DEEP_LINK_PREFIX) {
        if let Some((_, value)) = input.split_once("qrcode=") {
            return value;
        }
    }
    input
}

/// Mirror the app's QR check: leading `"0"`, Base64 remainder whose decoded
/// length is a multiple of 8, and at least 152 characters overall.
pub fn is_valid_qr(value: &str) -> bool {
    use base64::engine::{general_purpose, DecodePaddingMode, GeneralPurpose};
    use base64::{alphabet, Engine};
    // Android's `Base64.DEFAULT` decoder does not insist on padding.
    const LENIENT: GeneralPurpose = GeneralPurpose::new(
        &alphabet::STANDARD,
        general_purpose::PAD.with_decode_padding_mode(DecodePaddingMode::Indifferent),
    );

    let Some(payload) = value.strip_prefix('0') else {
        return false;
    };
    value.len() >= 152
        && LENIENT
            .decode(payload)
            .is_ok_and(|bytes| bytes.len() % 8 == 0)
}

/// Authenticate a QR code shown by the AEAT website (`ClaveMovilQrSv`).
///
/// Validates the code like the app, fails on KO, and advances the polling
/// cursor from the returned `peticion`.  The returned request must then be
/// confirmed or rejected like any pending request.
pub async fn qr_authenticate(
    client: &LlaveClient,
    session: &Session,
    input: &str,
) -> Result<serde_json::Value> {
    let value = qr_value_from_input(input);
    if !is_valid_qr(value) {
        return Err(crate::LlaveError::InvalidQr);
    }

    let resp = client
        .qr_authenticate(&session.device_id, &session.nif, value)
        .await?;
    let respuesta = resp.into_result()?;
    record_cursor(&respuesta);
    Ok(respuesta)
}

/// Reject a pending Llave Móvil authentication request.
///
/// Note: unlike polling, the official app does **not** call `ClaveStartingSv`
/// before `ClaveCancelAuthenticateSv`.  Doing so resets server-side session
/// state and causes error 205 ("datos no correctos").
pub async fn reject_authentication(
    client: &LlaveClient,
    session: &Session,
    token_clave_movil: &str,
    codigo_idp: &str,
) -> Result<serde_json::Value> {
    tracing::info!("rejecting auth request");

    let resp = client
        .cancel_authenticate(
            &session.device_id,
            &session.nif,
            token_clave_movil,
            codigo_idp,
        )
        .await?;

    if resp.status != "OK" {
        return Err(resp.ko_error("Reject failed"));
    }

    let pending = resp
        .respuesta
        .as_ref()
        .and_then(|r| r.get("pending_requests"))
        .and_then(|v| v.as_str());
    Ok(serde_json::json!({
        "status": resp.status,
        "pending_requests": pending,
        "more_pending": more_pending(pending),
        "response": resp.respuesta,
    }))
}

/// Whether to look for another request after a confirm or reject. Like the
/// app, a missing `pending_requests` counts as "S" (`ClaveDeviceActiveFragment`).
fn more_pending(pending_requests: Option<&str>) -> bool {
    pending_requests.unwrap_or("S") == "S"
}

/// Continuously poll for pending requests (blocking loop for CLI).
///
/// `max_attempts == 0` polls until a request arrives.
pub async fn listen_for_requests(
    client: &LlaveClient,
    session: &Session,
    interval_secs: u64,
    max_attempts: u32,
) -> Result<serde_json::Value> {
    let mut attempt: u32 = 1;
    while attempt_allowed(attempt, max_attempts) {
        tracing::debug!(attempt = attempt, max = max_attempts, "polling");

        // Same rule as `next_pending_request`, keeping the full poll result.
        match poll_pending_requests(client, session).await {
            Ok(result) if result.get("data").is_some_and(has_pending_request) => return Ok(result),
            Ok(_) => {}
            Err(e) if keep_polling(&e) => {
                tracing::debug!(err = %e, "no pending operations, continuing poll");
            }
            Err(e) => return Err(e),
        }

        attempt = attempt.saturating_add(1);
        if attempt_allowed(attempt, max_attempts) {
            tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
        }
    }

    Ok(serde_json::json!({
        "status": "timeout",
        "message": format!("No pending requests found after {max_attempts} attempts"),
    }))
}

/// `resultado` filter of `ClaveOperationsHistorySv`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistoryResult {
    /// Confirmed and rejected requests.
    All,
    /// Confirmed requests only — the app's default filter.
    #[default]
    Confirmed,
    /// Rejected requests only.
    Rejected,
}

impl HistoryResult {
    pub fn code(self) -> &'static str {
        match self {
            Self::All => "00",
            Self::Confirmed => "01",
            Self::Rejected => "10",
        }
    }
}

/// Result of [`check_nif`].
pub struct NifCheck {
    pub nif: String,
    /// `OK` when the NIF is activated in Cl@ve, `KO` otherwise.
    pub status: String,
    pub response: Option<crate::api::StartingResponse>,
}

/// Whether a NIF is activated (`ClaveIsNifActivatedSv`): `nif`, or the
/// configured one. Like the app, it sends the stored device id (empty
/// when this device is not activated).
pub async fn check_nif(client: &LlaveClient, nif: Option<&str>) -> Result<NifCheck> {
    let nif = match nif {
        Some(n) => crate::config::validate_nif(n)?,
        None => crate::Config::load()?.nif.ok_or_else(|| {
            crate::LlaveError::Config(
                "No NIF configured. Pass a NIF or activate this device first.".into(),
            )
        })?,
    };
    let device_id = Session::load().map(|s| s.device_id).unwrap_or_default();
    let resp = client.clave_is_nif_activated(&device_id, &nif).await?;
    Ok(NifCheck {
        nif,
        status: resp.status,
        response: resp.respuesta,
    })
}

/// The app shows the last week by default.
pub const HISTORY_DEFAULT_DAYS: i64 = 7;

/// `(fecha_desde, fecha_hasta)` in the app's `dd/MM/yyyy` format.
fn history_range(today: chrono::NaiveDate, days: i64) -> (String, String) {
    let desde = today - chrono::Duration::days(days);
    (
        desde.format("%d/%m/%Y").to_string(),
        today.format("%d/%m/%Y").to_string(),
    )
}

/// Operations history for the last `days` days (`ClaveOperationsHistorySv`).
pub async fn history(
    client: &LlaveClient,
    session: &Session,
    days: i64,
    resultado: HistoryResult,
) -> Result<Vec<crate::api::OperationEntry>> {
    let (desde, hasta) = history_range(chrono::Local::now().date_naive(), days);
    let resp = client
        .operations_history(
            &session.device_id,
            &session.device_password,
            &session.nif,
            &desde,
            &hasta,
            resultado.code(),
        )
        .await?;
    check_device_active(resp.codigo_error.as_deref(), resp.mensaje.as_deref())?;
    resp.into_result()
}

/// Account data, like the app: `ClaveIsNifActivatedSv` must be OK before
/// `ClaveCheckMyDataSv`.
pub async fn my_data(
    client: &LlaveClient,
    session: &Session,
) -> Result<crate::api::ClaveCheckMyDataResponse> {
    let activated = client
        .clave_is_nif_activated(&session.device_id, &session.nif)
        .await?;
    if activated.status != "OK" {
        check_device_active(
            activated.codigo_error.as_deref(),
            activated.mensaje.as_deref(),
        )?;
        activated.into_result()?;
    }
    let resp = client
        .clave_check_my_data(&session.device_id, &session.nif)
        .await?;
    check_device_active(resp.codigo_error.as_deref(), resp.mensaje.as_deref())?;
    resp.into_result()
}

/// Deactivate this device (`ClaveDesactivateAuthSv`).
///
/// Like the app, the local session is deleted only when the server confirms
/// (or reports the device as already revoked, 105/109); on any other KO the
/// credentials are kept so the user can retry.
pub async fn deactivate(client: &LlaveClient, session: &Session) -> Result<()> {
    let resp = client
        .deactivate_authentication(&session.device_id, &session.nif)
        .await?;
    if resp.status != "OK" {
        check_device_active(resp.codigo_error.as_deref(), resp.mensaje.as_deref())?;
        resp.into_result()?;
        return Ok(());
    }
    Session::delete()
}

/// Classify device-status codes.  `Some(true)`: device revoked, the app
/// wipes local credentials (105/109).  `Some(false)`: device not active,
/// no wipe (416).  `None`: not a device-status error.
fn device_inactive(code: Option<&str>) -> Option<bool> {
    match code? {
        "105" | "109" => Some(true),
        "416" => Some(false),
        _ => None,
    }
}

/// Fail with [`LlaveError::DeviceNotActive`](crate::LlaveError::DeviceNotActive)
/// on a device-status code, deleting the local session first when the app
/// would (105/109).
fn check_device_active(code: Option<&str>, message: Option<&str>) -> Result<()> {
    let Some(wipe) = device_inactive(code) else {
        return Ok(());
    };
    if wipe {
        tracing::warn!(
            code = code.unwrap_or_default(),
            "device revoked by server — deleting local session"
        );
        if let Err(e) = Session::delete() {
            tracing::warn!(err = %e, "could not delete local session");
        }
    }
    Err(crate::LlaveError::DeviceNotActive {
        code: code.unwrap_or_default().to_string(),
        message: message.unwrap_or_default().to_string(),
    })
}

/// The app only requests the SMS for a registered user with a phone.
fn registration_gate(state: &crate::api::ClaveRequestStateResponse) -> Result<()> {
    if state.registrado.as_deref() != Some("true") {
        return Err(crate::LlaveError::NotRegistered);
    }
    if state.telefono.as_deref() != Some("S") {
        return Err(crate::LlaveError::NoRegisteredPhone);
    }
    Ok(())
}

/// Code 407: the app asks the user to wait a few seconds and retry.
fn is_retry_later_code(code: &str) -> bool {
    code == "407"
}

/// Build the session from a `ClaveActivateAuthenticationSv` payload.
///
/// The server assigns `device_id` (required, like the app's `checkNotNull`);
/// the device password is always the locally generated one — the response
/// carries only `device_id`, `nif` and `token`.
fn session_from_activation(
    activation: crate::api::ActivateResponse,
    nif: &str,
    device_password: &str,
) -> Result<Session> {
    let device_id = activation
        .device_id
        .filter(|id| !id.is_empty())
        .ok_or_else(|| crate::error::LlaveError::Api {
            status: "OK".into(),
            code: String::new(),
            message: "Activation response has no device_id".into(),
        })?;
    // The app stores the NIF returned by the server.
    let nif = activation
        .nif
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| nif.to_string());
    Ok(Session {
        device_id,
        nif,
        device_password: Zeroizing::new(device_password.to_string()),
        created_at: chrono::Utc::now().to_rfc3339(),
        poll_cursor: None,
    })
}

/// After a successful `ClaveStartingSv`, request the `appmovil` cookie if the
/// server did not set it — mirrors the app.  Non-fatal: failures are logged.
async fn ensure_app_movil_cookie(client: &LlaveClient) {
    if client.has_cookie("appmovil") {
        return;
    }
    match client.set_cookie_app_movil().await {
        Ok(resp) => tracing::info!(
            status = %resp.status,
            has_appmovil = client.has_cookie("appmovil"),
            "SetCookieAppMovil"
        ),
        Err(e) => tracing::warn!(err = %e, "SetCookieAppMovil failed (continuing anyway)"),
    }
}

/// `ClaveRequestStateSv` KO with code 987: the app calls `SetCookieAppMovil`
/// and retries once.  (Meaning of 987 is not documented; inferred to be a
/// missing/invalid `appmovil` cookie.)
fn needs_app_movil_retry(status: &str, code: Option<&str>) -> bool {
    status == "KO" && code == Some("987")
}

/// Error codes that indicate "no pending operations" rather than a real failure.
///
/// `40` is the code the app maps to its "no request" screen; `205` was
/// observed empirically when polling without pending requests.
fn is_no_pending_code(code: &str) -> bool {
    matches!(code, "40" | "205")
}

/// Whether a polled `respuesta` contains an actionable authentication request.
///
/// Mirrors the app, which only shows a request when
/// `peticion.tokenClaveMovil` is set; a pending PIN or a bare `timestamp`
/// is not a request.
fn has_pending_request(data: &serde_json::Value) -> bool {
    data.get("peticion")
        .and_then(|p| p.get("tokenClaveMovil"))
        .and_then(|t| t.as_str())
        .is_some_and(|t| !t.is_empty())
}

/// Whether polling attempt number `attempt` (1-based) may run.
/// `max_attempts == 0` means unlimited.
fn attempt_allowed(attempt: u32, max_attempts: u32) -> bool {
    max_attempts == 0 || attempt <= max_attempts
}

/// Whether a `ClaveRequestAllOperationsSv` payload announces a pending PIN.
///
/// The app only reveals a PIN (via `ClaveRequestPinSv`) when
/// `respuesta.pin` is present; otherwise it shows "no pending PIN".
fn has_pending_pin(respuesta: &serde_json::Value) -> bool {
    respuesta.get("pin").is_some_and(|p| p.is_object())
}

/// Convert the server's `timeToLive` (milliseconds) to whole seconds.
///
/// The app feeds this value straight into `CountDownTimer(millisInFuture)`.
fn ttl_ms_to_seconds(ttl_ms: &str) -> Option<String> {
    ttl_ms
        .trim()
        .parse::<u64>()
        .ok()
        .map(|ms| (ms / 1000).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn more_pending_follows_the_app() {
        assert!(more_pending(Some("S")));
        assert!(more_pending(None));
        assert!(!more_pending(Some("N")));
    }

    // -- has_pending_pin --

    #[test]
    fn pending_pin_present() {
        let resp = serde_json::json!({
            "pin": {"timeToLive": "120000", "timestamp": "20261015T120000000Z"},
            "peticion": null,
        });
        assert!(has_pending_pin(&resp));
    }

    #[test]
    fn pending_pin_null() {
        let resp = serde_json::json!({"pin": null, "peticion": null});
        assert!(!has_pending_pin(&resp));
    }

    #[test]
    fn pending_pin_missing() {
        let resp = serde_json::json!({"timestamp": "20261015T120000000Z"});
        assert!(!has_pending_pin(&resp));
    }

    // -- history filters --

    #[test]
    fn history_result_codes_match_app() {
        assert_eq!(
            [
                HistoryResult::All,
                HistoryResult::Confirmed,
                HistoryResult::Rejected
            ]
            .map(HistoryResult::code),
            ["00", "01", "10"]
        );
    }

    #[test]
    fn history_default_is_confirmed_only() {
        assert_eq!(HistoryResult::default(), HistoryResult::Confirmed);
    }

    #[test]
    fn history_default_range_is_last_week() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
        assert_eq!(
            history_range(today, HISTORY_DEFAULT_DAYS),
            ("09/10/2026".to_string(), "16/10/2026".to_string())
        );
    }

    // -- device status --

    #[test]
    fn device_code_105_is_inactive_and_wipes() {
        assert_eq!(device_inactive(Some("105")), Some(true));
    }

    #[test]
    fn device_code_109_is_inactive_and_wipes() {
        assert_eq!(device_inactive(Some("109")), Some(true));
    }

    #[test]
    fn device_code_416_is_inactive_without_wipe() {
        assert_eq!(device_inactive(Some("416")), Some(false));
    }

    #[test]
    fn device_other_codes_are_not_device_errors() {
        assert_eq!(device_inactive(Some("40")), None);
    }

    // -- registration gate --

    fn state(registrado: &str, telefono: &str) -> crate::api::ClaveRequestStateResponse {
        crate::api::ClaveRequestStateResponse {
            registrado: Some(registrado.into()),
            nivel_registro: None,
            telefono: Some(telefono.into()),
        }
    }

    #[test]
    fn registration_gate_passes_registered_with_phone() {
        assert!(registration_gate(&state("true", "S")).is_ok());
    }

    #[test]
    fn registration_gate_rejects_unregistered() {
        assert!(matches!(
            registration_gate(&state("false", "S")),
            Err(crate::LlaveError::NotRegistered)
        ));
    }

    #[test]
    fn registration_gate_rejects_missing_phone() {
        assert!(matches!(
            registration_gate(&state("true", "N")),
            Err(crate::LlaveError::NoRegisteredPhone)
        ));
    }

    // -- keep_polling --

    fn api_error(code: &str) -> crate::LlaveError {
        crate::LlaveError::Api {
            status: "KO".into(),
            code: code.into(),
            message: String::new(),
        }
    }

    #[test]
    fn keep_polling_on_no_request_codes() {
        assert!(["40", "205", "407"]
            .iter()
            .all(|c| keep_polling(&api_error(c))));
    }

    #[test]
    fn stop_polling_on_other_errors() {
        assert!(!keep_polling(&api_error("105")));
    }

    #[test]
    fn stop_polling_on_device_not_active() {
        let err = crate::LlaveError::DeviceNotActive {
            code: "109".into(),
            message: String::new(),
        };
        assert!(!keep_polling(&err));
    }

    // -- retry-later code --

    #[test]
    fn code_407_means_retry_later() {
        assert!(is_retry_later_code("407"));
    }

    // -- QR --

    /// "0" + Base64 of `n` bytes, like the codes the AEAT site shows.
    fn qr_with_payload(n: usize) -> String {
        use base64::Engine;
        format!(
            "0{}",
            base64::engine::general_purpose::STANDARD.encode(vec![7u8; n])
        )
    }

    #[test]
    fn qr_valid_code_accepted() {
        assert!(is_valid_qr(&qr_with_payload(120)));
    }

    #[test]
    fn qr_rejects_wrong_prefix() {
        let qr = qr_with_payload(120).replacen('0', "1", 1);
        assert!(!is_valid_qr(&qr));
    }

    #[test]
    fn qr_rejects_payload_not_multiple_of_8() {
        assert!(!is_valid_qr(&qr_with_payload(121)));
    }

    #[test]
    fn qr_rejects_short_code() {
        assert!(!is_valid_qr(&qr_with_payload(8)));
    }

    #[test]
    fn qr_rejects_invalid_base64() {
        let qr = format!("0{}", "!".repeat(160));
        assert!(!is_valid_qr(&qr));
    }

    #[test]
    fn qr_raw_value_is_trimmed() {
        assert_eq!(qr_value_from_input("  0abc \n"), "0abc");
    }

    #[test]
    fn qr_value_extracted_from_deep_link() {
        let url = "https://sede.agenciatributaria.gob.es/appclave/qrscan?qrcode=0AbC%2B";
        assert_eq!(qr_value_from_input(url), "0AbC%2B");
    }

    #[test]
    fn qr_unrelated_url_is_left_as_is() {
        let url = "https://example.com/appclave/qrscan?qrcode=0abc";
        assert_eq!(qr_value_from_input(url), url);
    }

    // -- poll cursor --

    #[test]
    fn cursor_prefers_in_memory_value() {
        assert_eq!(pick_cursor(Some("B"), Some("A")), "B");
    }

    #[test]
    fn cursor_falls_back_to_stored_value() {
        assert_eq!(pick_cursor(None, Some("A")), "A");
    }

    #[test]
    fn cursor_defaults_to_epoch() {
        assert_eq!(pick_cursor(None, None), EPOCH_TIMESTAMP);
    }

    #[test]
    fn cursor_read_from_peticion_timestamp() {
        let resp = serde_json::json!({"peticion": {"timestamp": "20261015T120000000Z"}});
        assert_eq!(peticion_timestamp(&resp), Some("20261015T120000000Z"));
    }

    #[test]
    fn cursor_ignores_top_level_timestamp() {
        let resp = serde_json::json!({"timestamp": "20261015T120000000Z", "peticion": null});
        assert_eq!(peticion_timestamp(&resp), None);
    }

    #[test]
    fn request_older_than_cursor_is_stale() {
        assert!(is_stale("20261015T110000000Z", "20261015T120000000Z"));
    }

    #[test]
    fn request_equal_to_cursor_is_not_stale() {
        assert!(!is_stale("20261015T120000000Z", "20261015T120000000Z"));
    }

    #[test]
    fn stale_request_is_dropped_from_payload() {
        let mut data = serde_json::json!({
            "peticion": {"tokenClaveMovil": "t", "timestamp": "20261015T110000000Z"}
        });
        drop_stale_peticion(&mut data, "20261015T120000000Z");
        assert!(!has_pending_request(&data));
    }

    #[test]
    fn fresh_request_is_kept_in_payload() {
        let mut data = serde_json::json!({
            "peticion": {"tokenClaveMovil": "t", "timestamp": "20261015T130000000Z"}
        });
        drop_stale_peticion(&mut data, "20261015T120000000Z");
        assert!(has_pending_request(&data));
    }

    // -- session_from_activation --

    fn activation(device_id: Option<&str>) -> crate::api::ActivateResponse {
        crate::api::ActivateResponse {
            device_id: device_id.map(str::to_string),
            nif: None,
            token: Some("push-token".into()),
        }
    }

    #[test]
    fn activation_session_prefers_server_nif() {
        let mut data = activation(Some("dev-1"));
        data.nif = Some("X1234567L".into());
        let session = session_from_activation(data, "12345678Z", "PWD").unwrap();
        assert_eq!(session.nif, "X1234567L");
    }

    #[test]
    fn activation_session_falls_back_to_input_nif() {
        let session =
            session_from_activation(activation(Some("dev-1")), "12345678Z", "PWD").unwrap();
        assert_eq!(session.nif, "12345678Z");
    }

    #[test]
    fn activation_session_uses_server_device_id() {
        let session =
            session_from_activation(activation(Some("dev-1")), "12345678Z", "PWD").unwrap();
        assert_eq!(session.device_id, "dev-1");
    }

    #[test]
    fn activation_session_stores_local_password() {
        let session =
            session_from_activation(activation(Some("dev-1")), "12345678Z", "PWD").unwrap();
        assert_eq!(session.device_password.as_str(), "PWD");
    }

    #[test]
    fn activation_fails_without_device_id() {
        assert!(session_from_activation(activation(None), "12345678Z", "PWD").is_err());
    }

    #[test]
    fn activation_fails_with_empty_device_id() {
        assert!(session_from_activation(activation(Some("")), "12345678Z", "PWD").is_err());
    }

    // -- needs_app_movil_retry --

    #[test]
    fn app_movil_retry_on_ko_987() {
        assert!(needs_app_movil_retry("KO", Some("987")));
    }

    #[test]
    fn app_movil_retry_not_on_other_codes() {
        assert!(!needs_app_movil_retry("KO", Some("105")));
    }

    #[test]
    fn app_movil_retry_not_on_ok() {
        assert!(!needs_app_movil_retry("OK", None));
    }

    // -- has_pending_request --

    #[test]
    fn pending_request_detected_when_peticion_has_token() {
        let data = serde_json::json!({
            "peticion": {"tokenClaveMovil": "abc", "codigoIdP": "1234"},
            "pin": null,
        });
        assert!(has_pending_request(&data));
    }

    #[test]
    fn pending_request_ignores_pin_only_payload() {
        let data = serde_json::json!({
            "pin": {"timeToLive": "120000"},
            "peticion": null,
            "timestamp": "20261015T120000000Z",
        });
        assert!(!has_pending_request(&data));
    }

    #[test]
    fn pending_request_ignores_empty_token() {
        let data = serde_json::json!({"peticion": {"tokenClaveMovil": ""}});
        assert!(!has_pending_request(&data));
    }

    #[test]
    fn pending_request_ignores_null_data() {
        assert!(!has_pending_request(&serde_json::Value::Null));
    }

    // -- is_no_pending_code --

    #[test]
    fn no_pending_code_accepts_app_code_40() {
        assert!(is_no_pending_code("40"));
    }

    #[test]
    fn no_pending_code_rejects_device_errors() {
        assert!(!is_no_pending_code("105"));
    }

    // -- attempt_allowed --

    #[test]
    fn attempts_unlimited_when_max_is_zero() {
        assert!(attempt_allowed(1_000_000, 0));
    }

    #[test]
    fn attempts_stop_after_max() {
        assert!(!attempt_allowed(4, 3));
    }

    #[test]
    fn attempts_allow_last_one() {
        assert!(attempt_allowed(3, 3));
    }

    // -- ttl_ms_to_seconds --

    #[test]
    fn ttl_converts_millis_to_seconds() {
        assert_eq!(ttl_ms_to_seconds("180000").as_deref(), Some("180"));
    }

    #[test]
    fn ttl_rounds_down_partial_seconds() {
        assert_eq!(ttl_ms_to_seconds("179999").as_deref(), Some("179"));
    }

    #[test]
    fn ttl_trims_whitespace() {
        assert_eq!(ttl_ms_to_seconds(" 60000 ").as_deref(), Some("60"));
    }

    #[test]
    fn ttl_rejects_non_numeric() {
        assert_eq!(ttl_ms_to_seconds("abc"), None);
        assert_eq!(ttl_ms_to_seconds(""), None);
    }
}
