# Android App Structure

**APK**: `es.aeat.pin24h` v6.2.5 (build 288)
**Decompiled with**: jadx 1.5.1, apktool 2.7.0

> **Naming convention**: The decompiled APK source code uses the prefix `Llave*`
> in its Retrofit interface class names (e.g. `LlaveStartingSv`), but the
> **actual HTTP endpoint URLs** on the AEAT server all use the prefix `Clave*`
> (e.g. `ClaveStartingSv`). This document uses the **server-side `Clave*` names**
> throughout, since those are the real URLs that must be called. The older
> `Llave*` variants return 404 on the public internet.

## Package Layout

The app follows Clean Architecture (presentation -> domain -> data) written in Kotlin:

```
es.aeat.pin24h/
├── data/
│   ├── webservices/     # Retrofit API client
│   │   ├── LlaveApi.java           # Retrofit interface (all API endpoints)
│   │   ├── RetrofitClient.java     # Retrofit builder + OkHttp setup
│   │   ├── SecuredHttpClient.java  # TLS configuration
│   │   ├── RedirectInterceptor.java
│   │   ├── AcceptLanguageInterceptor.java
│   │   ├── JSONInterceptor.java
│   │   └── UserAgentInterceptor.java
│   ├── repository/
│   │   └── Repository.java         # Main data source (maps use cases to API calls)
│   ├── manager/
│   │   ├── KeyChainManager.java    # Encrypted credential storage
│   │   └── CookiesManagerSingleton.java
│   └── model/
│       └── CookiesInApp.java
├── domain/
│   ├── model/
│   │   ├── request/    # All request DTOs
│   │   └── response/   # All response DTOs
│   ├── usecases/       # Business logic
│   └── interfaces/     # Repository contracts
├── presentation/       # Activities, Fragments, ViewModels
└── common/
    └── utils/
        ├── UrlUtils.java           # URL routing logic
        ├── DeviceUtils.java        # Device ID generation
        └── StringUtils.java        # Encoding helpers
```

## DNIe Library

DNIe (electronic national ID) support comes from a separate library:

```
es.gob.fnmt.dniedroid/
└── net/http/clave/
    └── ClaveAuthentication.java  # SAML-based auth via pasarela.clave.gob.es
```

This class handles the [SAML authentication flow](../authentication-flows/saml-dnie.md) using the Spanish DNIe card via NFC.

## Key Classes

| Class | Layer | Responsibility |
|-------|-------|---------------|
| `LlaveApi.java` | Data | Retrofit interface declaring all [API endpoints](../api-reference/endpoints.md) |
| `RetrofitClient.java` | Data | Builds the OkHttp + Retrofit client stack |
| `SecuredHttpClient.java` | Data | TLS version/cipher suite configuration (see [Constants](../api-reference/constants.md)) |
| `Repository.java` | Data | Maps domain use cases to HTTP calls |
| `KeyChainManager.java` | Data | [Credential storage](../cryptography/credential-storage.md) and [browser encryption](../cryptography/browser-encryption.md) |
| `CookiesManagerSingleton.java` | Data | [Cookie management](../implementation-guide/cookie-management.md) singleton |
| `DeviceUtils.java` | Common | [Device password](../cryptography/device-password.md) generation |
| `ClaveAuthentication.java` | DNIe lib | [SAML/DNIe](../authentication-flows/saml-dnie.md) form-based authentication |
