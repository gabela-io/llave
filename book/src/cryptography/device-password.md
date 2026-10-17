# Device Password

The device password (`user_password`) is generated **client-side** and saved locally before activation. The server neither generates nor returns it. The activation response contains only `device_id`, `nif` and `token`.

It is **not** a UUID.

## Algorithm (`DeviceUtils.generarUserPassword`)

```
1. Generate 256 characters with java.security.SecureRandom,
   each drawn uniformly from [0-9A-Za-z] (62 symbols)
2. SHA-512 over the UTF-8 bytes of that string
3. Encode the 64-byte digest as UPPERCASE hexadecimal
4. Result: 128-character string, e.g. "3F9A…C01E"
```

## Pseudocode

```python
import hashlib, secrets, string

charset = string.digits + string.ascii_uppercase + string.ascii_lowercase  # 0-9A-Za-z
random_string = ''.join(secrets.choice(charset) for _ in range(256))
password = hashlib.sha512(random_string.encode()).hexdigest().upper()
# 128 uppercase hex chars
```

llave: `crypto::generate_device_password()` uses the same alphabet order as `generarUserPassword` and a CSPRNG (`rand::thread_rng`). It zeroizes the intermediate random string.

## `generateTokenAuth`

`DeviceUtils.generateTokenAuth()` uses **the same algorithm**: 256 `SecureRandom` alphanumerics, then SHA-512, then uppercase hex. The only difference is the alphabet order (`A-Za-z0-9`), which does not matter for a uniform draw. The app sends the result as `tokenAuth` to `IniciarAltaConCertificadoSv` and `ValidarAltaConCertificadoSv` in [certificate registration](../authentication-flows/certificate-registration.md). llave does not implement that flow.

## Where It Is Generated

The app calls `generarUserPassword()` right before each activation path: DNI/NIE + SMS, DNIe via NFC, software certificate, VideoID and migration.

## Usage

The password goes to the server as `user_password` in `ClaveActivateAuthenticationSv`, and then in the requests that need it:

- [PIN Request](../authentication-flows/pin-request.md) (`ClaveRequestPinSv`)
- [Clave Movil](../authentication-flows/clave-movil.md) (`ClaveAuthenticateSv`; reject does **not** send it)
- [Operations History](../api-reference/request-response-formats.md#operations-history-detail) (`ClaveOperationsHistorySv`)

## Key Points

- The password is a **shared secret** between the device and the server.
- Authentication relies on `device_id` (server-generated) plus `user_password` (client-generated).
- It is kept in [secure storage](./credential-storage.md) alongside `device_id` and the NIF.
