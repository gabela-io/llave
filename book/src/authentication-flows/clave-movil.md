# Clave Movil Authentication

When a citizen logs in to a government service with Cl@ve Móvil, the server creates a **pending request** for the device. The device finds it by [polling](./pending-operations-polling.md) or after scanning a [QR code](./qr-code.md). The user then confirms or rejects it.

A request is only actionable when `respuesta.peticion.tokenClaveMovil` is set:

| `peticion` field | Use |
|------------------|-----|
| `tokenClaveMovil` | Identifies the request; required for confirm/reject |
| `codigoIdP` | Verification code; must match the code shown on the website |
| `organismo` | Requesting organisation (e.g. AEAT, Seguridad Social) |
| `tipo` | Request type (the app's UI tells apart DNI/NIE-started and QR-started requests) |
| `timestamp` | `yyyyMMdd'T'HHmmssSSS'Z'`; also the [poll cursor](./pending-operations-polling.md#timestamp-cursor) |
| `timeToLive` | Remaining validity |

## Security Check Before Confirming

The app never confirms blindly. Its confirmation dialog shows the organisation and the verification code (`codigoIdP`), then asks whether the user started this login on the official website and not from a link in an email or SMS:

- **Yes**: continue with the confirm flow, after biometric unlock in the app.
- **No**: the request is **rejected** (`ClaveCancelAuthenticateSv`).

llave's CLI does the same in `confirm` and `qr`. It prints the verification code and asks; answering "no" rejects the request. `--yes` skips the prompt.

## Confirm Flow

```
1. ClaveAuthenticateSv  ->  Confirm the request (www2)
   Fields: device_id, user_password, NIF, tokenClaveMovil, codigoIdP, OS info
   Response: { status: "OK", respuesta: { pending_requests: "0" } }

2. ValidarClaveMovil    ->  www12, ONLY if a WWW12 cookie is stored
   Fields: tokenClaveMovil
```

How the app uses step 2:

- **Confirming an external request** (the device-active screen, `confirmarPeticionAeat`): the app sends `ClaveAuthenticateSv` only.
- **Its own [www12 web login](./token-activation.md)** (`confirmarPeticionAutenticacionAndContinue`): the app adds `ValidarClaveMovil` when the `WWW12` cookie exists.

llave's `auth::confirm_authentication` mirrors `confirmarPeticionAutenticacionAndContinue`. Without a `WWW12` cookie, the normal CLI case, it reduces to step 1. A failed `ValidarClaveMovil` is logged and is not fatal.

> **Critical**: Do NOT call `ClaveStartingSv` before `ClaveAuthenticateSv`.
> llave observed that it resets server-side state and causes [error 205](../api-reference/error-codes.md) ("datos de operación no correctos").

## Reject Flow

```
1. ClaveCancelAuthenticateSv  ->  Reject the request (www2)
   Fields: device_id, NIF, tokenClaveMovil, codigoIdP, OS info   (no user_password)
```

> Same caveat: do not call `ClaveStartingSv` before rejecting.

`ClaveCancelAuthenticateSv` is not `ClaveCancelSv`. In the app, `ClaveCancelSv` means "Renunciar a Cl@ve": it deregisters the user from Cl@ve entirely. llave does not implement it.

## CLI

```
llave listen               # poll until a request arrives
llave confirm -t <tokenClaveMovil> -i <codigoIdP>   # security check, then confirm
llave reject  -t <tokenClaveMovil> -i <codigoIdP>
```
