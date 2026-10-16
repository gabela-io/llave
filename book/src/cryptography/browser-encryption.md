# Browser Encryption (Removed)

> **Historical note.** Earlier analyses of the Cl@ve app described an
> AES-CBC/PBKDF2 scheme (`KeyChainManager.encryptTextForBrowser()`) for passing
> device credentials to a browser. **It no longer exists in Cl@ve 6.2.10
> (build 294)**, and llave has removed its implementation.

## What Changed in 6.2.10

- The app code contains no `encryptTextForBrowser`, `javax.crypto`, `Cipher`, `SecretKeySpec` or PBKDF2. Its only hand-written cryptographic primitive is `MessageDigest` SHA-512, used for the [device password](./device-password.md).
- `PreferencesManager` still has `save/getServerPassphrase`, `save/getServerIv` and `save/getServerSalt`, but nothing calls them. They are dead leftovers.
- The plaintext payload format (`$id_dispositivo:…$password_dispositivo:…$url_destino:…`) appears nowhere in the app.
- Local secrets are now protected by Google Tink (see [Credential Storage](./credential-storage.md)), not by this scheme.

## What It Used to Be

From older app versions; it could not be re-verified against 6.2.10:

```
Algorithm:      AES/CBC/PKCS5PADDING
Key derivation: PBKDF2WithHmacSHA1, 1000 iterations, 128-bit key
                passphrase / salt / IV supplied by the server
Output:         Base64, then URI-encoded
```

## llave

`crypto::encrypt_for_browser` and `build_browser_payload` had no callers and have been deleted, together with their dependencies (`aes`, `cbc`, `pbkdf2`, `sha1`, `hmac`). llave's own session encryption (Argon2id + AES-256-GCM) is unrelated; see [Credential Storage](./credential-storage.md#llave).

For the DNIe SAML path, see [SAML / DNIe Authentication](../authentication-flows/saml-dnie.md).
