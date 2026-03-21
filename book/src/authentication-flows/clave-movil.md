# Clave Movil Authentication

When a pending authentication request arrives (via push notification or polling), the user can confirm or reject it.

## Confirm Flow

```
1. ClaveAuthenticateSv  ->  Confirm authentication (www2)
   Fields: device_id, user_password, NIF, tokenClaveMovil, codigoIdP
   Response: { status: "OK", respuesta: { pending_requests: "0" } }

2. ValidarClaveMovil  ->  Complete SSO session (www12) -- ONLY if WWW12 cookie is present
   Fields: tokenClaveMovil
   This step finalises the service provider's login redirect (e.g. Seguridad Social).
```

> **Critical**: Do NOT call `ClaveStartingSv` before `ClaveAuthenticateSv`.
> Calling it resets server-side session state and causes [error 205](../api-reference/error-codes.md) ("datos no correctos").

## Reject Flow

```
1. ClaveCancelAuthenticateSv  ->  Reject authentication (www2)
   Fields: device_id, NIF, tokenClaveMovil, codigoIdP
```

> Same caveat: do NOT call `ClaveStartingSv` before reject -- it causes [error 205](../api-reference/error-codes.md).

## Two-Step Confirm

Confirming a Cl@ve Movil request is two steps:

1. `ClaveAuthenticateSv` (www2)
2. `ValidarClaveMovil` (www12, only if `WWW12` cookie is present)

Calling `ClaveStartingSv` before either step resets server state and causes error 205.
