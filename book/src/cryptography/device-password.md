# Device Password

The device password is generated **client-side** and saved locally before the activation API call. The server does not generate or return the password.

## Algorithm (from Android APK)

```
1. Generate 256 random alphanumeric characters (charset: 0-9, A-Z, a-z)
2. Compute SHA-512 hash of the random string
3. Convert hash to uppercase hexadecimal
4. Result: 128-character uppercase hex string
```

## Pseudocode

```python
import hashlib, random, string

charset = string.digits + string.ascii_letters  # 0-9, A-Z, a-z
random_string = ''.join(random.choice(charset) for _ in range(256))
password = hashlib.sha512(random_string.encode()).hexdigest().upper()
# Result: 128 chars, e.g. "A1B2C3D4E5F6..."
```

## Usage

The password is sent to the server as the `user_password` field during `ClaveActivateAuthenticationSv` and on every subsequent authenticated request:

- [PIN Request](../authentication-flows/pin-request.md) (`ClaveRequestPinSv`)
- [Clave Movil](../authentication-flows/clave-movil.md) (`ClaveAuthenticateSv`)
- [Operations History](../api-reference/request-response-formats.md#operations-history-detail) (`ClaveOperationsHistorySv`)

## Key Points

- The password acts as a **shared secret** between the device and the server.
- Authentication is tied to `device_id` (server-generated) + `device_password` (client-generated). Both are required for every authenticated API call.
- The password is stored in [secure storage](./credential-storage.md) alongside the `device_id` and NIF.
