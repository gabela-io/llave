# Certificate Registration

Registration using a software certificate (`.p12`/`.pfx` file) or hardware certificate (DNIe via NFC).

## Flow

```
1. IniciarAltaConCertificadoSv  ->  POST JSON body
2. ValidarAltaConCertificadoSv  ->  POST JSON body (www12)
```

These endpoints use **JSON request bodies** (not form-urlencoded), unlike most other endpoints.

## Client Certificates

### DNIe via NFC

The app supports reading the Spanish DNIe (electronic national ID) via NFC:

```java
// CertificadoNfcData provides SSLSocketFactory for mutual TLS
certificadoNfcData.getDnieSSLSocketFactory()
```

This is used in `RetrofitClient.createHttpClientWithCertificate()` to create an OkHttp client that presents the eID certificate during the TLS handshake.

### Software Certificates

The app can also use software certificates stored in Android KeyChain:
- Stored in `EncryptedSharedPreferences` under `"user_certificates"`
- Can be imported from `.p12`/`.pfx` files
- Used for certificate-based registration flows

See [Credential Storage](../cryptography/credential-storage.md) for details on how certificates are stored.
