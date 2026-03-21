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

const APP_VERSION: &str = "6.2.5";
/// Android-style sistema_operativo value ("A" = Android).
/// The AEAT server uses this to determine session routing.
const OS_NAME: &str = "A";
const OS_VERSION: &str = "14";
const DEVICE_MODEL: &str = "SM-S928U";
/// User-Agent matching the Android app format so the AEAT server
/// recognises us as a mobile client (critical for session handling).
/// Format: APPMovil/Cl@ve/v{version}({build})/{os_ver}/Android/{System.getProperty("http.agent")}
/// The full Dalvik user-agent suffix is required — the truncated version
/// (without the parenthetical) may be rejected by the server/WAF.
const USER_AGENT: &str = "APPMovil/Cl@ve/v6.2.5(288)/14/Android/Dalvik/2.1.0 (Linux; U; Android 14; SM-S928U Build/UP1A.231005.007)";

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
            Err(LlaveError::Api {
                status: self.status,
                code,
                message,
            })
        }
    }
}

// --- Response types ---

#[derive(Debug, Deserialize, Serialize)]
pub struct StartingResponse {
    #[serde(rename = "listaBlancaUrls")]
    pub lista_blanca_urls: Option<Vec<String>>,
    #[serde(rename = "listaNegraUrls")]
    pub lista_negra_urls: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IsNifActivatedResponse {
    #[serde(rename = "activado")]
    pub activated: Option<String>,
    #[serde(rename = "nivelAcceso")]
    pub access_level: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestPinResponse {
    pub pin: Option<String>,
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
    pub device_id: Option<String>,
    pub user_password: Option<String>,
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
/// Manages cookies manually across all `*.agenciatributaria.gob.es` subdomains,
/// matching the Android app's `CookiePolicy.ACCEPT_ALL` + `setCookiesInJar()`.
/// Reqwest's built-in cookie store follows RFC domain-matching rules which
/// prevents cookies set by `www2` from being sent to `www6` or `www12`.
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
        let mut builder = Client::builder()
            .user_agent(USER_AGENT)
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

    /// Build the `Cookie` header value from all stored cookies.
    fn cookie_header(&self) -> String {
        self.cookie_header_filtered(&[])
    }

    /// Build cookie header, excluding cookies whose names are in `exclude`.
    fn cookie_header_filtered(&self, exclude: &[&str]) -> String {
        let cookies = self.cookies.lock();
        cookies
            .iter()
            .filter(|(k, _)| !exclude.contains(&k.as_str()))
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ")
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
            let name = cookie.name().to_string();
            let value = cookie.value().to_string();
            // Update existing cookie or add new one.
            if let Some(existing) = cookies.iter_mut().find(|(k, _)| k == &name) {
                existing.1 = value;
            } else {
                cookies.push((name, value));
            }
        }
    }

    /// Build the TrazasApp header value matching the Android app's `getCookiesInApp()`.
    ///
    /// The Android app sends a JSON object containing cookie diagnostic info.
    /// We replicate this so the server recognises us as a legitimate client.
    fn trazas_app_header(&self) -> String {
        let cookies = self.cookies.lock();
        let get = |name: &str| -> String {
            cookies
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "null".into())
        };
        let www12 = get("WWW12");
        let www12v = get("WWW12V");
        let pin24h = get("pin24H");
        let pin24v = get("pin24V");
        let cert1 = get("CERT_WWW1");
        let cert1v = get("CERT_WWW1V");
        let appmovil = get("appmovil");
        let sgat_lang = get("sgat-language");

        // Field names must match the Kotlin @Serializable CookiesInApp class:
        //   cookiesWww1Gestor  = CERT_WWW1 " y " CERT_WWW1V  (from CookieJar)
        //   cookiesWww6Gestor  = pin24H    " y " pin24V      (from CookieJar)
        //   cookiesWww12Gestor = WWW12     " y " WWW12V      (from CookieJar)
        //   cookiesAppMovilGestor / cookiesSgatLanguageGestor
        //   *Local variants are the same values from keychain storage.
        serde_json::to_string(&serde_json::json!({
            "cookiesWww1Gestor": format!("{cert1} y {cert1v}"),
            "cookiesWww6Gestor": format!("{pin24h} y {pin24v}"),
            "cookiesWww12Gestor": format!("{www12} y {www12v}"),
            "cookiesAppMovilGestor": appmovil,
            "cookiesSgatLanguageGestor": sgat_lang,
            "cookiesWww1Local": format!("{cert1} y {cert1v}"),
            "cookiesWww6Local": format!("{pin24h} y {pin24v}"),
            "cookiesWww12Local": format!("{www12} y {www12v}"),
            "cookiesAppMovilLocal": appmovil,
            "cookiesSgatLanguageLocal": sgat_lang,
        }))
        .unwrap_or_else(|_| "{}".into())
    }

    /// Maximum number of redirects to follow before returning an error.
    const MAX_REDIRECTS: u32 = 10;

    /// Follow HTTP redirects, re-attaching cookies and TrazasApp at each hop.
    ///
    /// Returns the final (non-redirect) response and the number of redirects
    /// followed. Caps at [`Self::MAX_REDIRECTS`] to prevent infinite loops.
    async fn follow_redirects(
        &self,
        endpoint: &str,
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

            let next_url = if location.starts_with("http") {
                location.clone()
            } else {
                let base = resp.url().origin().unicode_serialization();
                format!("{base}{location}")
            };

            tracing::debug!(endpoint = endpoint, redirect_to = %next_url, "following redirect");

            let cookie_header = self.cookie_header();
            let mut next = self
                .client
                .get(&next_url)
                .header("TrazasApp", self.trazas_app_header());
            if !cookie_header.is_empty() {
                next = next.header(reqwest::header::COOKIE, &cookie_header);
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

            if count >= Self::MAX_REDIRECTS {
                return Err(LlaveError::TooManyRedirects {
                    endpoint: endpoint.to_string(),
                    count,
                });
            }
        }
        Ok((resp, count))
    }

    async fn post_form<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        url: &str,
        form: &[(&str, &str)],
    ) -> Result<ApiResponse<T>> {
        let cookie_header = self.cookie_header();
        tracing::debug!(endpoint = endpoint, url = url, cookies = %cookie_header, "aeat request");
        let mut req = self
            .client
            .post(url)
            .header("TrazasApp", self.trazas_app_header())
            .form(form);
        if !cookie_header.is_empty() {
            req = req.header(reqwest::header::COOKIE, &cookie_header);
        }

        let resp = req.send().await?;
        tracing::debug!(endpoint = endpoint, http_status = %resp.status(), "aeat response");
        self.capture_cookies(&resp);

        let (resp, _) = self.follow_redirects(endpoint, resp).await?;

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

    /// POST with no body (matching Retrofit `@POST` without `@FormUrlEncoded`).
    ///
    /// Some AEAT endpoints (like ObtenerClaveMovilSMS) expect a bare POST
    /// with no Content-Type or body. Sending form-encoded data causes 902024.
    async fn post_empty<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        url: &str,
    ) -> Result<ApiResponse<T>> {
        let cookie_header = self.cookie_header();
        let trazas = self.trazas_app_header();
        tracing::info!(endpoint = endpoint, url = url, cookies = %cookie_header, trazas_app = %trazas, "aeat request (empty POST)");
        let mut req = self
            .client
            .post(url)
            .header("TrazasApp", &trazas)
            .body(""); // Empty body → Content-Length: 0, no Content-Type
        if !cookie_header.is_empty() {
            req = req.header(reqwest::header::COOKIE, &cookie_header);
        }

        let resp = req.send().await?;
        tracing::debug!(endpoint = endpoint, http_status = %resp.status(), "aeat response");
        self.capture_cookies(&resp);

        let (resp, _) = self.follow_redirects(endpoint, resp).await?;

        let final_status = resp.status();
        let final_url = resp.url().to_string();
        let body = resp.text().await?;
        tracing::info!(endpoint = endpoint, http_status = %final_status, final_url = %final_url, body_len = body.len(), body_preview = %truncate_str(&body, 512), "aeat response body (empty POST)");

        let trimmed = body.trim_start();
        if trimmed.starts_with("<!DOCTYPE") || trimmed.starts_with("<html") {
            return Err(LlaveError::HtmlResponse {
                endpoint: endpoint.to_string(),
            });
        }

        serde_json::from_str::<ApiResponse<T>>(&body).map_err(|e| {
            tracing::error!(endpoint = endpoint, err = %e, body_preview = %truncate_str(&body, 1024), "failed to decode response");
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

    /// Check if a NIF is activated in Clave (www2 endpoint).
    pub async fn clave_is_nif_activated(
        &self,
        device_id: &str,
        nif: &str,
    ) -> Result<ApiResponse<IsNifActivatedResponse>> {
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

    /// Request a Cl@ve PIN.
    ///
    /// Uses `ClaveRequestPinSv` (the current endpoint). The older
    /// `LlaveRequestPinSv` returns 404 on public internet.
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

    /// Confirm a Cl@ve Móvil authentication request.
    ///
    /// Uses `ClaveAuthenticateSv` (the current endpoint). The older
    /// `LlaveAuthenticateSv` returns 404 on public internet.
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

    /// Cancel a pending Cl@ve Móvil authentication.
    ///
    /// Uses `ClaveCancelAuthenticateSv`. The older `LlaveCancelAuthenticateSv`
    /// returns 404 on public internet.
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
    /// The endpoint only exists on www6. We send WWW12/WWW12V auth tokens
    /// but strip the www2 JSESSIONID — sending a JSESSIONID from a different
    /// WebSphere cluster confuses www6 into returning the homepage.
    pub async fn activate_authentication(
        &self,
        device_password: &str,
        token_push: &str,
    ) -> Result<ApiResponse<ActivateResponse>> {
        let url = format!("{BASE_URL_WWW6}/wlpl/MOVI-P24H/ClaveActivateAuthenticationSv");

        // Strip JSESSIONID — it's from www2's WebSphere cluster and
        // confuses www6. Send only the cross-domain auth tokens (WWW12 etc).
        let cookie_header = self.cookie_header_filtered(&["JSESSIONID"]);
        tracing::info!(cookies = %cookie_header, "activation request to www6 (no JSESSIONID)");

        let mut req = self
            .client
            .post(&url)
            .header("TrazasApp", self.trazas_app_header())
            .form(&[
                ("sistema_operativo", OS_NAME),
                ("version_os", OS_VERSION),
                ("version_app", APP_VERSION),
                ("token_push", token_push),
                ("user_password", device_password),
                ("modelo", DEVICE_MODEL),
            ]);
        if !cookie_header.is_empty() {
            req = req.header(reqwest::header::COOKIE, &cookie_header);
        }

        let resp = req.send().await?;
        tracing::info!(http_status = %resp.status(), "www6 activation response");
        self.capture_cookies(&resp);

        let (resp, _) = self.follow_redirects("activate_authentication", resp).await?;

        let final_status = resp.status();
        let final_url = resp.url().to_string();
        let body = resp.text().await?;
        tracing::info!(
            http_status = %final_status,
            final_url = %final_url,
            body_len = body.len(),
            body_preview = %truncate_str(&body, 300),
            "www6 activation response body"
        );

        let trimmed = body.trim_start();
        if trimmed.starts_with("<!DOCTYPE") || trimmed.starts_with("<html") {
            return Err(LlaveError::HtmlResponse {
                endpoint: "activate_authentication".to_string(),
            });
        }

        serde_json::from_str::<ApiResponse<ActivateResponse>>(&body).map_err(|e| {
            tracing::error!(err = %e, body_preview = %truncate_str(&body, 1024), "failed to decode activation response");
            LlaveError::Json(e)
        })
    }

    /// Deactivate device authentication.
    ///
    /// Uses `ClaveDesactivateAuthSv`. The older `LlaveDesactivateAuthSv`
    /// returns 404 on public internet.
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
    /// This endpoint **always returns HTML** (not JSON).  The Android app
    /// ignores the response body entirely — the only thing that matters are
    /// the session cookies set during the HTTP redirect chain.
    ///
    /// We follow redirects manually so we can capture `Set-Cookie` headers
    /// from every hop (reqwest's automatic redirect only exposes cookies
    /// from the final response).
    pub async fn authenticate_dni_nie(
        &self,
        nif: &str,
        fecha: &str,
        soporte: &str,
    ) -> Result<()> {
        let url = format!(
            "{BASE_URL}/wlpl/BUCV-JDIT/AutenticaDniNieContrasteh?ref=%2Fwlpl%2FMOVI-AEAT%2FAccesoW12Sv"
        );
        tracing::debug!(endpoint = "authenticate_dni_nie", "aeat request");

        // The main client already has redirect(Policy::none()), so we
        // can use it directly and follow redirects manually.
        let cookie_header = self.cookie_header();
        let mut req = self.client
            .post(&url)
            .header("TrazasApp", self.trazas_app_header())
            .form(&[
                ("NIF", nif),
                ("FECHA", fecha),
                ("SOPORTE", soporte),
                ("botonAutenticacionDebil", "Continuar"),
                ("APP", "S"),
                ("modo", "json"),
                ("AZUL", ""),
                ("FECHANIE", ""),
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

        let (resp, redirect_count) = self.follow_redirects("authenticate_dni_nie", resp).await?;

        // A successful DNI/NIE auth produces at least one redirect.
        // If we got 0 redirects, the server returned the login form again,
        // which means the credentials were rejected.
        if redirect_count == 0 {
            tracing::warn!(
                endpoint = "authenticate_dni_nie",
                http_status = %resp.status(),
                "no redirects — DNI/NIE auth likely failed (bad credentials)"
            );
            return Err(LlaveError::DniAuthFailed);
        }

        tracing::debug!(
            endpoint = "authenticate_dni_nie",
            final_status = %resp.status(),
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
    ) -> Result<ApiResponse<Vec<OperationEntry>>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveOperationsHistorySv");
        self.post_form("operations_history", &url, &[
            ("device_id", device_id),
            ("user_password", device_password),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
            ("aut_fir", "0"),
            ("resultado", "00"),
            ("organismo", ""),
            ("fecha_desde", fecha_desde),
            ("fecha_hasta", fecha_hasta),
            ("tipo", "111"),
            ("orden", "0"),
        ]).await
    }

    /// Get Llave Móvil activation page.
    pub async fn get_llave_movil(&self) -> Result<ApiResponse<serde_json::Value>> {
        let url = format!("{BASE_URL_WWW12}/wlpl/MOVI-P24H/ObtenerClaveMovil");
        tracing::debug!(endpoint = "get_llave_movil", "aeat request");
        let cookie_header = self.cookie_header();
        let mut req = self.client.get(&url);
        if !cookie_header.is_empty() {
            req = req.header(reqwest::header::COOKIE, &cookie_header);
        }
        let resp = req.send().await?;
        let status = resp.status();
        tracing::debug!(endpoint = "get_llave_movil", http_status = %status, "aeat response");
        self.capture_cookies(&resp);
        Ok(resp.json().await?)
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

    /// Register a push notification token with the server.
    pub async fn set_firebase_token(
        &self,
        device_id: &str,
        nif: &str,
        token_push: &str,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ClaveSetFirebaseTokenSv");
        self.post_form("set_firebase_token", &url, &[
            ("device_id", device_id),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
            ("token_push", token_push),
        ]).await
    }

    /// Check current request state (pending authentication requests).
    pub async fn request_state(
        &self,
        device_id: &str,
        device_password: &str,
        nif: &str,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let url = format!("{BASE_URL_WWW12}/wlpl/MOVI-P24H/ClaveRequestStateSv");
        self.post_form("request_state", &url, &[
            ("device_id", device_id),
            ("user_password", device_password),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
        ]).await
    }

    /// Get pending market petitions.
    pub async fn get_pending_petitions(
        &self,
        device_id: &str,
        nif: &str,
    ) -> Result<ApiResponse<serde_json::Value>> {
        let url = format!("{BASE_URL}/wlpl/MOVI-P24H/ObtenerPeticionesMarketsSv");
        self.post_form("get_pending_petitions", &url, &[
            ("device_id", device_id),
            ("NIF", nif),
            ("sistema_operativo", OS_NAME),
            ("version_os", OS_VERSION),
            ("version_app", APP_VERSION),
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
