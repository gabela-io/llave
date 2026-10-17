# Credential Storage

## Android App (Cl@ve 6.2.10)

`KeyChainManager` encrypts every value with **Google Tink AEAD** and stores it in a Jetpack **DataStore** (Preferences) named `ClaveSecureDataStore`.

```
Value on disk = Base64(NO_WRAP)( Aead.encrypt(utf8(value), associatedData = null) )
```

| Item | Value |
|------|-------|
| AEAD key template | `AeadKeyTemplates.AES256_GCM` |
| Keyset | `clave_master_keyset` in SharedPreferences file `clave_tink_key_prefs` (`AndroidKeysetManager`) |
| Keyset wrapping key | Android Keystore, URI `android-keystore://clave_secure_master_key` |
| Provider | `TinkProviderSingleton.getAead(context)` |
| DataStore | `ClaveSecureDataStore` |

### Stored Keys

| Key | Description |
|-----|-------------|
| `nif_usuario` | User's NIF/NIE (the value returned by activation) |
| `id_dispositivo` | Server-assigned device ID |
| `user_password` | Client-generated [device password](./device-password.md) |
| `id_firebase` | FCM push token |
| `token` | Token from the activation response |
| `map_cookies` | Cookie map as JSON: cookie name → JSON-serialised cookie (`AeatCookie`) |

The poll cursor (`PreferencesManager.saveUltimaPeticionClaveMovil`) and other non-secret settings live in `PreferencesManager`, not in this store.

### Migration from EncryptedSharedPreferences

Older versions kept the same keys in an AndroidX `EncryptedSharedPreferences` file named `Clave`: a `MasterKey`, AES256-SIV for keys and AES256-GCM for values. `KeyChainManager.migrateToSecureDataStore()` copies `nif_usuario`, `id_dispositivo`, `id_firebase`, `token`, `user_password` and `map_cookies` into `ClaveSecureDataStore`. It then clears the legacy `Clave`, `ClaveAuthentication` and `ClaveCertificados` preference files.

The older `LlavePinAlias` AES-GCM key and the `LlavePin` / `LlaveCertificados` files described in earlier versions of this book do not exist in 6.2.10.

## llave

llave does not copy the app's storage. It stores one JSON [session](../implementation-guide/session-persistence.md) (`device_id`, `nif`, `device_password`, `created_at`, `poll_cursor`):

| Client | Storage |
|--------|---------|
| CLI | **PIN-sealed** file `$XDG_DATA_HOME/llave-cli/session.sealed` (`SealedFileStorage`, mode 0600), with the unlocked session cached in the kernel keyring (`KeyringStorage`, service `llave-cli`, account `session`) for the current login. The cache alone would be lost on reboot |
| Flutter | Rust holds the session in memory (`MemoryStorage`). Flutter persists a **PIN-sealed** blob in `flutter_secure_storage` (key `llave_session`) |

### PIN Sealing (CLI and Flutter)

`crypto::seal_session` / `open_session`:

```
key      = Argon2id(PIN, salt)           (argon2 crate defaults: 19 MiB, 2 iterations, 1 lane)
envelope = base64( version=1 || salt(16) || nonce(12) || AES-256-GCM(session_json) )
```

A wrong PIN fails GCM authentication. Because the blob is already encrypted, the Linux fallback (`~/.local/share/llave/session.enc`, mode 0600, used when libsecret is unavailable) does not weaken it.

llave persists no cookies: every CLI run starts with a fresh cookie map and requests `appmovil` again.
