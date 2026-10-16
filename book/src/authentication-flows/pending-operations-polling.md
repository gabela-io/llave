# Pending Operations Polling

Pending Cl@ve Móvil requests (and pending PINs) are fetched with `ClaveRequestAllOperationsSv` (www2).

## Push Is Only a Wake-Up

The app receives Firebase push messages. Their data holds only display fields (`titulo`, `mensaje`, `tipoNotificacion`, `origen`, `url`) and **never the request itself**: no `tokenClaveMovil`, no `codigoIdP`. A push only prompts the app to call `ClaveRequestAllOperationsSv`. The app makes the same call when its device-active screen loads (`consultarClaveMovilOPin`).

llave registers no push token (`token_push` is empty), so it polls.

## Flow

```
ClaveRequestAllOperationsSv  (www2)
  Fields: device_id, NIF, timestamp (cursor), sistema_operativo, version_os, version_app

  OK -> respuesta: { peticion: {tokenClaveMovil, codigoIdP, organismo, tipo, timestamp, timeToLive} | null,
                     pin: {timeToLive, timestamp} | null,
                     timestamp, timeToLive }
        - peticion.timestamp older than the cursor -> stale, ignore it
        - peticion.tokenClaveMovil present         -> pending request: show the security check, then
                                                      confirm or reject (see Clave Movil flow)
        - otherwise                                 -> nothing to do (a `pin` alone is not a request)
  KO -> see error codes below
```

Do **not** call `ClaveStartingSv` between polling and confirm/reject (see [Clave Movil](./clave-movil.md)).

## Timestamp Cursor

The `timestamp` field is a cursor in the format `yyyyMMdd'T'HHmmssSSS'Z'`:

| | App 6.2.10 | llave |
|---|---|---|
| Source | `respuesta.peticion.timestamp` only (not the top-level `timestamp`) | Same |
| Storage | Persisted in its DataStore (`saveUltimaPeticionClaveMovil`) | Persisted in [`Session.poll_cursor`](../implementation-guide/session-persistence.md); also kept in memory for the current process |
| First value | `19700201T<current time>Z` | `19700101T000000000Z` |
| Stale filter | Shows a request only if `cursor <= peticion.timestamp` | `drop_stale_peticion`: a `peticion` older than the cursor is replaced by `null` |
| Updated by | AllOperations and `ClaveMovilQrSv` responses | AllOperations (`poll_pending_requests`, `request_pin`) and `qr_authenticate` |

Both first values are earlier than any real request, so the difference is irrelevant. Timestamps have a fixed width, so plain string comparison orders them chronologically.

## Response Handling

| Response | Meaning | Action |
|----------|---------|--------|
| `OK`, `peticion.tokenClaveMovil` set (and not stale) | Pending request | Security check, then [confirm or reject](./clave-movil.md) |
| `OK`, no `peticion` / stale `peticion` | Nothing pending | Keep polling |
| `KO` `40` | No request (the app's "no request" screen) | Keep polling |
| `KO` `205` | "Datos de operación no correctos". The app treats it as an error, but llave observed it when polling with nothing pending | Keep polling (llave) |
| `KO` `407` | Server asks to retry in a few seconds | Keep polling |
| `KO` `105` / `109` | Device revoked / not registered | Delete local session; re-activation required |
| `KO` `416` | Device not active | Stop; keep local session |

## llave

- `auth::poll_pending_requests` makes a single poll (CLI: `llave pending`).
- `auth::listen_for_requests` loops every `interval` seconds until a request with a `tokenClaveMovil` arrives (CLI: `llave listen -i 5 -m 60`; `-m 0` = unlimited). Codes `40`, `205` and `407` mean "keep polling"; any other error stops the loop.
- `auth::next_pending_request` makes one poll and returns the pending `peticion`, or nothing. The keep-polling codes count as nothing; any other error is returned.
- The Flutter app has a **waiting for a login** mode instead of push. The user starts a login on the website, then opens *Pending requests*. The screen calls `next_pending_request` every 5 seconds for up to 2 minutes. When a request arrives it shows the security check (organismo, verification code, "did you start this?"). Like the app, after a confirm **or** a reject it waits for the next request unless `respuesta.pending_requests` is `"N"`; a missing value counts as `"S"` (`ClaveDeviceActiveFragment`). The core exposes this as `more_pending` in both results.
