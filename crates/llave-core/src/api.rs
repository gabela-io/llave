use crate::error::{LlaveError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use parking_lot::Mutex;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Shared client for the FFI layer — persists cookies across calls.
// ---------------------------------------------------------------------------

static SHARED_CLIENT: Mutex<Option<Arc<LlaveClient>>> = Mutex::new(None);

/// Get or create the shared [`LlaveClient`].
///
/// The client is lazily initialised on first use and reused for all
/// subsequent calls so that cookies accumulate across FFI invocations.
/// Call [`reset_shared_client`] when the proxy changes or on logout.
pub fn shared_client() -> Result<Arc<LlaveClient>> {
    let mut guard = SHARED_CLIENT.lock();
    if let Some(ref client) = *guard {
        return Ok(Arc::clone(client));
    }
    let client = Arc::new(LlaveClient::new()?);
    *guard = Some(Arc::clone(&client));
    Ok(client)
}

/// Drop the shared client so the next [`shared_client`] call creates a
/// fresh one.  Called on proxy change and logout.
pub fn reset_shared_client() {
    *SHARED_CLIENT.lock() = None;
}

/// Truncate a string to at most `max_bytes`, ensuring the cut falls on a
/// UTF-8 character boundary so indexing never panics.
/// Extra check the server asks for during DNI/NIE weak authentication.
///
/// When `AutenticaDniNieContrasteh` answers KO `-8` / `-10`, the app asks
/// for the date of birth and resends with `AZUL="X"` / `FECHANIE="X"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DniChallenge {
    /// Code `-8`: resend with `AZUL="X"`.
    Azul,
    /// Code `-10`: resend with `FECHANIE="X"`.
    FechaNie,
}

impl DniChallenge {
    /// `(AZUL, FECHANIE)` form values for a (re)submission.
    pub fn form_flags(challenge: Option<Self>) -> (&'static str, &'static str) {
        match challenge {
            None => ("", ""),
            Some(Self::Azul) => ("X", ""),
            Some(Self::FechaNie) => ("", "X"),
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        match code {
            "-8" => Some(Self::Azul),
            "-10" => Some(Self::FechaNie),
            _ => None,
        }
    }
}

/// Interpret an `AutenticaDniNieContrasteh` body like the app does.
///
/// Returns `None` when the body is not the JSON envelope (e.g. HTML after
/// redirects) — the caller then falls back to the redirect heuristic.
fn classify_dni_body(body: &str) -> Option<Result<()>> {
    let resp: ApiResponse<serde_json::Value> = serde_json::from_str(body).ok()?;
    if resp.status != "KO" {
        return Some(Ok(()));
    }
    let code = resp.codigo_error.unwrap_or_default();
    let message = resp.mensaje.unwrap_or_default();
    Some(Err(match DniChallenge::from_code(&code) {
        Some(challenge) => LlaveError::DniBirthDateRequired { challenge, message },
        None => LlaveError::Api {
            status: resp.status,
            code,
            message,
        },
    }))
}

/// Insert or update a cookie; an empty or `""` value deletes it, like the
/// app's `Repository.saveListCookiesInLocal` (server-side cookie removal).
fn store_cookie(cookies: &mut Vec<(String, String)>, name: &str, value: &str) {
    if value.is_empty() || value == "\"\"" {
        cookies.retain(|(k, _)| k != name);
    } else if let Some(existing) = cookies.iter_mut().find(|(k, _)| k == name) {
        existing.1 = value.to_string();
    } else {
        cookies.push((name.to_string(), value.to_string()));
    }
}

/// Cookies the app keeps when it clears the session
/// (`RemoveSessionCookiesUseCase`).
const PERSISTENT_COOKIES: [&str; 3] = ["appmovil", "sgat-language", "token-exteriores"];

/// Drop session cookies, keeping only [`PERSISTENT_COOKIES`].
fn retain_persistent_cookies(cookies: &mut Vec<(String, String)>) {
    cookies.retain(|(k, _)| PERSISTENT_COOKIES.contains(&k.as_str()));
}

/// OkHttp (used by the app) follows at most 20 redirects.
const MAX_REDIRECTS: u32 = 20;

/// Whether another redirect hop may be followed after `followed` hops.
fn redirect_allowed(followed: u32) -> bool {
    followed < MAX_REDIRECTS
}

/// Build a `Cookie` header from a cookie jar, mirroring the app's
/// `Repository.recuperarCookies(requireJsessionId = false)`: every stored
/// cookie is sent to every host, except `JSESSIONID` and empty values.
/// Only the app's PDF download includes `JSESSIONID`; no API call does.
fn build_cookie_header(cookies: &[(String, String)]) -> String {
    cookies
        .iter()
        .filter(|(k, v)| k != "JSESSIONID" && !v.is_empty())
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn truncate_str(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

const BASE_URL: &str = "https://www2.agenciatributaria.gob.es";
const BASE_URL_WWW6: &str = "https://www6.agenciatributaria.gob.es";
const BASE_URL_WWW12: &str = "https://www12.agenciatributaria.gob.es";

/// App version sent as `version_app` (app 6.2.10, build 294).
const APP_VERSION: &str = "6.2.10";
/// Android-style sistema_operativo value ("A" = Android).
/// The AEAT server uses this to determine session routing.
const OS_NAME: &str = "A";
const OS_VERSION: &str = "14";
/// Sent as `modelo`.  Mirrors the app's `DeviceUtils.getDeviceName()`:
/// capitalised `Build.MANUFACTURER` + " " + `Build.MODEL`.
const DEVICE_MODEL: &str = "Samsung SM-S928U";
/// User-Agent matching the Android app format so the AEAT server
/// recognises us as a mobile client (critical for session handling).
/// Format: APPMovil/Cl@ve/v{version}({build})/{os_ver}/Android/{System.getProperty("http.agent")}
/// The full Dalvik user-agent suffix is required — the truncated version
/// (without the parenthetical) may be rejected by the server/WAF.
/// The Dalvik suffix uses the bare `Build.MODEL`, not [`DEVICE_MODEL`].
const USER_AGENT: &str = "APPMovil/Cl@ve/v6.2.10(294)/14/Android/Dalvik/2.1.0 (Linux; U; Android 14; SM-S928U Build/UP1A.231005.007)";

/// Standard response envelope from all Llave API endpoints.
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub visible: Option<String>,
    pub crashlytics: Option<String>,
    pub codigo_error: Option<String>,
    pub mensaje: Option<String>,
    pub respuesta: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn into_result(self) -> Result<T> {
        if self.status == "OK" {
            self.respuesta
                .ok_or_else(|| LlaveError::Api {
                    status: self.status,
                    code: self.codigo_error.unwrap_or_default(),
                    message: "Empty response payload".into(),
                })
        } else {
            let code = self.codigo_error.unwrap_or_default();
            let message = self.mensaje.unwrap_or_else(|| "Unknown error".into());
            tracing::error!(status = %self.status, code = %code, msg = %message, "api error");
            if code == "16600" {
                return Err(LlaveError::IpRateLimited { message });
            }
            // The app shows an "update the app" dialog for this code.
            if code == "500600" {
                return Err(LlaveError::AppVersionOutdated { message });
            }
            Err(LlaveError::Api {
                status: self.status,
                code,
                message,
            })
        }
    }
}

// --- Response types ---

/// Payload of `ClaveStartingSv` and `ClaveIsNifActivatedSv` (URL lists).
#[derive(Debug, Deserialize, Serialize)]
pub struct StartingResponse {
    #[serde(rename = "listaBlancaUrls")]
    pub lista_blanca_urls: Option<Vec<String>>,
    #[serde(rename = "listaNegraUrls")]
    pub lista_negra_urls: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestPinResponse {
    pub pin: Option<String>,
    /// Remaining validity in **milliseconds** (as sent by the server).
    #[serde(rename = "timeToLive")]
    pub time_to_live: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthenticateResponse {
    pub pending_requests: Option<String>,
}

/// Response from ClaveCheckMyDataSv.
///
/// The `respuesta` contains `email`, `numTelefono`, and `nivelRegistro`.
#[derive(Debug, Deserialize, Serialize)]
pub struct ClaveCheckMyDataResponse {
    pub email: Option<String>,
    #[serde(rename = "numTelefono")]
    pub num_telefono: Option<String>,
    #[serde(rename = "nivelRegistro")]
    pub nivel_registro: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivateResponse {
    /// Server-assigned device identifier (required).
    pub device_id: Option<String>,
    /// NIF the server bound the device to; the app stores this value.
    pub nif: Option<String>,
    /// Session/push token — NOT the device password.
    pub token: Option<String>,
}

/// A single entry in the operations history list.
///
/// The `respuesta` field of `ClaveOperationsHistorySv` is a JSON array of
/// these objects (not a wrapper object).
#[derive(Debug, Deserialize, Serialize)]
pub struct OperationEntry {
    /// Date/time string, typically `DD/MM/YYYY HH:MM:SS`.
    pub fecha: Option<String>,
    /// Result code: `"01"` = confirmed/accepted, `"10"` = rejected.
    pub resultado: Option<String>,
    /// Type bitmask as a 3-digit binary string:
    /// bit 0 = DNI/NIE, bit 1 = QR, bit 2 = SMS.
    /// E.g. `"001"` = DNI, `"010"` = QR, `"111"` = all.
    pub tipo: Option<String>,
    /// Organisation that requested authentication (e.g. `"AEAT"`).
    pub organismo: Option<String>,
}

/// Response from ClaveRequestStateSv (registration check after DNI auth).
#[derive(Debug, Deserialize, Serialize)]
pub struct ClaveRequestStateResponse {
    pub registrado: Option<String>,
    #[serde(rename = "nivelRegistro")]
    pub nivel_registro: Option<String>,
    pub telefono: Option<String>,
}

/// Response payload from ObtenerClaveMovilSMS.
#[derive(Debug, Deserialize, Serialize)]
pub struct ObtenerSmsResponse {
    #[serde(rename = "timeStampAltaSms")]
    pub timestamp_alta_sms: String,
    #[serde(rename = "tokenClaveMovilSms")]
    pub token_clave_movil_sms: String,
    #[serde(rename = "horaPeticion")]
    pub hora_peticion: String,
    pub movil: String,
}

/// The Llave API client.
///
/// Keeps one cookie map for all `*.agenciatributaria.gob.es` subdomains and
/// sends it to every host, like the app's stored `map_cookies`.  Reqwest's
/// built-in cookie store follows RFC domain-matching rules, which would keep
/// cookies set by `www2` from reaching `www6` or `www12`.
pub struct LlaveClient {
    client: Client,
    /// All cookies collected from responses, forwarded to every request.
    cookies: Mutex<Vec<(String, String)>>,
}

impl LlaveClient {
    pub fn new() -> Result<Self> {
        // Do NOT use cookie_store(true) — we manage cookies manually to
        // propagate them across subdomains (www2 ↔ www12 ↔ www6).
        // Disable automatic redirects so we can re-attach cookies at each
        // hop (reqwest strips custom headers on cross-origin redirects).
        // Mirrors the app's SecuredHttpClient: TLS 1.2+, 30 s connect,
        // 190 s overall.
        let mut builder = Client::builder()
            .user_agent(USER_AGENT)
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .connect_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(190))
            .redirect(reqwest::redirect::Policy::none())
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(reqwest::header::ACCEPT, "application/json".parse().unwrap());
                h.insert(reqwest::header::ACCEPT_LANGUAGE, "es_ES".parse().unwrap());
                h
            });

        // Apply proxy from global config (supports http, https, socks5).
        if let Some(proxy_url) = crate::config::get_proxy() {
            tracing::info!(proxy = %proxy_url, "using proxy");
            builder = builder.proxy(
                reqwest::Proxy::all(&proxy_url)
                    .map_err(|e| LlaveError::Config(format!("invalid proxy URL: {e}")))?
            );
        }

        let client = builder.build()?;

        Ok(Self {
            client,
            // Pre-seed with sgat-language cookie, matching the Android app's
            // CookiesManagerSingleton which always sets this at startup.
            cookies: Mutex::new(vec![
                ("sgat-language".into(), "es_ES".into()),
            ]),
        })
    }

    /// Clear session cookies (keeps `appmovil`, `sgat-language`,
    /// `token-exteriores`), like the app on its identification screen.
    pub fn clear_session_cookies(&self) {
        retain_persistent_cookies(&mut self.cookies.lock());
    }

    /// Check whether a cookie with the given name exists in the jar.
    pub fn has_cookie(&self, name: &str) -> bool {
        let cookies = self.cookies.lock();
        cookies.iter().any(|(k, _)| k == name)
    }

    /// Export cookies as JSON for persistence across FFI calls.
    pub fn export_cookies(&self) -> String {
        let cookies = self.cookies.lock();
        serde_json::to_string(&*cookies).unwrap_or_else(|_| "[]".into())
    }

    /// Import cookies from a previously exported JSON string.
    pub fn import_cookies(&self, json: &str) {
        if let Ok(imported) = serde_json::from_str::<Vec<(String, String)>>(json) {
            let mut cookies = self.cookies.lock();
            for (name, value) in imported {
                if let Some(existing) = cookies.iter_mut().find(|(k, _)| k == &name) {
                    existing.1 = value;
                } else {
                    cookies.push((name, value));
                }
            }
        }
    }

    /// Build the `Cookie` header value from the stored cookies.
    fn cookie_header(&self) -> String {
        build_cookie_header(&self.cookies.lock())
    }

    /// Capture `Set-Cookie` headers from a response and store them.
    fn capture_cookies(&self, resp: &reqwest::Response) {
        // Also log raw Set-Cookie headers for debugging.
        let raw_set_cookies: Vec<String> = resp
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(|s| {
                // Truncate long values for readability.
                if s.len() > 80 {
                    format!("{}...", &s[..80])
                } else {
                    s.to_string()
                }
            })
            .collect();
        if !raw_set_cookies.is_empty() {
            tracing::info!(
                url = %resp.url(),
                set_cookies = ?raw_set_cookies,
                "captured Set-Cookie headers"
            );
        }

        let mut cookies = self.cookies.lock();
        for cookie in resp.cookies() {
            store_cookie(&mut cookies, cookie.name(), cookie.value());
        }
    }

    /// Follow HTTP redirects like OkHttp does in the app.
    ///
    /// Every hop reuses the request's original `cookie_header` (OkHttp copies
    /// the explicit `Cookie` header; the app has no cookie jar), while
    /// `Set-Cookie` from each hop is still captured into the jar.  At most
    /// [`MAX_REDIRECTS`] hops are followed.  Returns the final response and
    /// the number of redirects followed.
    async fn follow_redirects(
        &self,
        endpoint: &str,
        cookie_header: &str,
        mut resp: reqwest::Response,
    ) -> Result<(reqwest::Response, u32)> {
        let mut count = 0u32;
        while resp.status().is_redirection() {
            let location = resp
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string();
            if location.is_empty() {
                break;
            }
            if !redirect_allowed(count) {
                return Err(LlaveError::TooManyRedirects {
                    endpoint: endpoint.to_string(),
                    count,
                });
            }

            let next_url = if location.starts_with("http") {
                location
            } else {
                let base = resp.url().origin().unicode_serialization();
                format!("{base}{location}")
            };
            tracing::debug!(endpoint = endpoint, redirect_to = %next_url, "following redirect");

            let mut next = self.client.get(&next_url);
            if !cookie_header.is_empty() {
                next = next.header(reqwest::header::COOKIE, cookie_header);
            }
            resp = next.send().await?;
            count += 1;
            tracing::debug!(
                endpoint = endpoint,
                http_status = %resp.status(),
                redirect_count = count,
                "redirect response"
            );
            self.capture_cookies(&resp);
        }
        Ok((resp, count))
    }

    async fn post_form<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        url: &str,
        form: &[(&str, &str)],
    ) -> Result<ApiResponse<T>> {
        self.execute(endpoint, self.client.post(url).form(form)).await
    }

    /// POST with a JSON body (matching Retrofit `@POST` + `@Body`).
    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<ApiResponse<T>> {
        // Retrofit's Gson converter sends this exact content type.
        let req = self
            .client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json; charset=UTF-8")
            .json(body);
        self.execute(endpoint, req).await
    }

    /// POST with no body (matching Retrofit `@POST` without `@FormUrlEncoded`).
    ///
    /// Some AEAT endpoints (like ObtenerClaveMovilSMS) expect a bare POST
    /// with no Content-Type or body. Sending form-encoded data causes 902024.
    async fn post_empty<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        url: &str,
    ) -> Result<ApiResponse<T>> {
        // Empty body → Content-Length: 0, no Content-Type
        self.execute(endpoint, self.client.post(url).body("")).await
    }

    /// Attach cookies, send, follow redirects and decode the
    /// standard JSON envelope.  Shared by all `post_*` helpers.
    async fn execute<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        req: reqwest::RequestBuilder,
    ) -> Result<ApiResponse<T>> {
        let cookie_header = self.cookie_header();
        tracing::debug!(endpoint = endpoint, cookies = %cookie_header, "aeat request");
        let mut req = req;
        if !cookie_header.is_empty() {
            req = req.header(reqwest::header::COOKIE, &cookie_header);
        }

        let resp = req.send().await?;
        tracing::debug!(endpoint = endpoint, http_status = %resp.status(), "aeat response");
        self.capture_cookies(&resp);

        let (resp, _) = self.follow_redirects(endpoint, &cookie_header, resp).await?;

        let final_status = resp.status();
        let final_url = resp.url().to_string();
        let body = resp.text().await?;
        // Log at info level so response bodies are visible in normal output.
        tracing::info!(endpoint = endpoint, http_status = %final_status, final_url = %final_url, body_len = body.len(), body_preview = %truncate_str(&body, 512), "aeat response body");

        // Detect HTML responses early — the server returns the login page
        // when session cookies are missing or expired.
        let trimmed = body.trim_start();
        if trimmed.starts_with("<!DOCTYPE") || trimmed.starts_with("<html") {
            tracing::error!(
                endpoint = endpoint,
                body_preview = %truncate_str(&body, 512),
                "server returned HTML instead of JSON — session likely invalid"
            );
            return Err(LlaveError::HtmlResponse {
                endpoint: endpoint.to_string(),
            });
        }

        serde_json::from_str::<ApiResponse<T>>(&body).map_err(|e| {
            tracing::error!(
                endpoint = endpoint,
                err = %e,
                body_preview = %truncate_str(&body, 1024),
                "failed to decode response as JSON"
            );
            LlaveError::Json(e)
        })
    }

    /// Initialize a session via ClaveStartingSv on www2.
    pub async fn clave_starting(
        &self,
        device_id: &str,
        nif: &str,
        token_push: &str,
    ) -> Result<ApiResponse<StartingResponse>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveStartingSv");
        self.post_form("clave_starting", &url, &[
                ("device_id", device_id),
                ("NIF", nif),
                ("sistema_operativo", OS_NAME),
                ("token_push", token_push),
                ("version_os", OS_VERSION),
                ("version_app", APP_VERSION),
                ("modelo", DEVICE_MODEL),
            ]).await
    }

    /// Ask the server to set the `appmovil` cookie (www2, JSON body).
    ///
    /// The app calls this after a successful `ClaveStartingSv` when no
    /// `appmovil` cookie is stored, and before retrying `ClaveRequestStateSv`
    /// after error 987.
    pub async fn set_cookie_app_movil(&self) -> Result<ApiResponse<serde_json::Value>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-JDIT/C/SetCookieAppMovil");
        let body = serde_json::json!({
            "version_app": APP_VERSION,
            "sistema_operativo": OS_NAME,
            "version_os": OS_VERSION,
            "modelo": DEVICE_MODEL,
        });
        self.post_json("set_cookie_app_movil", &url, &body).await
    }

    /// Check this device/NIF against Cl@ve (www2).  Only `status` is
    /// meaningful; the payload is the same as `ClaveStartingSv`'s.
    pub async fn clave_is_nif_activated(
        &self,
        device_id: &str,
        nif: &str,
    ) -> Result<ApiResponse<StartingResponse>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveIsNifActivatedSv");
        self.post_form("clave_is_nif_activated", &url, &[
            ("device_id", device_id),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
        ]).await
    }

    /// Get account data via Clave endpoint (what the Android app actually uses).
    ///
    /// Returns `email`, `numTelefono`, and `nivelRegistro`.
    pub async fn clave_check_my_data(
        &self,
        device_id: &str,
        nif: &str,
    ) -> Result<ApiResponse<ClaveCheckMyDataResponse>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveCheckMyDataSv");
        self.post_form("clave_check_my_data", &url, &[
            ("device_id", device_id),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
        ]).await
    }

    /// Reveal the pending Cl@ve PIN (`ClaveRequestPinSv`).
    pub async fn request_pin(
        &self,
        device_id: &str,
        device_password: &str,
        nif: &str,
    ) -> Result<ApiResponse<RequestPinResponse>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveRequestPinSv");
        self.post_form("request_pin", &url, &[
            ("device_id", device_id),
            ("user_password", device_password),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
        ]).await
    }

    /// Confirm a Cl@ve Móvil authentication request (`ClaveAuthenticateSv`).
    pub async fn authenticate(
        &self,
        device_id: &str,
        device_password: &str,
        nif: &str,
        token_clave_movil: &str,
        codigo_idp: &str,
    ) -> Result<ApiResponse<AuthenticateResponse>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveAuthenticateSv");
        self.post_form("authenticate", &url, &[
            ("device_id", device_id),
            ("user_password", device_password),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
            ("tokenClaveMovil", token_clave_movil),
            ("codigoIdP", codigo_idp),
        ]).await
    }

    /// Reject a pending Cl@ve Móvil authentication request (`ClaveCancelAuthenticateSv`).
    pub async fn cancel_authenticate(
        &self,
        device_id: &str,
        nif: &str,
        token_clave_movil: &str,
        codigo_idp: &str,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveCancelAuthenticateSv");
        self.post_form("cancel_authenticate", &url, &[
            ("device_id", device_id),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
            ("tokenClaveMovil", token_clave_movil),
            ("codigoIdP", codigo_idp),
        ]).await
    }

    /// Activate device authentication.
    ///
    /// The endpoint only exists on www6.  Like every API call, the cookie
    /// header omits `JSESSIONID` (see [`build_cookie_header`]).
    pub async fn activate_authentication(
        &self,
        device_password: &str,
        token_push: &str,
    ) -> Result<ApiResponse<ActivateResponse>> {
        let url = format!("{BASE_URL_WWW6}/wlpl/MOVI-P24H/ClaveActivateAuthenticationSv");
        self.post_form("activate_authentication", &url, &[
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
            ("token_push", token_push),
            ("user_password", device_password),
            ("modelo", DEVICE_MODEL),
        ]).await
    }

    /// Deactivate this device (`ClaveDesactivateAuthSv`).
    pub async fn deactivate_authentication(
        &self,
        device_id: &str,
        nif: &str,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveDesactivateAuthSv");
        self.post_form("deactivate_authentication", &url, &[
            ("device_id", device_id),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
        ]).await
    }

    /// Authenticate with DNI/NIE + date of birth (weak auth).
    ///
    /// Like the app, the final body is parsed as the JSON envelope: KO `-8` /
    /// `-10` map to [`LlaveError::DniBirthDateRequired`] (resend with the
    /// date of birth and the returned [`DniChallenge`]); other KO codes are
    /// API errors.  A non-JSON body falls back to the redirect heuristic.
    ///
    /// We follow redirects manually so we can capture `Set-Cookie` headers
    /// from every hop (reqwest's automatic redirect only exposes cookies
    /// from the final response).
    pub async fn authenticate_dni_nie(
        &self,
        nif: &str,
        fecha: &str,
        soporte: &str,
        challenge: Option<DniChallenge>,
    ) -> Result<()> {
        let (azul, fecha_nie) = DniChallenge::form_flags(challenge);
        let url = format!(
            "{BASE_URL}/wlpl/BUCV-JDIT/AutenticaDniNieContrasteh?ref=%2Fwlpl%2FMOVI-AEAT%2FAccesoW12Sv"
        );
        tracing::debug!(endpoint = "authenticate_dni_nie", "aeat request");

        // The main client already has redirect(Policy::none()), so we
        // can use it directly and follow redirects manually.
        let cookie_header = self.cookie_header();
        let mut req = self.client
            .post(&url)
            .form(&[
                ("NIF", nif),
                ("FECHA", fecha),
                ("SOPORTE", soporte),
                ("botonAutenticacionDebil", "Continuar"),
                ("APP", "S"),
                ("modo", "json"),
                ("AZUL", azul),
                ("FECHANIE", fecha_nie),
            ]);
        if !cookie_header.is_empty() {
            req = req.header(reqwest::header::COOKIE, &cookie_header);
        }

        let resp = req.send().await?;
        tracing::debug!(
            endpoint = "authenticate_dni_nie",
            http_status = %resp.status(),
            "initial response"
        );
        self.capture_cookies(&resp);

        let (resp, redirect_count) = self.follow_redirects("authenticate_dni_nie", &cookie_header, resp).await?;
        let final_status = resp.status();
        let body = resp.text().await?;

        if let Some(outcome) = classify_dni_body(&body) {
            tracing::info!(
                endpoint = "authenticate_dni_nie",
                ok = outcome.is_ok(),
                redirect_count = redirect_count,
                "DNI/NIE auth JSON response"
            );
            return outcome;
        }

        // Non-JSON body: a successful DNI/NIE auth produces at least one redirect.
        // If we got 0 redirects, the server returned the login form again,
        // which means the credentials were rejected.
        if redirect_count == 0 {
            tracing::warn!(
                endpoint = "authenticate_dni_nie",
                http_status = %final_status,
                "no redirects — DNI/NIE auth likely failed (bad credentials)"
            );
            return Err(LlaveError::DniAuthFailed);
        }

        tracing::debug!(
            endpoint = "authenticate_dni_nie",
            final_status = %final_status,
            redirect_count = redirect_count,
            cookies = %self.cookie_header(),
            "DNI/NIE auth complete (cookies captured)"
        );
        Ok(())
    }

    /// Check registration state after DNI/NIE authentication.
    ///
    /// This is the intermediate step between DNI auth and device activation.
    /// Only sends device metadata (no device_id or NIF) — the server uses
    /// the session cookie established by `authenticate_dni_nie`.
    pub async fn clave_request_state(&self) -> Result<ApiResponse<ClaveRequestStateResponse>> {
        let url = format!("{BASE_URL_WWW12}/wlpl/MOVI-P24H/ClaveRequestStateSv");
        self.post_form("clave_request_state", &url, &[
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
        ]).await
    }

    /// Get pending operations for polling.
    pub async fn request_all_operations(
        &self,
        device_id: &str,
        nif: &str,
        timestamp: &str,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveRequestAllOperationsSv");
        self.post_form("request_all_operations", &url, &[
            ("device_id", device_id),
            ("NIF", nif),
            ("timestamp", timestamp),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
        ]).await
    }

    /// Query operations history.
    ///
    /// Mirrors the original app's `ClaveOperationsHistorySv` endpoint.
    /// The `respuesta` field is a JSON array of [`OperationEntry`] objects.
    ///
    /// Default filter values match the original app:
    /// - `aut_fir`: `"0"`
    /// - `resultado`: `"00"` (both confirmed and rejected)
    /// - `tipo`: `"111"` (all types: DNI/NIE + QR + SMS)
    /// - `organismo`: `""` (all organisations)
    /// - `orden`: `"0"` (default sort, most recent first)
    pub async fn operations_history(
        &self,
        device_id: &str,
        device_password: &str,
        nif: &str,
        fecha_desde: &str,
        fecha_hasta: &str,
        resultado: &str,
    ) -> Result<ApiResponse<Vec<OperationEntry>>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveOperationsHistorySv");
        // `aut_fir`, `orden` and `tipo` are what the app sends (sorting is
        // done on the device; `tipo` "111" = SMS, QR and DNI requests).
        self.post_form("operations_history", &url, &[
            ("device_id", device_id),
            ("user_password", device_password),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
            ("aut_fir", "0"),
            ("resultado", resultado),
            ("organismo", ""),
            ("fecha_desde", fecha_desde),
            ("fecha_hasta", fecha_hasta),
            ("tipo", "111"),
            ("orden", "0"),
        ]).await
    }

    /// Validate Llave Móvil token.
    pub async fn validate_llave_movil(
        &self,
        token: &str,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let url = format!("{BASE_URL_WWW12}/wlpl/MOVI-P24H/ValidarClaveMovil");
        self.post_form("validate_llave_movil", &url, &[
            ("tokenClaveMovil", token),
        ]).await
    }

    /// Request SMS verification code for device activation.
    ///
    /// Calls ObtenerClaveMovilSMS on www12. Returns `timeStampAltaSms`,
    /// `tokenClaveMovilSms`, `horaPeticion`, and the masked `movil` number.
    ///
    /// The Android app sends a bare POST (@POST without @FormUrlEncoded).
    pub async fn request_sms_code(&self) -> Result<ApiResponse<ObtenerSmsResponse>> {
        let url = format!("{BASE_URL_WWW12}/wlpl/MOVI-P24H/ObtenerClaveMovilSMS");
        self.post_empty("request_sms_code", &url).await
    }

    /// Validate SMS verification code.
    pub async fn validate_sms_code(
        &self,
        timestamp: &str,
        token: &str,
        pin: &str,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let url = format!("{BASE_URL_WWW12}/wlpl/MOVI-P24H/ValidarClaveMovilSMS");
        self.post_form("validate_sms_code", &url, &[
            ("timeStampAltaSms", timestamp),
            ("tokenClaveMovilSms", token),
            ("pinAcceso", pin),
        ]).await
    }

    /// QR code authentication.
    pub async fn qr_authenticate(
        &self,
        device_id: &str,
        nif: &str,
        qr_value: &str,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveMovilQrSv");
        self.post_form("qr_authenticate", &url, &[
            ("device_id", device_id),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
            ("valor_qr", qr_value),
        ]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- store_cookie --

    #[test]
    fn store_cookie_adds_new_cookie() {
        let mut cookies = jar(&[]);
        store_cookie(&mut cookies, "appmovil", "a");
        assert_eq!(cookies, jar(&[("appmovil", "a")]));
    }

    #[test]
    fn store_cookie_updates_existing_cookie() {
        let mut cookies = jar(&[("appmovil", "a")]);
        store_cookie(&mut cookies, "appmovil", "b");
        assert_eq!(cookies, jar(&[("appmovil", "b")]));
    }

    #[test]
    fn store_cookie_empty_value_deletes_cookie() {
        let mut cookies = jar(&[("pin24H", "x"), ("appmovil", "a")]);
        store_cookie(&mut cookies, "pin24H", "");
        assert_eq!(cookies, jar(&[("appmovil", "a")]));
    }

    #[test]
    fn store_cookie_quoted_empty_value_deletes_cookie() {
        let mut cookies = jar(&[("pin24H", "x")]);
        store_cookie(&mut cookies, "pin24H", "\"\"");
        assert!(cookies.is_empty());
    }

    // -- retain_persistent_cookies --

    #[test]
    fn session_cookies_cleared_keeping_app_cookies() {
        let mut cookies = jar(&[
            ("appmovil", "a"),
            ("WWW12", "w"),
            ("sgat-language", "es_ES"),
            ("pin24H", "p"),
        ]);
        retain_persistent_cookies(&mut cookies);
        assert_eq!(cookies, jar(&[("appmovil", "a"), ("sgat-language", "es_ES")]));
    }

    // -- redirects --

    #[test]
    fn redirect_allowed_up_to_okhttp_limit() {
        assert!(redirect_allowed(19));
    }

    #[test]
    fn redirect_refused_after_okhttp_limit() {
        assert!(!redirect_allowed(20));
    }

    // -- ApiResponse::into_result --

    #[test]
    fn into_result_maps_500600_to_outdated_app() {
        let resp: ApiResponse<serde_json::Value> = serde_json::from_str(
            r#"{"status":"KO","codigo_error":"500600","mensaje":"Actualice"}"#,
        )
        .unwrap();
        assert!(matches!(resp.into_result(), Err(LlaveError::AppVersionOutdated { .. })));
    }

    // -- classify_dni_body --

    #[test]
    fn dni_body_html_is_unclassified() {
        assert!(classify_dni_body("<!DOCTYPE html><html></html>").is_none());
    }

    #[test]
    fn dni_body_ok_json_succeeds() {
        let body = r#"{"status":"OK","codigo_error":"","mensaje":""}"#;
        assert!(matches!(classify_dni_body(body), Some(Ok(()))));
    }

    #[test]
    fn dni_body_minus_8_requires_birth_date_azul() {
        let body = r#"{"status":"KO","codigo_error":"-8","mensaje":"Introduzca fecha de nacimiento"}"#;
        assert!(matches!(
            classify_dni_body(body),
            Some(Err(LlaveError::DniBirthDateRequired { challenge: DniChallenge::Azul, .. }))
        ));
    }

    #[test]
    fn dni_body_minus_10_requires_birth_date_nie() {
        let body = r#"{"status":"KO","codigo_error":"-10","mensaje":"x"}"#;
        assert!(matches!(
            classify_dni_body(body),
            Some(Err(LlaveError::DniBirthDateRequired { challenge: DniChallenge::FechaNie, .. }))
        ));
    }

    #[test]
    fn dni_body_keeps_server_message() {
        let body = r#"{"status":"KO","codigo_error":"-8","mensaje":"Introduzca fecha"}"#;
        match classify_dni_body(body) {
            Some(Err(LlaveError::DniBirthDateRequired { message, .. })) => {
                assert_eq!(message, "Introduzca fecha");
            }
            _ => panic!("expected DniBirthDateRequired"),
        }
    }

    #[test]
    fn dni_body_other_ko_is_api_error() {
        let body = r#"{"status":"KO","codigo_error":"-1","mensaje":"Datos incorrectos"}"#;
        assert!(matches!(classify_dni_body(body), Some(Err(LlaveError::Api { .. }))));
    }

    // -- DniChallenge --

    #[test]
    fn challenge_none_sends_empty_flags() {
        assert_eq!(DniChallenge::form_flags(None), ("", ""));
    }

    #[test]
    fn challenge_azul_sets_azul_flag() {
        assert_eq!(DniChallenge::form_flags(Some(DniChallenge::Azul)), ("X", ""));
    }

    #[test]
    fn challenge_fecha_nie_sets_fechanie_flag() {
        assert_eq!(DniChallenge::form_flags(Some(DniChallenge::FechaNie)), ("", "X"));
    }

    // -- client identity --

    #[test]
    fn user_agent_version_matches_app_version() {
        assert!(USER_AGENT.starts_with(&format!("APPMovil/Cl@ve/v{APP_VERSION}(")));
    }

    #[test]
    fn user_agent_os_version_matches_os_version() {
        assert!(USER_AGENT.contains(&format!(")/{OS_VERSION}/Android/")));
    }

    // -- build_cookie_header --

    fn jar(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn cookie_header_joins_cookies() {
        let cookies = jar(&[("appmovil", "a"), ("WWW12", "b")]);
        assert_eq!(build_cookie_header(&cookies), "appmovil=a; WWW12=b");
    }

    #[test]
    fn cookie_header_never_sends_jsessionid() {
        let cookies = jar(&[("JSESSIONID", "x"), ("appmovil", "a")]);
        assert_eq!(build_cookie_header(&cookies), "appmovil=a");
    }

    #[test]
    fn cookie_header_skips_empty_values() {
        let cookies = jar(&[("pin24H", ""), ("appmovil", "a")]);
        assert_eq!(build_cookie_header(&cookies), "appmovil=a");
    }

    #[test]
    fn cookie_header_empty_jar_is_empty() {
        assert_eq!(build_cookie_header(&[]), "");
    }

    // -- truncate_str --

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn truncate_exact_length() {
        assert_eq!(truncate_str("hello", 5), "hello");
    }

    #[test]
    fn truncate_cuts() {
        assert_eq!(truncate_str("hello world", 5), "hello");
    }

    #[test]
    fn truncate_empty() {
        assert_eq!(truncate_str("", 5), "");
    }

    #[test]
    fn truncate_zero_max() {
        assert_eq!(truncate_str("hello", 0), "");
    }

    #[test]
    fn truncate_multibyte_boundary() {
        // 'é' is 2 bytes in UTF-8: c(1) a(1) f(1) é(2) = 5 bytes
        // max_bytes=4 falls inside 'é', should back up to 3
        assert_eq!(truncate_str("café", 4), "caf");
    }

    #[test]
    fn truncate_emoji_boundary() {
        // '🦀' is 4 bytes: a(1) 🦀(4) b(1) = 6 bytes
        // max_bytes=3 falls inside the emoji, should back up to 1
        assert_eq!(truncate_str("a🦀b", 3), "a");
    }

    // -- ApiResponse::into_result --

    #[test]
    fn api_response_ok_with_data() {
        let resp = ApiResponse {
            status: "OK".into(),
            visible: None,
            crashlytics: None,
            codigo_error: None,
            mensaje: None,
            respuesta: Some("data".to_string()),
        };
        assert_eq!(resp.into_result().unwrap(), "data");
    }

    #[test]
    fn api_response_ok_without_data() {
        let resp: ApiResponse<String> = ApiResponse {
            status: "OK".into(),
            visible: None,
            crashlytics: None,
            codigo_error: None,
            mensaje: None,
            respuesta: None,
        };
        let err = resp.into_result().unwrap_err();
        assert!(err.to_string().contains("Empty response payload"));
    }

    #[test]
    fn api_response_ko_generic() {
        let resp: ApiResponse<String> = ApiResponse {
            status: "KO".into(),
            visible: None,
            crashlytics: None,
            codigo_error: Some("999".into()),
            mensaje: Some("something failed".into()),
            respuesta: None,
        };
        let err = resp.into_result().unwrap_err();
        match err {
            LlaveError::Api { status, code, message } => {
                assert_eq!(status, "KO");
                assert_eq!(code, "999");
                assert_eq!(message, "something failed");
            }
            other => panic!("expected Api error, got: {other:?}"),
        }
    }

    #[test]
    fn api_response_ko_rate_limited() {
        let resp: ApiResponse<String> = ApiResponse {
            status: "KO".into(),
            visible: None,
            crashlytics: None,
            codigo_error: Some("16600".into()),
            mensaje: Some("rate limited".into()),
            respuesta: None,
        };
        let err = resp.into_result().unwrap_err();
        assert!(matches!(err, LlaveError::IpRateLimited { .. }));
    }

    #[test]
    fn api_response_ko_defaults() {
        let resp: ApiResponse<String> = ApiResponse {
            status: "KO".into(),
            visible: None,
            crashlytics: None,
            codigo_error: None,
            mensaje: None,
            respuesta: None,
        };
        let err = resp.into_result().unwrap_err();
        match err {
            LlaveError::Api { code, message, .. } => {
                assert_eq!(code, "");
                assert_eq!(message, "Unknown error");
            }
            other => panic!("expected Api error, got: {other:?}"),
        }
    }

    // -- ApiResponse serde deserialization --

    #[test]
    fn api_response_deserialize_ok() {
        let json = r#"{"status":"OK","respuesta":{"key":"value"}}"#;
        let resp: ApiResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "OK");
        assert_eq!(resp.respuesta.unwrap()["key"], "value");
    }

    #[test]
    fn api_response_deserialize_ko() {
        let json = r#"{"status":"KO","codigo_error":"123","mensaje":"bad request"}"#;
        let resp: ApiResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.status, "KO");
        assert_eq!(resp.codigo_error.as_deref(), Some("123"));
        assert_eq!(resp.mensaje.as_deref(), Some("bad request"));
        assert!(resp.respuesta.is_none());
    }
}
