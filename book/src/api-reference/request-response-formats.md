# Request & Response Formats

Formats as used by the official app v6.2.10 (build 294) and by llave.

## Common Request Fields (form-urlencoded)

| Field | Value | Description |
|-------|-------|-------------|
| `device_id` | e.g. `OyXoskiz6fLnzd080Xxzrpr40` | Server-assigned device identifier (returned by `ClaveActivateAuthenticationSv`). Sent empty to `ClaveStartingSv` before activation. |
| `user_password` | 128-char uppercase hex | Device password (see [Device Password](../cryptography/device-password.md)) |
| `NIF` | e.g. `12345678Z` | Spanish tax ID (DNI: 8 digits + letter; NIE: X/Y/Z + 7 digits + letter; also K/L/M + 7 digits + letter) |
| `sistema_operativo` | `"A"` | OS type. `"A"` = Android. |
| `version_os` | `"14"` | Android version (`Build.VERSION.RELEASE`) |
| `version_app` | `"6.2.10"` | App version string |
| `modelo` | `"Samsung SM-S928U"` | Capitalised manufacturer + `" "` + model (see [Constants](./constants.md#app-identity)) |
| `token_push` | Firebase id or `""` | Push token. llave sends `""`. |
| `timestamp` | e.g. `"19700101T000000000Z"` | Polling cursor for `ClaveRequestAllOperationsSv` (format `yyyyMMdd'T'HHmmssSSS'Z'`) |

Fields sent per endpoint (besides `sistema_operativo`, `version_os`, `version_app`, which all of them send):

| Endpoint | `device_id` | `user_password` | `NIF` | Other |
|----------|:-:|:-:|:-:|-------|
| `ClaveStartingSv` | yes | | yes | `token_push`, `modelo` |
| `ClaveIsNifActivatedSv`, `ClaveCheckMyDataSv`, `ClaveDesactivateAuthSv` | yes | | yes | |
| `ClaveRequestAllOperationsSv` | yes | | yes | `timestamp` |
| `ClaveRequestPinSv` | yes | yes | yes | |
| `ClaveAuthenticateSv` | yes | yes | yes | `tokenClaveMovil`, `codigoIdP` |
| `ClaveCancelAuthenticateSv` | yes | | yes | `tokenClaveMovil`, `codigoIdP` |
| `ClaveMovilQrSv` | yes | | yes | `valor_qr` (raw QR text) |
| `ClaveOperationsHistorySv` | yes | yes | yes | filters, see [below](#operations-history-detail) |
| `ClaveRequestStateSv` | | | | |
| `ClaveActivateAuthenticationSv` | | yes | | `token_push`, `modelo` |

## JSON Bodies

Newer endpoints (`SetCookieAppMovil`, certificate registration, VideoID, phone verification) take a JSON body sent with `Content-Type: application/json; charset=UTF-8` (Retrofit's Gson converter). Example, `SetCookieAppMovil`:

```json
{
  "version_app": "6.2.10",
  "sistema_operativo": "A",
  "version_os": "14",
  "modelo": "Samsung SM-S928U"
}
```

`ObtenerClaveMovilSMS` is a POST with an **empty body and no Content-Type**.

## Common Request Headers

| Header | Value | Description |
|--------|-------|-------------|
| `User-Agent` | `APPMovil/Cl@ve/v6.2.10(294)/14/Android/Dalvik/2.1.0 (…)` | Full app user agent; a truncated one is rejected by the WAF. See [Client Identity](../implementation-guide/client-identity.md). |
| `Accept` | `application/json` | |
| `Accept-Language` | `es_ES` | App language |
| `Cookie` | See [Cookies](#cookies) | Built explicitly for every call |

The app sends no other custom headers. In particular, there is no `TrazasApp` header in 6.2.10.

## Cookies

The app keeps **one cookie map** (`map_cookies` in its secure store) and builds the `Cookie` header itself; OkHttp has no cookie jar. llave does the same:

- **One map for all hosts.** Every stored cookie is sent to www2, www6 and www12 alike, regardless of the domain that set it.
- **`JSESSIONID` is never sent on API calls.** The app's `recuperarCookies(requireJsessionId = false)` skips it; only the PDF download can include it.
- **Empty values delete the cookie.** A `Set-Cookie` with an empty value or `""` removes it from the map, and empty values are never sent.
- **Redirects reuse the original `Cookie` header.** OkHttp copies the request's explicit header to every hop, and follows at most 20 redirects. llave follows redirects by hand with the same header and limit, and stores `Set-Cookie` from every hop.
- **Session reset keeps `appmovil`, `sgat-language` and `token-exteriores`.** The app clears everything else when it enters the identification screen; llave does the same at the start of activation.

llave pre-seeds `sgat-language=es_ES`. See [Cookie Management](../implementation-guide/cookie-management.md).

## Standard Response Envelope

All JSON responses follow this pattern:

```json
{
  "status": "OK",
  "visible": "N",
  "crashlytics": "N",
  "codigo_error": null,
  "mensaje": null,
  "respuesta": { ... }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `status` | `string` | `"OK"` for success, `"KO"` for failure |
| `visible` | `string?` | `"S"` = show the server `mensaje` to the user as is |
| `crashlytics` | `string?` | `"S"` = report to Crashlytics |
| `codigo_error` | `string?` | Error code (see [Error Codes](./error-codes.md)) |
| `mensaje` | `string?` | Human-readable error message (Spanish) |
| `respuesta` | `T?` | Endpoint-specific payload |

The app's model for `SetCookieAppMovil` names the error field `codigoError` instead of `codigo_error`, and its `respuesta` is empty.

## Specific Response Payloads

### ClaveStartingSv

```json
{
  "listaBlancaUrls": ["url1", "url2"],
  "listaNegraUrls": ["url3"],
  "menu": { "nombre": "...", "idioma": "...", "elementos": [ ... ] }
}
```

URL allow/deny lists for the app's WebView, plus the side menu. llave ignores `menu`.

### ClaveIsNifActivatedSv

Same payload as `ClaveStartingSv`. Only `status` matters: `OK` means the device/NIF is active. The app (and llave) require it before `ClaveCheckMyDataSv`.

### ClaveRequestAllOperationsSv

```json
{
  "pin": {
    "timeToLive": "120000",
    "timestamp": "20261015T120000000Z"
  },
  "peticion": {
    "tokenClaveMovil": "abc123...",
    "organismo": "AEAT",
    "timeToLive": "300000",
    "codigoIdP": "1234",
    "timestamp": "20261015T120001000Z",
    "tipo": "..."
  },
  "timeToLive": "...",
  "timestamp": "..."
}
```

| Field | Meaning |
|-------|---------|
| `pin` | Present when a PIN is pending (requested on the AEAT website). The app only then offers to reveal it with `ClaveRequestPinSv`; otherwise it shows "no tiene pendiente ningún PIN". |
| `peticion` | Pending Cl@ve Móvil request. The app treats it as a request only when `tokenClaveMovil` is set. `codigoIdP` is the code the user compares with the website; `organismo`/`tipo` are shown to the user. |
| `peticion.timestamp` | New polling cursor. The app persists it and ignores a `peticion` older than the stored cursor. |

All `timeToLive` values are **milliseconds**. When nothing is pending, the server answers KO `40` (the app's "no request" screen); llave also treats KO `205` as "nothing pending" while polling (see [Error Codes](./error-codes.md)).

### ClaveRequestPinSv

```json
{
  "pin": "<pin>",
  "timeToLive": "120000"
}
```

Reveals the PIN already announced by `respuesta.pin` in AllOperations; it does not create one. `timeToLive` is the remaining validity in **milliseconds** (the app passes it straight to `CountDownTimer`). llave converts it to seconds. KO `307` = no valid PIN.

### ClaveAuthenticateSv

```json
{
  "pending_requests": "S"
}
```

`pending_requests` tells whether more requests are waiting.

### ClaveMovilQrSv

```json
{
  "peticion": { "tokenClaveMovil": "...", "organismo": "...", "codigoIdP": "...", "timestamp": "...", "timeToLive": "...", "tipo": "..." }
}
```

The QR creates a normal pending request, which must then be confirmed (`ClaveAuthenticateSv`) or rejected (`ClaveCancelAuthenticateSv`). The app only accepts QR text that starts with `"0"`, is at least 152 characters long, and whose remainder is Base64 decoding to a multiple of 8 bytes.

### ClaveCheckMyDataSv

```json
{
  "email": "user@example.com",
  "numTelefono": "612345678",
  "nivelRegistro": "A"
}
```

### ClaveActivateAuthenticationSv

```json
{
  "device_id": "OyXoskiz6fLnzd080Xxzrpr40",
  "nif": "12345678Z",
  "token": "fzReyhbEtsPqdfXfDM8P9Zs9rSSKwEu9eLobA1SK3L4..."
}
```

The `device_id` is server-assigned and required for all later calls; the app stores the returned `nif`. The response does **not** contain a `user_password`: the device password is always the one the client generated and sent. `token` is not the device password.

### ClaveRequestStateSv

```json
{
  "registrado": "true",
  "nivelRegistro": "A",
  "telefono": "S"
}
```

| Field | Values | Meaning |
|-------|--------|---------|
| `registrado` | `"true"` / other | Whether the NIF is registered in Cl@ve |
| `nivelRegistro` | `"A"`, `"B"`, etc. | Registration level |
| `telefono` | `"S"` / `"N"` | Whether a mobile phone is registered |

The app requests the SMS only when `registrado == "true"` and `telefono == "S"`.

### ObtenerClaveMovilSMS

```json
{
  "tokenClaveMovilSms": "838f5824490558cc6c456e550ae7b6c2...",
  "timeStampAltaSms": "20260310151053593031",
  "movil": "22230",
  "horaPeticion": "15:10:53 (GMT +1)"
}
```

| Field | Description |
|-------|-------------|
| `tokenClaveMovilSms` | Token for this SMS session. Sent back in `ValidarClaveMovilSMS`. |
| `timeStampAltaSms` | Server timestamp of the SMS request. Sent back in `ValidarClaveMovilSMS`. |
| `movil` | Last digits of the registered phone number (for display). |
| `horaPeticion` | Time of the SMS request (for display). |

### ClaveCancelSv (not implemented by llave)

```json
{
  "csv": "...",
  "url": "..."
}
```

Renunciation of Cl@ve: a secure verification code and the URL of the PDF receipt.

## Operations History Detail

**Endpoint**: `ClaveOperationsHistorySv` (www2)

### Request Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| `device_id` | string | Device identifier |
| `user_password` | 128-char hex | Device password |
| `NIF` | Tax ID | User's NIF |
| `sistema_operativo` | `"A"` | OS type |
| `version_os` | `"14"` | OS version |
| `version_app` | `"6.2.10"` | App version |
| `aut_fir` | `"0"` | Always `"0"` in the app; meaning unknown |
| `resultado` | `"00"` / `"01"` / `"10"` | `"00"` = all, `"01"` = confirmed (app default), `"10"` = rejected |
| `organismo` | `""` | Organisation filter (empty = all) |
| `fecha_desde` | `"08/10/2026"` | Start date, `dd/MM/yyyy` |
| `fecha_hasta` | `"15/10/2026"` | End date, `dd/MM/yyyy` |
| `tipo` | `"111"` | Always `"111"` in the app (all request types) |
| `orden` | `"0"` | Always `"0"`; the app sorts on the device |

The app shows the last **7 days** of **confirmed** requests by default. llave uses the same defaults (`HISTORY_DEFAULT_DAYS`, `HistoryResult::Confirmed`); the CLI exposes `--days` and `--result`.

### Response

`respuesta` is a **JSON array** (not a wrapper object):

```json
[
  {
    "fecha": "14/10/2026 12:30:45",
    "resultado": "01",
    "tipo": "001",
    "organismo": "AEAT"
  },
  {
    "fecha": "13/10/2026 09:15:22",
    "resultado": "10",
    "tipo": "010",
    "organismo": "SEGURIDAD SOCIAL"
  }
]
```

### Result Codes

| Code | Meaning |
|------|---------|
| `"01"` | Confirmed |
| `"10"` | Rejected |

### Type Bitmask (inferred)

`tipo` looks like a 3-digit bitmask of request channels: SMS, QR and DNI/NIE. The exact bit order is only partially confirmed:

| Value | Probable meaning |
|-------|------------------|
| `"100"` | SMS |
| `"010"` | QR |
| `"001"` | DNI/NIE |
| `"111"` | All (filter value) |
