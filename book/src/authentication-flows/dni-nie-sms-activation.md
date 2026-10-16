# DNI/NIE + SMS Activation

This is the main onboarding path for using Cl@ve Móvil. It uses *weak authentication* (autenticación débil): the user proves their identity with data printed on their physical DNI or NIE card, then confirms with an SMS code sent to the phone number registered in Cl@ve.

## Overview

The flow has **two phases**:

1. **Phase 1**: The DNI/NIE data establishes a session. The registration state is checked, and an SMS code is sent to the registered mobile.
2. **Phase 2**: The user enters the SMS code. The code is validated, and the device is activated (registered) with the AEAT backend.

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ User's card  │     │ User's Phone │     │  AEAT Server │
│  (DNI/NIE)   │     │  (receives   │     │  (www2/www6/ │
│              │     │   SMS code)  │     │   www12)     │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       │  NIF + date or     │                    │
       │  support number    │                    │
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
    participant App as Mobile App / llave
    participant www2 as www2 (Gateway)
    participant www12 as www12 (Registration)
    participant Phone as User's Phone

    Note over App: Clear session cookies<br/>(keep appmovil, sgat-language, token-exteriores)

    App->>www2: POST /wlpl/MOVI-P24H/ClaveStartingSv (device_id="")
    www2-->>App: OK / KO (KO expected before activation)
    opt Starting OK and no appmovil cookie
        App->>www2: POST /wlpl/MOVI-JDIT/C/SetCookieAppMovil (JSON)
        www2-->>App: Set-Cookie: appmovil
    end

    App->>www2: POST /wlpl/BUCV-JDIT/AutenticaDniNieContrasteh
    Note right of www2: NIF + FECHA or SOPORTE
    www2-->>App: 302 -> www12 (+ Set-Cookie)
    App->>www12: GET /wlpl/MOVI-AEAT/AccesoW12Sv
    www12-->>App: JSON envelope or HTML<br/>+ Set-Cookie: WWW12, WWW12V, sgat_id_usr_sede
    Note over App: KO -8 / -10 -> ask date of birth,<br/>resend with AZUL / FECHANIE = "X"

    App->>www12: POST /wlpl/MOVI-P24H/ClaveRequestStateSv
    www12-->>App: {"status":"OK","respuesta":<br/>{"registrado":"true","telefono":"S","nivelRegistro":"..."}}
    Note over App: KO 987 -> SetCookieAppMovil, retry once<br/>Gate: registrado=="true" && telefono=="S"

    App->>www12: POST /wlpl/MOVI-P24H/ObtenerClaveMovilSMS (empty body)
    www12-->>App: {"status":"OK","respuesta":<br/>{"movil":"22230","horaPeticion":"15:10:53 (GMT +1)",<br/>"tokenClaveMovilSms":"838f...","timeStampAltaSms":"2026..."}}

    www12->>Phone: SMS with verification code

    Note over App: Serialise cookies to JSON<br/>for phase 2
```

## Phase 2: SMS Validation + Device Activation

```mermaid
sequenceDiagram
    participant User as User
    participant App as Mobile App / llave
    participant www12 as www12 (Registration)
    participant www6 as www6 (Activation)

    User->>App: Enters SMS code from phone

    Note over App: Import cookies from phase 1

    App->>www12: POST /wlpl/MOVI-P24H/ValidarClaveMovilSMS
    Note right of www12: timeStampAltaSms + tokenClaveMovilSms + pinAcceso
    www12-->>App: {"status":"OK"}<br/>+ Set-Cookie: pin24H=..., pin24V=...

    Note over App: Generate user_password<br/>(SHA-512 hex of 256 random chars)

    App->>www6: POST /wlpl/MOVI-P24H/ClaveActivateAuthenticationSv
    Note right of www6: Cookie header without JSESSIONID<br/>pin24H proves SMS verification
    www6-->>App: {"status":"OK","respuesta":<br/>{"device_id":"OyXosk...","nif":"...","token":"fzReyh..."}}

    Note over App: Save session:<br/>device_id (server) + nif (server)<br/>+ device_password (client)
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
        Note over C,W2: Phase 1a: Session Initialisation
        C->>C: Clear session cookies
        C->>W2: POST ClaveStartingSv {device_id: "", NIF, ...}
        W2-->>C: OK / KO (ignored)
        C->>W2: POST SetCookieAppMovil (if OK and no appmovil)
    end

    rect rgb(255, 248, 240)
        Note over C,W12: Phase 1b: DNI/NIE Weak Authentication
        U->>C: Provides NIF + FECHA or SOPORTE
        C->>W2: POST AutenticaDniNieContrasteh {NIF, FECHA, SOPORTE, ...}
        W2-->>C: 302 -> www12 + Set-Cookie
        C->>W12: GET AccesoW12Sv (follow redirect)
        W12-->>C: JSON envelope or HTML + Set-Cookie: WWW12, WWW12V, sgat_id_usr_sede
    end

    rect rgb(240, 255, 240)
        Note over C,W12: Phase 1c: Registration Check + SMS
        C->>W12: POST ClaveRequestStateSv
        W12-->>C: {registrado: "true", telefono: "S", nivelRegistro}
        C->>W12: POST ObtenerClaveMovilSMS (empty body!)
        W12-->>C: {tokenClaveMovilSms, timeStampAltaSms, movil: "22230"}
        W12->>SMS: Trigger SMS delivery
        SMS->>U: SMS code received
    end

    Note over C: Serialise cookies -> JSON<br/>Wait for user to enter SMS code

    rect rgb(255, 240, 255)
        Note over C,W6: Phase 2: SMS Validation + Device Activation
        U->>C: Enters SMS code
        C->>C: Import cookies from phase 1
        C->>W12: POST ValidarClaveMovilSMS {timestamp, token, pin}
        W12-->>C: OK + Set-Cookie: pin24H, pin24V
        C->>W6: POST ClaveActivateAuthenticationSv {user_password, ...}
        W6-->>C: {device_id, nif, token}
    end

    rect rgb(248, 248, 248)
        Note over C: Session Saved
        C->>C: Save {device_id, nif, device_password} to secure storage
    end
```

## Step-by-Step HTTP Details

No API call sends `JSESSIONID`, and every stored cookie goes to every host. See [Cookie Management](../implementation-guide/cookie-management.md).

### Step 0a: Clear Session Cookies

When the app enters its identification screen, it deletes every cookie except `appmovil`, `sgat-language` and `token-exteriores` (`RemoveSessionCookiesUseCase`). llave does the same in `auth::dni_request_sms` (`LlaveClient::clear_session_cookies`). It skips the clearing on a `-8`/`-10` resubmission.

### Step 0b: ClaveStartingSv

**Purpose**: Device context, as the app sends it at launch. An app that is not yet activated has no device id and sends it **empty**.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www2.agenciatributaria.gob.es/wlpl/MOVI-P24H/ClaveStartingSv` |
| **Content-Type** | `application/x-www-form-urlencoded` |

**Form Parameters**:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `device_id` | Stored device id; **empty** before activation | `""` |
| `NIF` | User's tax ID | `09030055W` |
| `sistema_operativo` | OS identifier | `A` (Android) |
| `token_push` | Firebase token (llave: empty) | `""` |
| `version_os` | OS version | `14` |
| `version_app` | App version | `6.2.10` |
| `modelo` | Manufacturer + model | `Samsung SM-S928U` |

A `KO` response (e.g. `105`) is expected before activation, and llave ignores it here. If the response is `OK` and there is no `appmovil` cookie, the app calls **SetCookieAppMovil**:

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www2.agenciatributaria.gob.es/wlpl/MOVI-JDIT/C/SetCookieAppMovil` |
| **Content-Type** | `application/json; charset=UTF-8` |
| **Body** | `{"version_app":"6.2.10","sistema_operativo":"A","version_os":"14","modelo":"Samsung SM-S928U"}` |

The `appmovil` cookie is Base64-encoded JSON that describes the client (app, `version_app`, `sistema_operativo`, `version_os`, `modelo`).

### Step 1: AutenticaDniNieContrasteh

**Purpose**: Weak authentication with the card data. The cookies set along the redirect chain (`WWW12`, `WWW12V`, `sgat_id_usr_sede`) carry the session into the next steps.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www2.agenciatributaria.gob.es/wlpl/BUCV-JDIT/AutenticaDniNieContrasteh?ref=%2Fwlpl%2FMOVI-AEAT%2FAccesoW12Sv` |
| **Content-Type** | `application/x-www-form-urlencoded` |

**Form Parameters**:

| Parameter | Value | Description |
|-----------|-------|-------------|
| `NIF` | e.g. `09030055W` | DNI/NIE number |
| `FECHA` | `DD-MM-YYYY` or `""` | Depends on the document type (see below) |
| `SOPORTE` | e.g. `E12345678` or `""` | Depends on the document type (see below) |
| `botonAutenticacionDebil` | `Continuar` | Literal; identifies weak auth |
| `APP` | `S` | Mobile app context |
| `modo` | `json` | Asks for the JSON envelope |
| `AZUL` | `""` or `"X"` | `"X"` when resending after KO `-8` |
| `FECHANIE` | `""` or `"X"` | `"X"` when resending after KO `-10` |

**FECHA / SOPORTE by document type**: the app shows, and sends, only one field. The other is sent empty (llave: `config::required_dni_input`).

| Document | First character | `FECHA` | `SOPORTE` |
|----------|-----------------|---------|-----------|
| DNI | digit | Card **validity date** | `""` |
| DNI (special) | `K`, `L` | **Date of birth** | `""` |
| NIE | `X`, `Y`, `Z` | `""` | Card **support number** |
| NIE (special) | `M` | **Date of birth** | `""` |

**Response handling**:

1. Follow redirects manually, reusing the original `Cookie` header and capturing `Set-Cookie` at every hop. See [Redirect Handling](../implementation-guide/redirect-handling.md).
2. Parse the final body as the standard JSON envelope `{status, codigo_error, mensaje, ...}`, as the app does:
   - `OK`: authenticated.
   - `KO -8`: the server wants the date of birth. Ask for it and resend with `FECHA=<birth date>`, `SOPORTE=""`, `AZUL="X"`.
   - `KO -10`: same, but resend with `FECHANIE="X"`.
   - Other `KO`: API error.
3. If the body is **not** JSON, llave falls back to its older heuristic: zero redirects means the credentials were rejected (`DniAuthFailed`). The app treats a non-JSON body as success.

llave returns `LlaveError::DniBirthDateRequired { challenge, message }` for `-8`/`-10`. The CLI then prompts for the date of birth and calls `dni_request_sms` again with the challenge.

### Step 2: ClaveRequestStateSv

**Purpose**: Check the user's Cl@ve registration after DNI/NIE auth.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www12.agenciatributaria.gob.es/wlpl/MOVI-P24H/ClaveRequestStateSv` |
| **Content-Type** | `application/x-www-form-urlencoded` |

**Form Parameters**: `sistema_operativo=A`, `version_os=14`, `version_app=6.2.10`. No `device_id` or `NIF` is sent: the server identifies the user from the session cookies.

- **KO `987`**: the app calls `SetCookieAppMovil` and retries **once**. llave does the same. The exact meaning of `987` is undocumented; it is presumably a missing or invalid `appmovil` cookie.
- **Registration gate**: the app requests the SMS only when `registrado == "true"` **and** `telefono == "S"`. Otherwise llave fails with `NotRegistered` or `NoRegisteredPhone`. The user must register, or register a phone, with Cl@ve first.

See [ClaveRequestStateSv payload](../api-reference/request-response-formats.md#claverequeststatesv).

### Step 3: ObtenerClaveMovilSMS

**Purpose**: Send an SMS verification code to the registered phone.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www12.agenciatributaria.gob.es/wlpl/MOVI-P24H/ObtenerClaveMovilSMS` |
| **Content-Type** | **None** (empty body) |

> **Critical**: This endpoint **must** get an **empty POST body** and
> **no Content-Type header**, like Retrofit `@POST` without `@Body`. Sending
> form-encoded data (even empty) causes [error 902024](../api-reference/error-codes.md).

The response is `{movil, horaPeticion, tokenClaveMovilSms, timeStampAltaSms}`; `movil` is a masked phone number. See [ObtenerClaveMovilSMS payload](../api-reference/request-response-formats.md#obtenerclavemovilsms).

**Phase 1 ends here.** llave serialises the cookie map to JSON (`export_cookies`) so phase 2 can resume after the user types the code.

### Step 4: ValidarClaveMovilSMS

**Purpose**: Validate the SMS code. On success, the server sets `pin24H` and `pin24V`.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www12.agenciatributaria.gob.es/wlpl/MOVI-P24H/ValidarClaveMovilSMS` |
| **Content-Type** | `application/x-www-form-urlencoded` |

| Parameter | Description | Example |
|-----------|-------------|---------|
| `timeStampAltaSms` | From step 3 | `20260310151053593031` |
| `tokenClaveMovilSms` | From step 3 | `838f5824490558cc6c456e550ae7b6c2...` |
| `pinAcceso` | SMS code typed by the user | `123456` |

> The `pin24H` cookie is the gate for device activation. Without it, step 5
> returns HTML instead of JSON. The app skips this step if a `pin24H` cookie is
> already stored; llave always runs it.

### Step 5: ClaveActivateAuthenticationSv

**Purpose**: Register the device. The server assigns `device_id`.

| Property | Value |
|----------|-------|
| **Method** | `POST` |
| **URL** | `https://www6.agenciatributaria.gob.es/wlpl/MOVI-P24H/ClaveActivateAuthenticationSv` |
| **Content-Type** | `application/x-www-form-urlencoded` |

| Parameter | Description | Example |
|-----------|-------------|---------|
| `sistema_operativo` | OS identifier | `A` |
| `version_os` | OS version | `14` |
| `version_app` | App version | `6.2.10` |
| `token_push` | Firebase token (llave: empty) | `""` |
| `user_password` | Client-generated [device password](../cryptography/device-password.md): 128 uppercase hex chars | `3F9A...C01E` |
| `modelo` | Manufacturer + model | `Samsung SM-S928U` |

No `device_id` or `NIF` is sent. The cookies (`WWW12`, `WWW12V`, `pin24H`, `pin24V`, `appmovil`, `sgat-language`, `sgat_id_usr_sede`; never `JSESSIONID`) identify the user.

**Response**: `respuesta` contains **only** `device_id`, `nif` and `token`. There is no `user_password`; the password is the one the client generated. See [ClaveActivateAuthenticationSv payload](../api-reference/request-response-formats.md#claveactivateauthenticationsv).

After activation, llave (`auth::session_from_activation`):

- requires a non-empty `device_id` and fails otherwise, like the app's `checkNotNull`;
- stores the **NIF returned by the server**, falling back to the typed NIF only when the response has none;
- stores the locally generated password.

```json
{
  "device_id": "OyXoskiz6fLnzd080Xxzrpr40",
  "nif": "09030055W",
  "device_password": "3F9A…(128 uppercase hex chars)…C01E",
  "firebase_token": null,
  "created_at": "2026-03-10T14:11:16.999Z",
  "poll_cursor": null
}
```

See [Session Persistence](../implementation-guide/session-persistence.md).

## Data Flow Diagram

```mermaid
flowchart TD
    Start([Start]) --> Clear["Clear session cookies<br/>(keep appmovil, sgat-language, token-exteriores)"]
    Clear --> S0["Step 0: ClaveStartingSv (www2), device_id empty"]
    S0 -->|"OK, no appmovil"| SCA["SetCookieAppMovil (www2)"]
    S0 -->|"OK with appmovil / KO / network error"| S1
    SCA --> S1["Step 1: AutenticaDniNieContrasteh (www2)"]

    S1 -->|"KO -8 / -10"| Birth["Ask date of birth<br/>resend with AZUL / FECHANIE = X"]
    Birth --> S1
    S1 -->|"other KO, or non-JSON with 0 redirects"| Fail1([DNI/NIE auth failed])
    S1 -->|"OK"| S2["Step 2: ClaveRequestStateSv (www12)"]

    S2 -->|"KO 987"| Retry["SetCookieAppMovil + retry once"]
    Retry --> S2b{"registrado == true<br/>and telefono == S?"}
    S2 -->|"OK"| S2b
    S2b -->|"no"| Fail2([NotRegistered / NoRegisteredPhone])
    S2b -->|"yes"| S3["Step 3: ObtenerClaveMovilSMS (www12)<br/>Empty POST body"]

    S3 -->|"OK"| Wait["Show masked phone<br/>Wait for SMS code"]
    S3 -->|"KO"| Fail3([SMS request failed])

    Wait -->|"Cookies -> JSON"| Phase2["Phase 2: user enters SMS code"]
    Phase2 --> Import["Import cookies from JSON"]
    Import --> S4["Step 4: ValidarClaveMovilSMS (www12)"]

    S4 -->|"KO"| Fail4([Invalid SMS code])
    S4 -->|"OK + pin24H cookie"| GenPwd["Generate user_password<br/>SHA-512 hex of 256 random chars"]

    GenPwd --> S5["Step 5: ClaveActivateAuthenticationSv (www6)"]

    S5 -->|"KO / HTML / no device_id"| Fail5([Activation failed])
    S5 -->|"OK"| Save["Save session:<br/>device_id + nif (from server)<br/>+ device_password"]

    Save --> Done([Device Activated])

    style Fail1 fill:#fcc,stroke:#c33
    style Fail2 fill:#fcc,stroke:#c33
    style Fail3 fill:#fcc,stroke:#c33
    style Fail4 fill:#fcc,stroke:#c33
    style Fail5 fill:#fcc,stroke:#c33
    style Done fill:#cfc,stroke:#3c3
    style Wait fill:#ffc,stroke:#cc3
```

## Key Points

- The device password is generated **client-side** before activation. It is a SHA-512 uppercase hex digest, not a UUID (see [Device Password](../cryptography/device-password.md)).
- `device_id` is assigned **server-side**. Before activation, `ClaveStartingSv` gets an empty `device_id`.
- The `token` in the activation response is a session/push token, **not** the device password.
- Store the `nif` the server returns.
- Cookies must survive between phase 1 and phase 2 (see [Cookie Management](../implementation-guide/cookie-management.md)).
- CLI: `llave activate -n <NIF> -f <DD-MM-YYYY>` (DNI, DNI K/L, NIE M) or `llave activate -n <NIE> -s <support number>` (NIE X/Y/Z).
