# Credential Storage

The Android app uses multiple storage mechanisms for credentials, with an Android Keystore-backed key for encryption.

## Local Credential Encryption (AES-GCM)

Found in `KeyChainManager.getValueLlavePin()`:

```
Algorithm: AES/GCM/NoPadding
Key: AndroidKeyStore "LlavePinAlias" (AES, 256-bit)
IV: Randomly generated, stored in preferences (Base64-encoded)
GCM Tag Length: 128 bits
```

## Android Keystore

- Key alias: `"LlavePinAlias"`
- Algorithm: AES
- Block mode: GCM
- Padding: NoPadding
- `setRandomizedEncryptionRequired(false)` -- allows reuse of IV

## Encrypted Shared Preferences

Uses Android's `EncryptedSharedPreferences` for storing:
- `nif_usuario` -- User's NIF
- `id_dispositivo` -- Device ID
- `user_password` -- Device password
- `id_firebase` -- Firebase token
- `token` -- Session token
- `map_cookies` -- Serialized cookie jar

## SharedPreferences Files

| Preference File | Purpose |
|-----------------|---------|
| `LlavePin` | Legacy encrypted credential storage |
| `ClaveAuthentication` | Authentication passwords (migrated) |
| `LlaveCertificados` | Software certificate storage |
| EncryptedSharedPreferences (default) | Primary secure storage |

## Stored Credentials

| Key | Description |
|-----|-------------|
| `nif_usuario` | User's NIF/NIE |
| `id_dispositivo` | Unique device ID |
| `user_password` | Device-specific password (for API auth) |
| `id_firebase` | FCM push token |
| `token` | Session token |
| `password_dispositivo` | Device password (legacy) |
| `dato_contraste_usuario` | Contrast data for verification |
| `map_cookies` | Serialized cookies (JSON HashMap) |
| `user_certificates` | Imported software certificates |

## Software Certificates

The app can use software certificates stored in Android KeyChain:
- Stored in `EncryptedSharedPreferences` under `"user_certificates"`
- Can be imported from `.p12`/`.pfx` files
- Used for [certificate-based registration flows](../authentication-flows/certificate-registration.md)

## Client Certificates (DNIe via NFC)

The app supports reading the Spanish DNIe (electronic national ID) via NFC:

```java
// CertificadoNfcData provides SSLSocketFactory for mutual TLS
certificadoNfcData.getDnieSSLSocketFactory()
```

This is used in `RetrofitClient.createHttpClientWithCertificate()` to create an OkHttp client that presents the eID certificate during the TLS handshake.
