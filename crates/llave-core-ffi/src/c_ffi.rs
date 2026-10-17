//! Manual C FFI layer for Flutter dart:ffi, bypassing flutter_rust_bridge codegen.
//!
//! Every function returns a `*mut c_char` (JSON string) that the caller **must**
//! free with [`llave_free_string`].  The JSON envelope is:
//!   `{"ok": true, "data": <value>}` on success
//!   `{"ok": false, "error": "message"}` on failure
//!
//! # Safety (all pointer arguments)
//!
//! Every `*const c_char` argument must be either null or a valid pointer to a
//! NUL-terminated string that stays alive and unmodified for the duration of
//! the call.  Null is treated as "absent" (`None`) or an empty string.
//!
//! Panics never cross the FFI boundary: each export runs inside
//! [`ffi_guard`], which turns a panic into an `{"ok": false}` envelope.

use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn rt() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().expect("tokio runtime"))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read an optional C string argument.
///
/// # Safety
///
/// `ptr` must be null or point to a valid NUL-terminated string that outlives
/// this call (see the module-level contract).
unsafe fn to_opt(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: non-null and, per the caller contract, NUL-terminated and
        // alive for the duration of the call.
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

/// Read a C string argument, treating null as an empty string.
///
/// # Safety
///
/// Same contract as [`to_opt`].
unsafe fn to_str(ptr: *const c_char) -> String {
    // SAFETY: forwarded caller contract.
    unsafe { to_opt(ptr) }.unwrap_or_default()
}

fn c(s: String) -> *mut c_char {
    CString::new(s.replace('\0', "?"))
        .expect("replacement removed all NUL bytes")
        .into_raw()
}

fn ok_json(data: serde_json::Value) -> *mut c_char {
    c(serde_json::json!({"ok": true, "data": data}).to_string())
}

fn err_json(msg: &str) -> *mut c_char {
    c(serde_json::json!({"ok": false, "error": msg}).to_string())
}

/// Run an export body, converting a panic into an error envelope so that
/// unwinding never crosses into Dart.
fn ffi_guard(body: impl FnOnce() -> *mut c_char) -> *mut c_char {
    catch_unwind(AssertUnwindSafe(body)).unwrap_or_else(|_| err_json("internal error (panic)"))
}

fn api_result_json(r: Result<crate::api::FfiApiResult, String>) -> *mut c_char {
    match r {
        Ok(r) => ok_json(serde_json::json!({
            "ok": r.ok,
            "data": r.data,
            "error": r.error,
            "error_code": r.error_code,
            "server_message": r.server_message,
        })),
        Err(e) => err_json(&e),
    }
}

// ---------------------------------------------------------------------------
// Free
// ---------------------------------------------------------------------------

/// Free a string returned by any `llave_*` function.
///
/// # Safety
///
/// `ptr` must be null or a pointer previously returned by a `llave_*`
/// function and not yet freed.  It must not be used after this call.
#[no_mangle]
pub unsafe extern "C" fn llave_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        // SAFETY: per the contract, `ptr` came from `CString::into_raw` in
        // `c()` and ownership is transferred back exactly once.
        drop(unsafe { CString::from_raw(ptr) });
    }
}

// ---------------------------------------------------------------------------
// init / session management
// ---------------------------------------------------------------------------

/// # Safety
///
/// `session_json` must satisfy the module-level pointer contract.
#[no_mangle]
pub unsafe extern "C" fn llave_init_core(session_json: *const c_char) -> *mut c_char {
    // SAFETY: caller contract.
    let session_json = unsafe { to_opt(session_json) };
    ffi_guard(|| match crate::api::init_core(session_json) {
        Ok(_) => ok_json(serde_json::json!(true)),
        Err(e) => err_json(&e),
    })
}

#[no_mangle]
pub extern "C" fn llave_export_session() -> *mut c_char {
    ffi_guard(|| match crate::api::export_session() {
        Some(json) => ok_json(serde_json::Value::String(json)),
        None => ok_json(serde_json::Value::Null),
    })
}

/// llave's version and the Cl@ve app version it matches.
#[no_mangle]
pub extern "C" fn llave_version() -> *mut c_char {
    ffi_guard(|| ok_json(serde_json::Value::String(crate::api::version())))
}

#[no_mangle]
pub extern "C" fn llave_get_status() -> *mut c_char {
    ffi_guard(|| {
        let s = crate::api::get_status();
        ok_json(serde_json::json!({
            "active": s.active,
            "nif": s.nif,
            "deviceId": s.device_id,
            "createdAt": s.created_at,
        }))
    })
}

#[no_mangle]
pub extern "C" fn llave_logout() -> *mut c_char {
    ffi_guard(|| match crate::api::logout() {
        Ok(_) => ok_json(serde_json::json!(true)),
        Err(e) => err_json(&e),
    })
}

/// # Safety
///
/// `nif` must satisfy the module-level pointer contract.
#[no_mangle]
pub unsafe extern "C" fn llave_validate_nif(nif: *const c_char) -> *mut c_char {
    // SAFETY: caller contract.
    let nif = unsafe { to_str(nif) };
    ffi_guard(|| match crate::api::validate_nif(nif) {
        Ok(v) => ok_json(serde_json::Value::String(v)),
        Err(e) => err_json(&e),
    })
}

/// # Safety
///
/// `nif` must satisfy the module-level pointer contract.
#[no_mangle]
pub unsafe extern "C" fn llave_dni_input(nif: *const c_char) -> *mut c_char {
    // SAFETY: caller contract.
    let nif = unsafe { to_str(nif) };
    ffi_guard(|| match crate::api::dni_input(nif) {
        Ok(v) => ok_json(serde_json::Value::String(v)),
        Err(e) => err_json(&e),
    })
}

// ---------------------------------------------------------------------------
// PIN-based session encryption
// ---------------------------------------------------------------------------

/// # Safety
///
/// `pin` must satisfy the module-level pointer contract.
#[no_mangle]
pub unsafe extern "C" fn llave_encrypt_session(pin: *const c_char) -> *mut c_char {
    // SAFETY: caller contract.
    let pin = unsafe { to_str(pin) };
    ffi_guard(|| match crate::api::encrypt_session(pin) {
        Ok(v) => ok_json(serde_json::Value::String(v)),
        Err(e) => err_json(&e),
    })
}

/// # Safety
///
/// `sealed` and `pin` must satisfy the module-level pointer contract.
#[no_mangle]
pub unsafe extern "C" fn llave_decrypt_and_load_session(
    sealed: *const c_char,
    pin: *const c_char,
) -> *mut c_char {
    // SAFETY: caller contract.
    let (sealed, pin) = unsafe { (to_str(sealed), to_str(pin)) };
    ffi_guard(|| match crate::api::decrypt_and_load_session(sealed, pin) {
        Ok(_) => ok_json(serde_json::json!(true)),
        Err(e) => err_json(&e),
    })
}

/// # Safety
///
/// All pointer arguments must satisfy the module-level pointer contract.
#[no_mangle]
pub unsafe extern "C" fn llave_change_session_pin(
    sealed: *const c_char,
    old_pin: *const c_char,
    new_pin: *const c_char,
) -> *mut c_char {
    // SAFETY: caller contract.
    let (sealed, old_pin, new_pin) = unsafe { (to_str(sealed), to_str(old_pin), to_str(new_pin)) };
    ffi_guard(
        || match crate::api::change_session_pin(sealed, old_pin, new_pin) {
            Ok(v) => ok_json(serde_json::Value::String(v)),
            Err(e) => err_json(&e),
        },
    )
}

// ---------------------------------------------------------------------------
// Async operations (block_on)
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn llave_request_pin() -> *mut c_char {
    ffi_guard(|| api_result_json(rt().block_on(crate::api::request_pin())))
}

/// # Safety
///
/// `nif` must satisfy the module-level pointer contract (null = saved NIF).
#[no_mangle]
pub unsafe extern "C" fn llave_check_nif(nif: *const c_char) -> *mut c_char {
    // SAFETY: caller contract.
    let nif = unsafe { to_opt(nif) };
    ffi_guard(|| match rt().block_on(crate::api::check_nif(nif)) {
        Ok(r) => ok_json(serde_json::json!({
            "nif": r.nif,
            "status": r.status,
            "responseJson": r.response_json,
        })),
        Err(e) => err_json(&e),
    })
}

#[no_mangle]
pub extern "C" fn llave_get_my_data() -> *mut c_char {
    ffi_guard(|| api_result_json(rt().block_on(crate::api::get_my_data())))
}

#[no_mangle]
pub extern "C" fn llave_get_history() -> *mut c_char {
    ffi_guard(|| api_result_json(rt().block_on(crate::api::get_history())))
}

#[no_mangle]
pub extern "C" fn llave_get_pending_requests() -> *mut c_char {
    ffi_guard(|| api_result_json(rt().block_on(crate::api::get_pending_requests())))
}

/// # Safety
///
/// `token` and `idp_code` must satisfy the module-level pointer contract.
#[no_mangle]
pub unsafe extern "C" fn llave_confirm_request(
    token: *const c_char,
    idp_code: *const c_char,
) -> *mut c_char {
    // SAFETY: caller contract.
    let (token, idp_code) = unsafe { (to_str(token), to_str(idp_code)) };
    ffi_guard(|| api_result_json(rt().block_on(crate::api::confirm_request(token, idp_code))))
}

/// # Safety
///
/// `token` and `idp_code` must satisfy the module-level pointer contract.
#[no_mangle]
pub unsafe extern "C" fn llave_reject_request(
    token: *const c_char,
    idp_code: *const c_char,
) -> *mut c_char {
    // SAFETY: caller contract.
    let (token, idp_code) = unsafe { (to_str(token), to_str(idp_code)) };
    ffi_guard(|| api_result_json(rt().block_on(crate::api::reject_request(token, idp_code))))
}

/// # Safety
///
/// `value` must satisfy the module-level pointer contract.
#[no_mangle]
pub unsafe extern "C" fn llave_qr_authenticate(value: *const c_char) -> *mut c_char {
    // SAFETY: caller contract.
    let value = unsafe { to_str(value) };
    ffi_guard(|| api_result_json(rt().block_on(crate::api::qr_authenticate(value))))
}

#[no_mangle]
pub extern "C" fn llave_deactivate() -> *mut c_char {
    ffi_guard(|| api_result_json(rt().block_on(crate::api::deactivate())))
}

/// # Safety
///
/// All pointer arguments must satisfy the module-level pointer contract.
#[no_mangle]
pub unsafe extern "C" fn llave_dni_authenticate(
    nif: *const c_char,
    fecha: *const c_char,
    soporte: *const c_char,
    challenge: *const c_char,
) -> *mut c_char {
    // SAFETY: caller contract.
    let (nif, fecha, soporte, challenge) = unsafe {
        (
            to_str(nif),
            to_str(fecha),
            to_str(soporte),
            to_str(challenge),
        )
    };
    ffi_guard(|| {
        api_result_json(rt().block_on(crate::api::dni_authenticate(nif, fecha, soporte, challenge)))
    })
}

/// # Safety
///
/// All pointer arguments must satisfy the module-level pointer contract.
#[no_mangle]
pub unsafe extern "C" fn llave_dni_complete_activation(
    nif: *const c_char,
    cookies_json: *const c_char,
    timestamp_alta_sms: *const c_char,
    token_clave_movil_sms: *const c_char,
    sms_pin: *const c_char,
) -> *mut c_char {
    // SAFETY: caller contract.
    let (nif, cookies_json, timestamp_alta_sms, token_clave_movil_sms, sms_pin) = unsafe {
        (
            to_str(nif),
            to_str(cookies_json),
            to_str(timestamp_alta_sms),
            to_str(token_clave_movil_sms),
            to_str(sms_pin),
        )
    };
    ffi_guard(|| {
        api_result_json(rt().block_on(crate::api::dni_complete_activation(
            nif,
            cookies_json,
            timestamp_alta_sms,
            token_clave_movil_sms,
            sms_pin,
        )))
    })
}

/// # Safety
///
/// `url` must satisfy the module-level pointer contract (null = no proxy).
#[no_mangle]
pub unsafe extern "C" fn llave_set_proxy(url: *const c_char) -> *mut c_char {
    // SAFETY: caller contract.
    let url = unsafe { to_opt(url) };
    ffi_guard(|| {
        crate::api::set_proxy(url);
        ok_json(serde_json::json!(true))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Take ownership of an export's result and parse its JSON envelope.
    fn take(ptr: *mut c_char) -> serde_json::Value {
        // SAFETY: `ptr` was just returned by a `llave_*` export.
        let s = unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();
        // SAFETY: freed exactly once, not used afterwards.
        unsafe { llave_free_string(ptr) };
        serde_json::from_str(&s).unwrap()
    }

    #[test]
    fn guard_turns_panic_into_error_envelope() {
        let v = take(ffi_guard(|| panic!("boom")));
        assert_eq!(v["ok"], false);
    }

    #[test]
    fn free_string_accepts_null() {
        // SAFETY: null is explicitly allowed.
        unsafe { llave_free_string(std::ptr::null_mut()) };
    }

    #[test]
    fn to_opt_maps_null_to_none() {
        // SAFETY: null is explicitly allowed.
        assert_eq!(unsafe { to_opt(std::ptr::null()) }, None);
    }

    #[test]
    fn validate_nif_null_is_error() {
        // SAFETY: null is explicitly allowed by the contract.
        let v = take(unsafe { llave_validate_nif(std::ptr::null()) });
        assert_eq!(v["ok"], false);
    }

    #[test]
    fn validate_nif_valid_returns_normalised() {
        let nif = CString::new(" 12345678z ").unwrap();
        // SAFETY: `nif` is a valid C string alive for the call.
        let v = take(unsafe { llave_validate_nif(nif.as_ptr()) });
        assert_eq!(v["data"], "12345678Z");
    }

    #[test]
    fn version_names_the_clave_app() {
        let v = take(llave_version());
        assert!(v["data"].as_str().unwrap().contains("Cl@ve 6.2.10"));
    }

    #[test]
    fn dni_input_reports_the_field() {
        let nif = CString::new("X1234567L").unwrap();
        // SAFETY: `nif` is a valid C string alive for the call.
        let v = take(unsafe { llave_dni_input(nif.as_ptr()) });
        assert_eq!(v["data"], "support_number");
    }
}
