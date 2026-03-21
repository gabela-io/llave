# Constants

Hardcoded values, URLs, headers, and build flags extracted from the Android APK.

## URLs

```
Base URL:                    https://www2.agenciatributaria.gob.es/
Llave Gateway:               https://pasarela.clave.gob.es/
Llave Gateway SP:            https://pasarela.clave.gob.es/Proxy2/ServiceProvider
AEAT Sede:                   https://sede.agenciatributaria.gob.es/
```

## HTTP Headers

```
TrazasApp: <trace_id>        (custom header for server-side tracing)
Referer: https://pasarela.clave.gob.es/  (for SAML flows)
Accept-Language: <lang>_ES or <lang>_GB
Content-Type: application/json (for JSON endpoints)
User-Agent: <custom_user_agent>  (from DeviceUtils)
```

See [Client Identity](../implementation-guide/client-identity.md) for the full User-Agent format and TrazasApp structure.

## App Constants

```
APP field value: "S"                           (indicates mobile app context)
botonAutenticacionDebil: "Continuar"           (submit button for weak auth)
sistema_operativo: "A"                         (Android)
SAML IdP hint: "AFIRMA"
EPOCH_TIMESTAMP: "19700101T000000000Z"         (initial polling cursor)
```

## Build Config Flags

```
IGNORE_SECURE_HTTP: Boolean (debug vs release TLS behavior)
HTTP_LOG_LEVEL: HttpLoggingInterceptor.Level
```

## TLS Configuration

Found in `SecuredHttpClient`:

```java
// Supported TLS versions
TLS 1.2, TLS 1.3

// Cipher suites
TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
TLS_AES_128_GCM_SHA256
TLS_AES_256_GCM_SHA384
TLS_CHACHA20_POLY1305_SHA256

// Timeouts
Connect: 190 seconds
Read: 190 seconds
Write: 190 seconds

// Certificate pinning: NONE
// The app does NOT implement certificate pinning!
// In debug mode (IGNORE_SECURE_HTTP=true), it trusts all certificates
```
