# Constants

Hardcoded values, URLs, headers, and build flags from the official app v6.2.10 (build 294), and the values llave sends.

## URLs

```
Base URL (www2):             https://www2.agenciatributaria.gob.es/
Activation (www6):           https://www6.agenciatributaria.gob.es/
Registration/SMS (www12):    https://www12.agenciatributaria.gob.es/
Cl@ve gateway:               https://pasarela.clave.gob.es/
Cl@ve gateway SP:            https://pasarela.clave.gob.es/Proxy2/ServiceProvider
AEAT Sede:                   https://sede.agenciatributaria.gob.es/
QR deep link:                https://sede.agenciatributaria.gob.es/appclave/qrscan?qrcode=<value>
```

## App Identity

| Constant | App 6.2.10 | llave | Source |
|----------|------------|-------|--------|
| `version_app` | `6.2.10` | `6.2.10` | `DeviceUtils.getVersionApp()` |
| User-Agent version | `v6.2.10(294)` | `v6.2.10(294)` | `DeviceUtils.getVersionAppForUserAgent()` |
| `sistema_operativo` | `A` (Android) | `A` | `DeviceUtils.getSoReduced()` |
| `version_os` | `Build.VERSION.RELEASE` | `14` | |
| `modelo` | Capitalised `Build.MANUFACTURER` + `" "` + `Build.MODEL` (only the capitalised model if it already starts with the manufacturer) | `Samsung SM-S928U` | `DeviceUtils.getDeviceName()` |
| `token_push` | Firebase installation id | `""` | |

User-Agent:

```
APPMovil/Cl@ve/v6.2.10(294)/<Build.VERSION.RELEASE>/Android/<System.getProperty("http.agent")>
```

llave sends:

```
APPMovil/Cl@ve/v6.2.10(294)/14/Android/Dalvik/2.1.0 (Linux; U; Android 14; SM-S928U Build/UP1A.231005.007)
```

The Dalvik suffix uses the bare `Build.MODEL`, not `modelo`. See [Client Identity](../implementation-guide/client-identity.md).

## HTTP Headers

The app adds these through OkHttp interceptors, plus an explicit `Cookie` header per call:

```
User-Agent: APPMovil/Cl@ve/v6.2.10(294)/...      (UserAgentInterceptor)
Accept: application/json                          (JSONInterceptor)
Accept-Language: es_ES                            (AcceptLanguageInterceptor; app language)
Content-Type: application/json; charset=UTF-8     (JSON-body endpoints only)
If-Modified-Since: <date>                         (carousel download only)
Referer: https://pasarela.clave.gob.es/           (SAML/DNIe library only)
```

There is **no `TrazasApp` header** in 6.2.10, and llave no longer sends one.

## App Constants

```
APP field value: "S"                           (DNI/NIE auth: mobile app context)
modo: "json"                                   (DNI/NIE auth: JSON response)
botonAutenticacionDebil: "Continuar"           (DNI/NIE auth submit button)
AZUL: "X" on retry after KO -8, else ""
FECHANIE: "X" on retry after KO -10, else ""
SAML IdP hint: "AFIRMA"                        (DNIe library)
EPOCH_TIMESTAMP: "19700101T000000000Z"         (llave's first polling cursor)
Timestamp format: yyyyMMdd'T'HHmmssSSS'Z'
History date format: dd/MM/yyyy
History default range: last 7 days
Persistent cookies: appmovil, sgat-language, token-exteriores
```

## Build Config Flags

```
IGNORE_SECURE_HTTP = false                     (release; true trusts all certificates)
HTTP_LOG_LEVEL = HttpLoggingInterceptor.Level.NONE
```

## TLS Configuration

Found in `SecuredHttpClient`:

```
// Supported TLS versions
TLS 1.2, TLS 1.3

// Cipher suites
TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
TLS_AES_128_GCM_SHA256
TLS_AES_256_GCM_SHA384
TLS_CHACHA20_POLY1305_SHA256

// Timeouts
Connect: 30 seconds
Read: 190 seconds
Write: 190 seconds

// Redirects: OkHttp default, at most 20 hops
// Certificate pinning: none
```

llave mirrors this: TLS 1.2 minimum, 30 s connect timeout, 190 s overall timeout, at most 20 redirects.
