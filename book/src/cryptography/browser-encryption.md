# Browser Encryption

The app encrypts data for browser handoff using AES-CBC with PBKDF2 key derivation. This is used when the app needs to pass authentication context to a browser session.

## Algorithm

Found in `KeyChainManager.encryptTextForBrowser()`:

```
Algorithm: AES/CBC/PKCS5PADDING
Key derivation: PBKDF2WithHmacSHA1
  - Passphrase: serverPassphrase (received from server)
  - Salt: serverSalt (hex string, received from server)
  - Iterations: 1000
  - Key length: 128 bits
IV: serverIv (hex string, received from server)
Output: Base64 -> URI-encoded
```

## Encrypted Payload Format

```
$id_dispositivo:<device_id>
$password_dispositivo:<device_password>
$url_destino:<target_url>
$idioma:<lang>_ES
[$NIF:<nif>]
[$datoContraste:<contrast_data>$tipoAutenticacion:<auth_type>]
[$codigo:<code>$pin:<pin>]
```

Fields in brackets are optional and included depending on the context.

## Key Points

- All encryption parameters (`serverPassphrase`, `serverIv`, `serverSalt`) come from the server, making this a server-controlled scheme.
- The output is Base64-encoded, then URI-encoded for transmission.
- This is only used for browser handoff scenarios -- not for general API communication.

## SAML Form-Based Flow

The DNIe path through `pasarela.clave.gob.es` uses standard SAML redirects:
- No custom crypto beyond TLS mutual auth with the DNIe certificate
- The eID card provides the `SSLSocketFactory` for client certificate authentication
- Forms contain `SAMLResponse` fields for assertion relay

See [SAML / DNIe Authentication](../authentication-flows/saml-dnie.md) for the full flow.
