# DNI/NIE + SMS Activation

This is the primary onboarding path for Spanish taxpayers who want to use the Clave Movil system. It uses *weak authentication* (autenticacion debil) -- identity proven via data printed on a physical DNI or NIE card, combined with SMS verification to the registered phone number.

## Overview

The flow has **two phases**:

1. **Phase 1** -- DNI/NIE credentials establish a session, then an SMS code is sent to the registered mobile.
2. **Phase 2** -- The user enters the SMS code, which is validated, and then the device is activated (registered) with the AEAT backend.

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  User's DNI  │     │ User's Phone │     │  AEAT Server │
│  (physical)  │     │  (receives   │     │  (www2/www6/ │
│              │     │   SMS code)  │     │   www12)     │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       │  NIF + date +      │                    │
       │  soporte number    │                    │
       └────────────────────┼───────────────────>│
                            │     SMS code       │
                            │<───────────────────│
                            │                    │
                            │  Enter SMS code    │
                            │───────────────────>│
                            │                    │
                            │  Device activated  │
                            │<───────────────────│
```

## Phase 1: DNI/NIE Auth + SMS Request

```mermaid
sequenceDiagram
    participant App as Mobile App
    participant www2 as www2 (Gateway)
    participant www12 as www12 (Registration)
    participant Phone as User's Phone

    Note over App: Generate random device_id (UUIDv4)

    App->>www2: POST /wlpl/MOVI-P24H/ClaveStartingSv
    Note right of www2: Establishes device context<br/>Sets appmovil cookie
    www2-->>App: {"status":"KO","codigo":"105"}<br/>+ Set-Cookie: appmovil=...

    Note over App: KO 105 is expected -- continue anyway

    App->>www2: POST /wlpl/BUCV-JDIT/AutenticaDniNieContrasteh
    Note right of www2: DNI/NIE weak auth<br/>NIF + fecha + soporte
    www2-->>App: 302 Redirect -> www12
    Note over App: Follow redirect chain manually,<br/>capturing Set-Cookie at each hop
    App->>www12: GET /wlpl/MOVI-AEAT/AccesoW12Sv
    www12-->>App: 200 OK (HTML -- ignored)<br/>+ Set-Cookie: WWW12, WWW12V, sgat_id_usr_sede

    App->>www12: POST /wlpl/MOVI-P24H/ClaveRequestStateSv
    Note right of www12: Checks registration state<br/>Uses session cookies from DNI auth
    www12-->>App: {"status":"OK","respuesta":<br/>{"registrado":"S","telefono":"S","nivelRegistro":"A"}}

    App->>www12: POST /wlpl/MOVI-P24H/ObtenerClaveMovilSMS
    Note right of www12: Empty POST body!<br/>Triggers SMS to registered phone
    www12-->>App: {"status":"OK","respuesta":<br/>{"movil":"22230","horaPeticion":"15:10:53 (GMT +1)",<br/>"tokenClaveMovilSms":"838f...","timeStampAltaSms":"2026..."}}

    www12->>Phone: SMS with verification code

    Note over App: Serialize cookies to JSON<br/>for phase 2 resumption
```

## Phase 2: SMS Validation + Device Activation

```mermaid
sequenceDiagram
    participant User as User
    participant App as Mobile App
    participant www12 as www12 (Registration)
    participant www6 as www6 (Activation)

    User->>App: Enters SMS code from phone

    Note over App: Import cookies from Phase 1 JSON

    App->>www12: POST /wlpl/MOVI-P24H/ValidarClaveMovilSMS
    Note right of www12: Validates SMS PIN<br/>timeStampAltaSms + tokenClaveMovilSms + pinAcceso
    www12-->>App: {"status":"OK"}<br/>+ Set-Cookie: pin24H=..., pin24V=...

    Note over App: pin24H cookie is now set!<br/>Generate device_password (UUIDv4)

    App->>www6: POST /wlpl/MOVI-P24H/ClaveActivateAuthenticationSv
    Note right of www6: JSESSIONID is STRIPPED<br/>pin24H cookie proves SMS verification<br/>user_password + device metadata
    www6-->>App: {"status":"OK","respuesta":<br/>{"device_id":"OyXosk...","token":"fzReyh..."}}

    Note over App: Save session:<br/>device_id (from server)<br/>+ device_password (client-generated)<br/>+ NIF

    App->>App: Persist session to secure storage
```

## Complete End-to-End Sequence

```mermaid
sequenceDiagram
    autonumber
    participant U as User
    participant C as Client
    participant W2 as www2
    participant W12 as www12
    participant W6 as www6
    participant SMS as SMS Gateway

    rect rgb(240, 248, 255)
        Note over C,W2: Phase 1a: Session Initialization
        C->>W2: POST ClaveStartingSv {device_id, NIF, ...}
        W2-->>C: KO/105 + Set-Cookie: appmovil
    end

    rect rgb(255, 248, 240)
        Note over C,W12: Phase 1b: DNI/NIE Weak Authentication
        U->>C: Provides NIF, fecha, soporte
        C->>W2: POST AutenticaDniNieContrasteh {NIF, FECHA, SOPORTE, ...}
        W2-->>C: 302 -> www12 + Set-Cookie: JSESSIONID, WWW12, WWW12V
        C->>W12: GET AccesoW12Sv (follow redirect)
        W12-->>C: 200 HTML + Set-Cookie: sgat_id_usr_sede
    end

    rect rgb(240, 255, 240)
        Note over C,W12: Phase 1c: Registration Check + SMS
        C->>W12: POST ClaveRequestStateSv
        W12-->>C: {registrado: "S", telefono: "S", nivel: "A"}
        C->>W12: POST ObtenerClaveMovilSMS (empty body!)
        W12-->>C: {tokenClaveMovilSms, timeStampAltaSms, movil: "22230"}
        W12->>SMS: Trigger SMS delivery
        SMS->>U: SMS code received
    end

    Note over C: Serialize cookies -> JSON<br/>Wait for user to enter SMS code

    rect rgb(255, 240, 255)
        Note over C,W6: Phase 2: SMS Validation + Device Activation
        U->>C: Enters SMS code
        C->>C: Import cookies from Phase 1
        C->>W12: POST ValidarClaveMovilSMS {timestamp, token, pin}
        W12-->>C: OK + Set-Cookie: pin24H, pin24V
        C->>W6: POST ClaveActivateAuthenticationSv {user_password, ...}
        Note over C,W6: No JSESSIONID! Has pin24H.
        W6-->>C: {device_id: "OyXosk...", token: "fzReyh..."}
    end

    rect rgb(248, 248, 248)
        Note over C: Session Saved
        C->>C: Save {device_id, NIF, device_password} to secure storage
    end
```

## Step-by-Step HTTP Details

### Step 0: ClaveStartingSv

**Purpose**: Establish device context with the AEAT backend. Sets the `appmovil` cookie, which is required by subsequent endpoints.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www2.agenciatributaria.gob.es/wlpl/MOVI-P24H/ClaveStartingSv` |
| **Content-Type** | `application/x-www-form-urlencoded` |
| **Required Cookies** | `sgat-language` (pre-seeded) |

**Form Parameters**:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `device_id` | Random UUIDv4 | `601a4147-fcd3-4c6a-9a9d-e627ca86deb8` |
| `NIF` | User's tax ID | `09030055W` |
| `sistema_operativo` | OS identifier | `A` (Android) |
| `token_push` | Firebase push token | `""` (empty for initial setup) |
| `version_os` | OS version | `14` |
| `version_app` | App version | `6.2.5` |
| `modelo` | Device model | `SM-S928U` |

**Response**: JSON with `status` field.

```json
{
  "status": "KO",
  "codigo_error": "105",
  "mensaje": "?",
  "visible": "N",
  "crashlytics": "N"
}
```

> A `KO` response with code `105` is **expected and normal** during initial
> registration. The important side effect is the `appmovil` Set-Cookie header.
> The flow continues regardless of this response status.

**Cookies Set**: `appmovil` (Base64-encoded JSON):

```json
{
  "app": "clave",
  "version_app": "6.2.5",
  "sistema_operativo": "A",
  "version_os": "14",
  "modelo": "SM-S928U"
}
```

### Step 1: AutenticaDniNieContrasteh

**Purpose**: Authenticate using DNI/NIE card data. This is the "weak authentication" step -- it proves the caller has the physical card's printed data. The response body is HTML and is completely ignored. Only the **cookies set during the redirect chain** matter.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www2.agenciatributaria.gob.es/wlpl/BUCV-JDIT/AutenticaDniNieContrasteh?ref=%2Fwlpl%2FMOVI-AEAT%2FAccesoW12Sv` |
| **Content-Type** | `application/x-www-form-urlencoded` |
| **Required Cookies** | `sgat-language`, `appmovil` |

**Form Parameters**:

| Parameter | Value | Description |
|-----------|-------|-------------|
| `NIF` | e.g. `09030055W` | Spanish tax ID number |
| `FECHA` | e.g. `15-06-2025` | Date of validity (DNI) or date of birth (NIE), format `DD-MM-YYYY` |
| `SOPORTE` | e.g. `BAA123456` | Support/serial number from the physical card |
| `botonAutenticacionDebil` | `Continuar` | Literal string -- identifies this as weak auth |
| `APP` | `S` | Indicates mobile app context |
| `modo` | `json` | Request JSON-mode (though response is still HTML) |
| `AZUL` | `""` | Empty -- reserved for certificate auth |
| `FECHANIE` | `""` | Empty -- reserved for NIE-specific date |

**Response Handling**:

This endpoint **does not return JSON**. It returns HTML and triggers a redirect chain. The client must:

1. **Disable automatic redirects** (reqwest's auto-redirect strips custom headers on cross-origin redirects).
2. **Follow each redirect manually**, re-attaching `Cookie` and `TrazasApp` headers at every hop.
3. **Capture `Set-Cookie` headers** from every response in the chain.
4. **Count redirects** -- zero redirects means auth failed (server returned the login form again).

See [Redirect Handling](../implementation-guide/redirect-handling.md) for full details.

**Redirect Chain**:

```mermaid
flowchart TD
    A["POST AutenticaDniNieContrasteh<br/>(www2)"] -->|"302 Redirect"| B["GET /wlpl/MOVI-AEAT/AccesoW12Sv<br/>(www12)"]
    B -->|"200 OK"| C["HTML response (ignored)"]

    A -->|"Set-Cookie"| D["JSESSIONID<br/>sgat_id_usr_sede<br/>WWW12 + WWW12V"]
    B -->|"Set-Cookie"| E["sgat_id_usr_sede (updated)"]

    style A fill:#ff9,stroke:#333
    style C fill:#ddd,stroke:#333
```

**Cookies Set During Redirect Chain**:
- `JSESSIONID` -- www2 WebSphere session
- `WWW12` -- Authentication token (long encrypted string)
- `WWW12V` -- Timestamp for WWW12 (format: `YYYYMMDD-HHmmssSSS`)
- `sgat_id_usr_sede` -- URL-encoded JSON with session info

**Failure Detection**: If the redirect count is 0, the server returned the login form again, meaning the credentials (NIF, fecha, or soporte) were rejected.

### Step 2: ClaveRequestStateSv

**Purpose**: Check the user's registration state after DNI authentication.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www12.agenciatributaria.gob.es/wlpl/MOVI-P24H/ClaveRequestStateSv` |
| **Content-Type** | `application/x-www-form-urlencoded` |
| **Required Cookies** | All cookies from Steps 0-1 (especially `WWW12`, `WWW12V`, `sgat_id_usr_sede`) |

**Form Parameters**:

| Parameter | Value | Description |
|-----------|-------|-------------|
| `sistema_operativo` | `A` | Android |
| `version_os` | `14` | OS version |
| `version_app` | `6.2.5` | App version |

> No `device_id` or `NIF` is sent. The server identifies the user entirely from
> the session cookies established by DNI authentication.

**Response**: See [ClaveRequestStateSv payload](../api-reference/request-response-formats.md#claverequeststatessv).

### Step 3: ObtenerClaveMovilSMS

**Purpose**: Trigger an SMS verification code to the user's registered phone number.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www12.agenciatributaria.gob.es/wlpl/MOVI-P24H/ObtenerClaveMovilSMS` |
| **Content-Type** | **None** (empty body) |
| **Required Cookies** | All cookies from Steps 0-2 |

> **Critical**: This endpoint **must** be called with an **empty POST body** and
> **no Content-Type header**. Sending form-encoded data (even empty) causes
> [error 902024](../api-reference/error-codes.md).

**Request Body**: Empty string (`""`) -- produces `Content-Length: 0`.

**Response**: See [ObtenerClaveMovilSMS payload](../api-reference/request-response-formats.md#obtenerclavemovilsms).

At this point, the user's phone receives an SMS with a verification code.

**Phase 1 ends here.** The client serialises all cookies to JSON so they can be restored in Phase 2. See [Cookie Management](../implementation-guide/cookie-management.md) for serialisation details.

### Step 4: ValidarClaveMovilSMS

**Purpose**: Validate the SMS code entered by the user. On success, the server sets the `pin24H` and `pin24V` cookies.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www12.agenciatributaria.gob.es/wlpl/MOVI-P24H/ValidarClaveMovilSMS` |
| **Content-Type** | `application/x-www-form-urlencoded` |
| **Required Cookies** | All cookies from Phase 1 (imported from serialised JSON) |

**Form Parameters**:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `timeStampAltaSms` | Server timestamp from Step 3 | `20260310151053593031` |
| `tokenClaveMovilSms` | SMS session token from Step 3 | `838f5824490558cc6c456e550ae7b6c2...` |
| `pinAcceso` | The SMS code entered by the user | `123456` |

**Cookies Set**:
- `pin24H` -- Long encrypted token proving SMS verification
- `pin24V` -- Timestamp companion (format: `YYYYMMDD-HHmmssSSS`)
- `JSESSIONID` -- New session ID for www12

> **This is the pivotal step.** The `pin24H` cookie is the "gate" that allows
> device activation. Without it, Step 5 will return HTML instead of JSON.

### Step 5: ClaveActivateAuthenticationSv

**Purpose**: Activate (register) the device with the AEAT backend. Returns a server-generated `device_id`.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www6.agenciatributaria.gob.es/wlpl/MOVI-P24H/ClaveActivateAuthenticationSv` |
| **Content-Type** | `application/x-www-form-urlencoded` |
| **Required Cookies** | All cookies **except JSESSIONID** |

> **Critical**: The `JSESSIONID` from www2/www12 **must be explicitly excluded**.
> See [Cookie Management](../implementation-guide/cookie-management.md) for why.

**Form Parameters**:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `sistema_operativo` | OS identifier | `A` |
| `version_os` | OS version | `14` |
| `version_app` | App version | `6.2.5` |
| `token_push` | Firebase push token | `""` (empty) |
| `user_password` | Client-generated password | `a1b2c3d4-...` |
| `modelo` | Device model | `SM-S928U` |

**Required Cookies**:

```
Cookie: sgat-language=es_ES;
        sgat_id_usr_sede={...};
        appmovil={base64};
        WWW12={encrypted};
        WWW12V={timestamp};
        pin24H={encrypted};    <- REQUIRED (proves SMS was verified)
        pin24V={timestamp}     <- REQUIRED (companion to pin24H)
```

**Response**: See [ClaveActivateAuthenticationSv payload](../api-reference/request-response-formats.md#claveactivateauthenticationsv).

**After activation**, the session is saved:

```json
{
  "device_id": "OyXoskiz6fLnzd080Xxzrpr40",
  "nif": "09030055W",
  "device_password": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "firebase_token": null,
  "created_at": "2026-03-10T14:11:16.999Z"
}
```

See [Session Persistence](../implementation-guide/session-persistence.md) for storage details.

## Data Flow Diagram

```mermaid
flowchart TD
    Start([Start]) --> GenUUID["Generate device_id (UUIDv4)"]
    GenUUID --> S0["Step 0: ClaveStartingSv (www2)"]
    S0 -->|"KO 105 -- OK, got appmovil cookie"| S1
    S0 -->|"OK"| S1
    S0 -->|"Network error"| S1["Step 1: AutenticaDniNieContrasteh (www2)"]

    S1 -->|"0 redirects"| Fail1([DniAuthFailed:<br/>Bad credentials])
    S1 -->|">=1 redirect"| S2["Step 2: ClaveRequestStateSv (www12)"]

    S2 -->|"registrado=N"| Fail2([Not registered:<br/>User must register at<br/>agenciatributaria.gob.es])
    S2 -->|"registrado=S"| S3["Step 3: ObtenerClaveMovilSMS (www12)<br/>Empty POST body"]

    S3 -->|"OK"| Wait["Display masked phone<br/>Wait for SMS code"]
    S3 -->|"Error"| Fail3([SMS request failed])

    Wait -->|"Cookies -> JSON"| Phase2["Phase 2: User enters SMS code"]

    Phase2 --> Import["Import cookies from JSON"]
    Import --> S4["Step 4: ValidarClaveMovilSMS (www12)"]

    S4 -->|"KO"| Fail4([Invalid SMS code])
    S4 -->|"OK + pin24H cookie"| GenPwd["Generate device_password (UUIDv4)"]

    GenPwd --> S5["Step 5: ClaveActivateAuthenticationSv (www6)<br/>No JSESSIONID!"]

    S5 -->|"HTML response"| Fail5([Activation failed:<br/>Missing pin24H?])
    S5 -->|"OK"| Save["Save session:<br/>device_id (from server)<br/>+ device_password<br/>+ NIF"]

    Save --> Done([Device Activated!])

    style Fail1 fill:#fcc,stroke:#c33
    style Fail2 fill:#fcc,stroke:#c33
    style Fail3 fill:#fcc,stroke:#c33
    style Fail4 fill:#fcc,stroke:#c33
    style Fail5 fill:#fcc,stroke:#c33
    style Done fill:#cfc,stroke:#3c3
    style Wait fill:#ffc,stroke:#cc3
```

## Key Points

- The `device_password` is generated **client-side** before the activation call (see [Device Password](../cryptography/device-password.md))
- The `device_id` is generated **server-side** and returned in the response
- The `user_password` form field in step 5 contains the client-generated `device_password`
- The `token` field in the response is a session/push token, NOT the device password
- Cookies must be serialised between Phase 1 and Phase 2 (see [Cookie Management](../implementation-guide/cookie-management.md))
- `JSESSIONID` must be stripped before calling www6 (see [Cookie Management](../implementation-guide/cookie-management.md))
