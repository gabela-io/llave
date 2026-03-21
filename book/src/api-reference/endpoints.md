# Endpoints

All API calls use **Retrofit** with **form-urlencoded POST** (most endpoints) or **JSON body POST**.

## Base URL

```
https://www2.agenciatributaria.gob.es/
```

See [AEAT Infrastructure](../architecture/aeat-infrastructure.md) for the full subdomain map.

## Core Endpoints (www2)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `wlpl/MOVI-P24H/ClaveStartingSv` | POST (form) | App initialization, establishes device context, sets `appmovil` cookie. Returns URL whitelists/blacklists. |
| `wlpl/MOVI-P24H/ClaveIsNifActivatedSv` | POST (form) | Check if a NIF is activated in Cl@ve |
| `wlpl/MOVI-P24H/ClaveRequestPinSv` | POST (form) | Request a Cl@ve PIN (returns `pin` + `timeToLive` in seconds) |
| `wlpl/MOVI-P24H/ClaveRequestAllOperationsSv` | POST (form) | Poll for pending authentication requests. Also serves as device session validation before PIN request. |
| `wlpl/MOVI-P24H/ClaveAuthenticateSv` | POST (form) | Confirm a pending Cl@ve Movil authentication request |
| `wlpl/MOVI-P24H/ClaveCancelAuthenticateSv` | POST (form) | Reject a pending Cl@ve Movil authentication request |
| `wlpl/MOVI-P24H/ClaveDesactivateAuthSv` | POST (form) | Deactivate device authentication |
| `wlpl/MOVI-P24H/ClaveCheckMyDataSv` | POST (form) | Get user account data (email, phone, registration level) |
| `wlpl/MOVI-P24H/ClaveOperationsHistorySv` | POST (form) | Query operations history with date range and filters |
| `wlpl/MOVI-P24H/ClaveSetFirebaseTokenSv` | POST (form) | Register Firebase push notification token |
| `wlpl/MOVI-P24H/ClaveMigrationSv` | POST (form) | Migrate from old Cl@ve PIN to new version |
| `wlpl/MOVI-P24H/ClaveMovilQrSv` | POST (form) | QR code authentication (step 1 of QR flow) |
| `wlpl/MOVI-P24H/ObtenerPeticionesMarketsSv` | POST | Get pending market petitions |

## Activation & Registration Endpoints

| Endpoint | Subdomain | Method | Purpose |
|----------|-----------|--------|---------|
| `wlpl/MOVI-P24H/ClaveActivateAuthenticationSv` | **www6** | POST (form) | Activate device authentication. Requires `pin24H` cookie. **JSESSIONID must be stripped.** |
| `wlpl/MOVI-P24H/ClaveRequestStateSv` | www12 | POST (form) | Check registration state after DNI auth (registered, phone, level) |
| `wlpl/MOVI-P24H/ObtenerClaveMovil` | www12 | GET | Get Cl@ve Movil activation page |
| `wlpl/MOVI-P24H/ValidarClaveMovil` | www12 | POST (form) | Validate Cl@ve Movil token (step 2 of SSO confirm flow) |
| `wlpl/MOVI-P24H/ObtenerClaveMovilSMS` | www12 | POST (**empty body**) | Trigger SMS verification code to registered phone. **Must not send Content-Type.** |
| `wlpl/MOVI-P24H/ValidarClaveMovilSMS` | www12 | POST (form) | Validate SMS code (sets `pin24H` + `pin24V` cookies) |

## DNI/NIE Authentication

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `wlpl/BUCV-JDIT/AutenticaDniNieContrasteh?ref=%2Fwlpl%2FMOVI-AEAT%2FAccesoW12Sv` | POST (form) | Authenticate with DNI/NIE + date of birth |

## Certificate Registration

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `wlpl/MOVI-JDIT/C/IniciarAltaConCertificadoSv` | POST (JSON) | Initiate certificate-based registration |
| `wlpl/MOVI-JDIT/C/ValidarAltaConCertificadoSv` (www12) | POST (JSON) | Validate certificate registration |

## Supporting URLs (browser-based)

| URL | Purpose |
|-----|---------|
| `wlpl/MOVI-JDIT/Acceso?gestion=REGISTRO_CERTIFICADO&nif=` | Register via certificate |
| `wlpl/MOVI-JDIT/Acceso?gestion=REGISTRO_CARTA` | Register via letter |
| `wlpl/MOVI-JDIT/Acceso?gestion=MODIFICA_TELEFONO&nif=` | Change phone number |
| `wlpl/MOVI-JDIT/Acceso?gestion=OBTENER_SUPERIOR&nif=` | Get advanced registration |
| `wlpl/MOVI-JDIT/Acceso?info=AYUDA_CERT_ANDROID` | Help with Android certificates |
| `wlpl/MOVI-JDIT/Acceso?ext=OFICINAS_CLAVE` | Llave offices |
| `wlpl/MOVI-JDIT/Acceso?ext=REGISTRO_PRESENCIAL` | In-person registration |
| `wlpl/MOVI-JDIT/SoporteAppSv?APP=CLAVE` | App support |

## Endpoint Reference (Weak Auth Flow)

The following table summarises the endpoints used in the [DNI/NIE + SMS activation flow](../authentication-flows/dni-nie-sms-activation.md):

| # | Endpoint | Subdomain | Path | Method | Body | Purpose |
|---|----------|-----------|------|--------|------|---------|
| 0 | `ClaveStartingSv` | www2 | `/wlpl/MOVI-P24H/` | POST | Form | Device context + appmovil cookie |
| 1 | `AutenticaDniNieContrasteh` | www2 | `/wlpl/BUCV-JDIT/` | POST | Form | DNI/NIE weak auth (returns HTML) |
| 2 | `ClaveRequestStateSv` | www12 | `/wlpl/MOVI-P24H/` | POST | Form | Registration state check |
| 3 | `ObtenerClaveMovilSMS` | www12 | `/wlpl/MOVI-P24H/` | POST | **Empty** | Trigger SMS code |
| 4 | `ValidarClaveMovilSMS` | www12 | `/wlpl/MOVI-P24H/` | POST | Form | Validate SMS code (sets pin24H) |
| 5 | `ClaveActivateAuthenticationSv` | www6 | `/wlpl/MOVI-P24H/` | POST | Form | Device activation (no JSESSIONID!) |
