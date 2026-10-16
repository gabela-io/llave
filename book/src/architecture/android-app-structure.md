# Android App Structure

**App**: `es.aeat.pin24h` v6.2.10 (build 294)
**SDK**: minSdk 28 (Android 9), targetSdk 37, compileSdk 37
**Decompiled with**: jadx and apktool

The Retrofit interface is `ClaveApi`, and every endpoint uses the server-side `Clave*` names (e.g. `ClaveStartingSv`).

## Builds and Protection

| Build | Source | Protection | Use for analysis |
|-------|--------|------------|------------------|
| Google Play (split `base.apk`) | Play Store split APKs | **PairIP**: `com/pairip/*`, `libpairipcore.so` and encrypted assets. Code runs in a VM and strings are decrypted at runtime, so most literals are unreadable statically. | Hard |
| Universal APK | Single APK signed by AEAT | No PairIP; strings in clear | Preferred |

Both builds ship `libtoolChecker.so`, a native root/integrity check.

Other native libraries in the universal APK:

| Library | Purpose |
|---------|---------|
| `libbarhopper_v3.so` | ML Kit barcode scanning (QR) |
| `libjingle_peerconnection_so.so`, `libx264encoder.so`, `libyuv*.so` | VideoID (WebRTC video) |
| `libdatastore_shared_counter.so` | Jetpack DataStore |
| `libimage_processing_util_jni.so`, `libsurface_util_jni.so` | CameraX |

## Package Layout

Clean Architecture (presentation → domain → data) in Kotlin, with Hilt for dependency injection:

```
es.aeat.pin24h/
├── ClaveApplication.java
├── data/
│   ├── webservices/
│   │   ├── ClaveApi.java                 # Retrofit interface (all API endpoints)
│   │   ├── RetrofitClient.java           # Retrofit + OkHttp setup
│   │   ├── SecuredHttpClient.java        # TLS, timeouts
│   │   ├── RedirectInterceptor.java      # Set-Cookie capture across redirects
│   │   ├── AcceptLanguageInterceptor.java
│   │   ├── JSONInterceptor.java          # Accept: application/json
│   │   └── UserAgentInterceptor.java
│   ├── repository/
│   │   └── Repository.java               # Use cases → HTTP calls, cookie map
│   └── manager/
│       ├── KeyChainManager.java          # Credential storage (Tink AEAD + DataStore)
│       ├── TinkProviderSingleton.java
│       ├── ClaveEncryptedSharedPreferencesSingleton.java  # Legacy store (migration source)
│       ├── PreferencesManager.java
│       └── FirebaseManager.java
├── domain/
│   ├── model/
│   │   ├── request/                      # Request DTOs
│   │   └── response/                     # Response DTOs
│   ├── usecases/
│   └── interfaces/
├── presentation/                         # Activities, Fragments, ViewModels
└── common/
    └── utils/
        ├── UrlUtils.java                 # URL routing
        ├── DeviceUtils.java              # User-Agent, modelo, user_password
        ├── CookiesUtils.java
        ├── DateUtils.java
        └── NifUtils.java
```

## DNIe Library

DNIe (electronic national ID) support comes from a separate library:

```
es.gob.fnmt.dniedroid/
└── net/http/clave/
    └── ClaveAuthentication.java  # SAML-based auth via pasarela.clave.gob.es
```

This class handles the [SAML authentication flow](../authentication-flows/saml-dnie.md) using the DNIe card over NFC.

## Key Classes

| Class | Layer | Responsibility |
|-------|-------|---------------|
| `ClaveApi.java` | Data | Retrofit interface declaring all [API endpoints](../api-reference/endpoints.md) (including two dead methods) |
| `RetrofitClient.java` | Data | Builds the OkHttp + Retrofit client stack |
| `SecuredHttpClient.java` | Data | TLS versions, cipher suites and timeouts (see [Constants](../api-reference/constants.md)) |
| `Repository.java` | Data | Maps use cases to HTTP calls; builds the `Cookie` header (`recuperarCookies`, never `JSESSIONID` on API calls) and stores `Set-Cookie` values |
| `KeyChainManager.java` | Data | Secure storage of `nif_usuario`, `id_dispositivo`, `id_firebase`, `token`, `user_password`, `map_cookies` with Google Tink AEAD (see [Credential Storage](../cryptography/credential-storage.md)) |
| `DeviceUtils.java` | Common | User-Agent, `modelo` and [device password](../cryptography/device-password.md) generation |
| `ClaveDeviceActiveFragment` / `ViewModel` | Presentation | Main screen: polling, PIN reveal, confirm/reject, QR, error-code dialogs |
| `ClaveAuthentication.java` | DNIe lib | [SAML/DNIe](../authentication-flows/saml-dnie.md) form-based authentication |

6.2.10 contains no browser-handoff encryption (no `javax.crypto` use in the app package); its only own crypto primitive is SHA-512 in `DeviceUtils`.
