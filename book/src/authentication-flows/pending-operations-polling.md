# Pending Operations Polling

The app polls for pending authentication requests using `ClaveRequestAllOperationsSv`.

## Flow

```
1. ClaveStartingSv  ->  Establish session (called once at app startup)
2. ClaveRequestAllOperationsSv  ->  Poll with timestamp cursor
   - timestamp = "19700101T000000000Z" for first poll
   - timestamp = peticion.timestamp from previous response for subsequent polls
   - Error 205 = no pending operations (keep polling)
   - OK with peticion data = pending request found
     -> Extract tokenClaveMovil + codigoIdP from peticion
     -> Call ClaveAuthenticateSv or ClaveCancelAuthenticateSv (see Clave Movil flow)
```

## Timestamp Cursor

The `timestamp` field acts as a cursor for pagination:

- **First call**: Use the epoch value `"19700101T000000000Z"`
- **Subsequent calls**: Use the `timestamp` value from the previous response's `peticion` object

## Response Handling

| Response | Meaning | Action |
|----------|---------|--------|
| `status: "KO"`, `codigo_error: "205"` | No pending operations | Keep polling |
| `status: "KO"`, `codigo_error: "105"` or `"109"` | Device not registered | Re-activation required |
| `status: "OK"` with `peticion` data | Pending request found | Extract `tokenClaveMovil` + `codigoIdP`, then [confirm or reject](./clave-movil.md) |
