# Request & Response Formats

## Common Request Fields (form-urlencoded)

Most endpoints accept a standard set of form fields:

| Field | Value | Description |
|-------|-------|-------------|
| `device_id` | UUID string | Server-generated unique device identifier (returned by `ClaveActivateAuthenticationSv`) |
| `user_password` | 128-char hex string | Device password (see [Device Password](../cryptography/device-password.md)) |
| `NIF` | e.g. `12345678Z` | Spanish tax identification number (DNI: 8 digits + letter, NIE: X/Y/Z + 7 digits + letter) |
| `sistema_operativo` | `"A"` | OS type. `"A"` = Android. |
| `version_os` | `"14"` | Android OS version number |
| `version_app` | `"6.2.5"` | App version string |
| `modelo` | `"SM-S928U"` | Device model name |
| `token_push` | FCM token or `""` | Firebase Cloud Messaging push token |
| `timestamp` | e.g. `"19700101T000000000Z"` | Polling cursor for `ClaveRequestAllOperationsSv` (format: `yyyyMMdd'T'HHmmssSSS'Z'`) |

## Common Request Headers

| Header | Value | Description |
|--------|-------|-------------|
| `User-Agent` | See [Client Identity](../implementation-guide/client-identity.md) | Full Android app user agent string (truncation causes WAF rejection) |
| `Accept` | `application/json` | Expected response format |
| `Accept-Language` | `es_ES` | Language preference |
| `Cookie` | All managed cookies | Manually propagated across all subdomains |
| `TrazasApp` | JSON object | Cookie diagnostic telemetry (see [Client Identity](../implementation-guide/client-identity.md#the-trazasapp-header)) |

## Standard Response Envelope

All JSON responses (except `AutenticaDniNieContrasteh` which returns HTML) follow this pattern:

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
| `visible` | `string?` | `"S"` = show error to user, `"N"` = silent |
| `crashlytics` | `string?` | `"S"` = report to Crashlytics, `"N"` = don't |
| `codigo_error` | `string?` | Machine-readable error code (see [Error Codes](./error-codes.md)) |
| `mensaje` | `string?` | Human-readable error message (Spanish) |
| `respuesta` | `T?` | Endpoint-specific response payload (type varies) |

## Specific Response Payloads

### ClaveStartingSv

```json
{
  "listaBlancaUrls": ["url1", "url2"],
  "listaNegraUrls": ["url3"]
}
```

### ClaveIsNifActivatedSv

```json
{
  "activado": "S",
  "nivelAcceso": "A"
}
```

### ClaveRequestPinSv

```json
{
  "pin": "123456",
  "timeToLive": "180"
}
```

`timeToLive` is in seconds. Typically 180 (3 minutes).

### ClaveAuthenticateSv

```json
{
  "pending_requests": "0"
}
```

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
  "token": "fzReyhbEtsPqdfXfDM8P9Zs9rSSKwEu9eLobA1SK3L4..."
}
```

The `device_id` is server-generated and must be stored for all future API calls.
The `token` is a session/push token (not the device password).

### ClaveRequestAllOperationsSv

```json
{
  "pin": null,
  "peticion": {
    "timestamp": "20260314T143022123Z",
    "tokenClaveMovil": "abc123...",
    "codigoIdP": "SEGURIDAD_SOCIAL",
    "...": "..."
  }
}
```

When no pending operations exist, the server returns `status: "KO"` with `codigo_error: "205"`. This is expected and not a real error.

### ClaveRequestStateSv

```json
{
  "registrado": "S",
  "nivelRegistro": "A",
  "telefono": "S"
}
```

| Field | Values | Meaning |
|-------|--------|---------|
| `registrado` | `"S"` / `"N"` | Whether the user has a registered device (Si/No) |
| `nivelRegistro` | `"A"`, `"B"`, etc. | Registration level (A = highest) |
| `telefono` | `"S"` / `"N"` | Whether a phone number is registered |

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
| `tokenClaveMovilSms` | Hex token identifying this SMS session. Must be sent back in the SMS validation step. |
| `timeStampAltaSms` | Server timestamp of the SMS request. Must be sent back in the SMS validation step. |
| `movil` | Last 5 digits of the registered phone number (masked for display). |
| `horaPeticion` | Human-readable time of the SMS request (for display). |

## Operations History Detail

**Endpoint**: `ClaveOperationsHistorySv` (www2)

### Request Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| `device_id` | UUID | Device identifier |
| `user_password` | 128-char hex | Device password |
| `NIF` | Tax ID | User's NIF |
| `sistema_operativo` | `"A"` | OS type |
| `version_os` | `"14"` | OS version |
| `version_app` | `"6.2.5"` | App version |
| `aut_fir` | `"0"` | Authentication filter (0 = all) |
| `resultado` | `"00"` | Result filter: `"00"` = both accepted and rejected |
| `organismo` | `""` | Organisation filter (empty = all) |
| `fecha_desde` | `"14/02/2026"` | Start date (DD/MM/YYYY format) |
| `fecha_hasta` | `"14/03/2026"` | End date (DD/MM/YYYY format) |
| `tipo` | `"111"` | Type bitmask filter: `"111"` = all types |
| `orden` | `"0"` | Sort order: `"0"` = most recent first |

### Response

The `respuesta` field is a **direct JSON array** (not a wrapper object):

```json
[
  {
    "fecha": "14/03/2026 12:30:45",
    "resultado": "01",
    "tipo": "001",
    "organismo": "AEAT"
  },
  {
    "fecha": "13/03/2026 09:15:22",
    "resultado": "10",
    "tipo": "010",
    "organismo": "SEGURIDAD SOCIAL"
  }
]
```

### Result Codes

| Code | Meaning |
|------|---------|
| `"01"` | Confirmed / Accepted |
| `"10"` | Rejected |

### Type Bitmask

A 3-digit binary string where each bit represents an authentication method:

| Bit position | Bit | Method |
|-------------|-----|--------|
| 0 (rightmost) | `tipo[2]` | DNI/NIE |
| 1 (middle) | `tipo[1]` | QR code |
| 2 (leftmost) | `tipo[0]` | SMS |

Examples:
- `"001"` = DNI/NIE only
- `"010"` = QR only
- `"100"` = SMS only
- `"011"` = DNI/NIE + QR
- `"111"` = all methods (used as default filter)
