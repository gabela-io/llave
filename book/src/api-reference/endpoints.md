# Endpoints

Endpoints of the official Cl@ve app **v6.2.10 (build 294)**, taken from its Retrofit interface (`ClaveApi`) and the URL literals in `Repository` / `UrlUtils`. Most calls are **form-urlencoded POST**; a few newer ones use a **JSON body** (`Content-Type: application/json; charset=UTF-8`).

Every call carries an explicit `Cookie` header built from one shared cookie map (see [Request & Response Formats](./request-response-formats.md#cookies)).

## Base URL

```
https://www2.agenciatributaria.gob.es/
```

See [AEAT Infrastructure](../architecture/aeat-infrastructure.md) for the full subdomain map.

## Endpoints Implemented by Llave

All paths are under `/wlpl/`. "llave" is the `LlaveClient` method in `crates/llave-core/src/api.rs`.

### Device session and account (www2)

| Endpoint | Body | Purpose | llave |
|----------|------|---------|-------|
| `MOVI-P24H/ClaveStartingSv` | form | App start: device context, may set `appmovil`. Returns URL allow/deny lists. Before activation `device_id` is sent empty. | `clave_starting` |
| `MOVI-JDIT/C/SetCookieAppMovil` | **JSON** `{version_app, sistema_operativo, version_os, modelo}` | Asks the server to set the `appmovil` cookie. Called after `ClaveStartingSv` returns OK and no `appmovil` cookie is stored, and before retrying `ClaveRequestStateSv` once after KO `987`. | `set_cookie_app_movil` |
| `MOVI-P24H/ClaveIsNifActivatedSv` | form | Checks that this device/NIF is still active. The payload is the `ClaveStartingSv` payload (URL lists); only `status` matters. The app calls it before "my data". | `clave_is_nif_activated` |
| `MOVI-P24H/ClaveCheckMyDataSv` | form | Account data (email, phone, registration level). | `clave_check_my_data` |
| `MOVI-P24H/ClaveRequestAllOperationsSv` | form | Poll: returns the pending PIN and/or pending authentication request. | `request_all_operations` |
| `MOVI-P24H/ClaveRequestPinSv` | form | **Reveals** a PIN that is already pending (announced by `respuesta.pin` in AllOperations). It does not create one. | `request_pin` |
| `MOVI-P24H/ClaveAuthenticateSv` | form | Confirm a pending Cl@ve Móvil request. | `authenticate` |
| `MOVI-P24H/ClaveCancelAuthenticateSv` | form | Reject a pending Cl@ve Móvil request (no `user_password`). | `cancel_authenticate` |
| `MOVI-P24H/ClaveMovilQrSv` | form (`valor_qr`) | Turns a QR shown on the AEAT website into a pending request, which is then confirmed or rejected. | `qr_authenticate` |
| `MOVI-P24H/ClaveOperationsHistorySv` | form | Operations history with date range and filters. | `operations_history` |
| `MOVI-P24H/ClaveDesactivateAuthSv` | form | Deactivate this device. | `deactivate_authentication` |

### Activation (DNI/NIE + SMS)

| Endpoint | Host | Body | Purpose | llave |
|----------|------|------|---------|-------|
| `BUCV-JDIT/AutenticaDniNieContrasteh?ref=%2Fwlpl%2FMOVI-AEAT%2FAccesoW12Sv` | www2 | form | DNI/NIE weak authentication (`modo=json`). | `authenticate_dni_nie` |
| `MOVI-P24H/ClaveRequestStateSv` | www12 | form (`sistema_operativo`, `version_os`, `version_app` only) | Registration state after DNI auth. KO `987` → `SetCookieAppMovil` and one retry. | `clave_request_state` |
| `MOVI-P24H/ObtenerClaveMovilSMS` | www12 | **empty** (no Content-Type) | Send the SMS code to the registered phone. | `request_sms_code` |
| `MOVI-P24H/ValidarClaveMovilSMS` | www12 | form | Validate the SMS code (sets `pin24H`). | `validate_sms_code` |
| `MOVI-P24H/ClaveActivateAuthenticationSv` | **www6** | form (no `device_id`/`NIF`) | Activate the device; the server assigns `device_id`. | `activate_authentication` |

### www12 web login

| Endpoint | Host | Body | Purpose | llave |
|----------|------|------|---------|-------|
| `MOVI-P24H/ObtenerClaveMovil` | www12 | GET | Part of the app's own www12 web login (in-app WebView sessions). **Not** an activation step. | not implemented |
| `MOVI-P24H/ValidarClaveMovil` | www12 | form (`tokenClaveMovil`) | Same web-login flow. After confirming a request, the app also calls it when a `WWW12` cookie is present; llave mirrors that. | `validate_llave_movil` |

> `LlaveClient::request_state` (`ClaveRequestStateSv` with `device_id`, `user_password`, `NIF`) has no caller and does not match the app, which only sends the three device fields.

## App Endpoints Llave Does Not Implement

| Endpoint | Host | Body | Purpose |
|----------|------|------|---------|
| `MOVI-P24H/ClaveSetFirebaseTokenSv` | www2 | form (`token_push`) | Register the Firebase push token. Push only wakes the app; it carries no request data. |
| `MOVI-P24H/ClaveCancelSv` | www2 | form (`device_id`, `user_password`, `NIF`, …) | **Renounce Cl@ve**: unregisters the user from the whole Cl@ve system and returns `{csv, url}` for a PDF receipt. Destructive; it is *not* "cancel PIN". |
| `MOVI-P24H/ClaveSendVerificationCodeSv`, `MOVI-P24H/ClaveValidatePhoneNumberSv` | www12 | JSON | Phone number verification. |
| `MOVI-P24H/ClaveStartVideoIdSv` | www12 | JSON | Start VideoID registration (eu.electronicid SDK). |
| `MOVI-P24H/NAC/ClaveCheckVideoIdSv`, `MOVI-P24H/NAC/1/ClaveResultVideoIdSv` | www12 | JSON | VideoID for Spanish nationals. |
| `MOVI-P24H/EXT/ClaveCheckVideoIdSv`, `MOVI-P24H/EXT/1/ClaveResultVideoIdSv`, `MOVI-P24H/EXT/2/ClaveResultVideoIdSv` | www12 | JSON | VideoID for foreign nationals. |
| `MOVI-JDIT/C/IniciarAltaConCertificadoSv` | www2 | JSON | Start [certificate registration](../authentication-flows/certificate-registration.md). |
| `MOVI-JDIT/C/ValidarAltaConCertificadoSv` | www12 | JSON | Validate certificate registration. |
| `MOVI-JDIT/C/ObtenerAltaConCertificadoSv?tokenId=` | www1 | GET (browser) | Certificate registration page. |
| `static_files/common/internet/dep/explotacion/moviles/clave/carruseles/<lang>/novedadesclave002.json` | www2 | GET (`If-Modified-Since`) | "What's new" carousel. |
| PDF download (URL from `ClaveCancelSv`) | — | GET | Renunciation receipt. The only call that may include `JSESSIONID`. |

### Dead code in the app

`ClaveApi` still declares these methods, but 6.2.10 has no URL for them and never calls them:

| Method | Endpoint |
|--------|----------|
| `claveMigrationSvAsync` | `ClaveMigrationSv` (old migration from the previous Cl@ve PIN app) |
| `claveGetAuthenticationTokenSvAsync` | unknown (no URL in the app; name suggests `ClaveGetAuthenticationTokenSv`) |

`ObtenerPeticionesMarketsSv`, documented in earlier versions of this book, does not appear anywhere in 6.2.10 and has been removed from llave.

## Web URLs (opened in a browser or WebView)

| URL | Purpose |
|-----|---------|
| `www12…/wlpl/OVCT-CXEW/SelectorAcceso` | Access selector (www12 cookie detection) |
| `www12…/wlpl/OVCT-CXEW/ObtenerPin?ref=` | Web "obtain PIN" page |
| `www12…/wlpl/MOVI-P24H/ObtenerClaveMovil?ref=` | Web Cl@ve Móvil login |
| `www2…/wlpl/MOVI-P24H/ObtenerClaveMovilQR` | QR variant of the web login |
| `www2…/wlpl/ADHT-AUTH/SesionCaducada` | Session expired page |
| `www1…/wlpl/MOVI-JDIT/Acceso?gestion=REGISTRO_CERTIFICADO&nif=` | Register via certificate |
| `www12…/wlpl/MOVI-JDIT/Acceso?gestion=REGISTRO_CARTA` | Register via letter |
| `www1…/wlpl/MOVI-JDIT/Acceso?gestion=MODIFICA_TELEFONO&nif=` | Change phone number |
| `www6…/wlpl/MOVI-JDIT/Acceso?gestion=MODIFICA_CORREO` | Change email |
| `www1…/wlpl/MOVI-JDIT/Acceso?gestion=OBTENER_SUPERIOR&nif=` | Upgrade registration level |
| `www2…/wlpl/MOVI-JDIT/Acceso?ext=OFICINAS_CLAVE` | Cl@ve offices |
| `www2…/wlpl/MOVI-JDIT/Acceso?ext=REGISTRO_PRESENCIAL` | In-person registration |
| `www2…/wlpl/MOVI-JDIT/Acceso?info=AYUDA_CERT_ANDROID` | Help with Android certificates |
| `www2…/wlpl/MOVI-JDIT/SoporteAppSv?APP=CLAVE` | App support |

## Endpoint Reference (Activation Flow)

The endpoints used, in order, by the [DNI/NIE + SMS activation flow](../authentication-flows/dni-nie-sms-activation.md):

| # | Endpoint | Subdomain | Path | Body | Purpose |
|---|----------|-----------|------|------|---------|
| 0 | `ClaveStartingSv` | www2 | `/wlpl/MOVI-P24H/` | Form | Device context (`device_id` empty) |
| 0b | `SetCookieAppMovil` | www2 | `/wlpl/MOVI-JDIT/C/` | JSON | Only if Starting is OK and no `appmovil` cookie |
| 1 | `AutenticaDniNieContrasteh` | www2 | `/wlpl/BUCV-JDIT/` | Form | DNI/NIE weak auth |
| 2 | `ClaveRequestStateSv` | www12 | `/wlpl/MOVI-P24H/` | Form | Registration state (KO `987` → 0b + retry once) |
| 3 | `ObtenerClaveMovilSMS` | www12 | `/wlpl/MOVI-P24H/` | **Empty** | Send SMS code |
| 4 | `ValidarClaveMovilSMS` | www12 | `/wlpl/MOVI-P24H/` | Form | Validate SMS code (sets `pin24H`) |
| 5 | `ClaveActivateAuthenticationSv` | www6 | `/wlpl/MOVI-P24H/` | Form | Device activation |
