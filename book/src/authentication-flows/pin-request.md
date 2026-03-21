# PIN Request

The full PIN request sequence mirrors the original app's startup behaviour when the device is already activated. The app calls three endpoints in order.

## Flow

```
1. ClaveStartingSv  ->  Establish device session (device_id, NIF, OS info)
                       Sets appmovil cookie.
                       Response: { status: "OK"|"KO", respuesta: { listaBlancaUrls, listaNegraUrls } }
                       KO with code 105 is normal for unregistered devices -- continue anyway.

2. ClaveRequestAllOperationsSv  ->  Validate device registration + poll for pending requests.
                                    Fields: device_id, NIF, timestamp, OS info
                                    The timestamp acts as a cursor:
                                      - First call: "19700101T000000000Z" (epoch)
                                      - Subsequent: timestamp from previous response
                                    Response: { status: "OK"|"KO", respuesta: { pin, peticion: { timestamp, ... } } }
                                    Error 105/109 -> device not registered (re-activation required).
                                    Error 205 -> no pending operations (normal).

3. ClaveRequestPinSv  ->  Request a temporary PIN.
                          Fields: device_id, user_password, NIF, OS info
                          Response: { status: "OK", respuesta: { pin: "123456", timeToLive: "180" } }
                          Error 307 -> "No tiene disponible ningun PIN" (no PIN available).
```

The PIN is a server-generated one-time code. `timeToLive` is in seconds (typically 180 = 3 minutes).

> **Important**: Step 2 (`ClaveRequestAllOperationsSv`) is called by the original
> app when the "device active" screen loads. Skipping it may cause step 3 to fail
> with [error 307](../api-reference/error-codes.md). It appears to serve as a server-side session validation step.

## Key Points

- PINs are **server-generated** via `ClaveRequestPinSv` -- the app does NOT generate OTPs locally (no TOTP/HOTP).
- Authentication is tied to `device_id` (server-generated) + `device_password` (client-generated SHA-512 hash). See [Device Password](../cryptography/device-password.md).
- The session validation step (step 2) must not be skipped, even if there are no pending operations.
