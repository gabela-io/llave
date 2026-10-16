# Redirect Handling

The DNI/NIE authentication step has a redirect chain that crosses subdomains (www2 → www12). llave turns off reqwest's automatic redirects (`Policy::none()`) and follows each hop by hand, **the way OkHttp does in the app**.

## Why Manual Redirect Following Is Required

1. The app sends an explicit `Cookie` header built from its cookie map. OkHttp's follow-up logic (`RetryAndFollowUpInterceptor`) copies that header to every redirected request, even across hosts.
2. reqwest's automatic redirects drop sensitive headers such as `Cookie` on cross-origin hops.
3. Without the cookies, the www12 request arrives with none, and the server returns a login page instead of setting session cookies.

## Rules (llave `follow_redirects`)

| Rule | Behaviour |
|------|-----------|
| Cookie header | Every hop reuses the **original** `Cookie` header of the first request. It is not rebuilt from cookies captured mid-chain |
| `Set-Cookie` | Captured from every hop into the cookie map (empty value = delete) |
| Method | Each hop is a `GET` to `Location`. Relative locations resolve against the current origin. 307/308 method preservation is not implemented, since these endpoints have not been seen to use it |
| Limit | At most **20** redirects, like OkHttp. The 21st fails with `TooManyRedirects` before it is sent |
| Scope | Applied to every request (`execute`), not only DNI/NIE auth |

## Redirect Chain Detail

```mermaid
sequenceDiagram
    participant Client
    participant www2 as www2 (BUCV-JDIT)
    participant www12 as www12 (MOVI-AEAT)

    Client->>www2: POST /wlpl/BUCV-JDIT/AutenticaDniNieContrasteh?ref=...<br/>Cookie: sgat-language; appmovil; ...
    Note right of www2: Validates NIF + FECHA / SOPORTE

    www2-->>Client: 302 Found<br/>Location: /wlpl/MOVI-AEAT/AccesoW12Sv<br/>Set-Cookie: sgat_id_usr_sede, WWW12, WWW12V, JSESSIONID

    Note over Client: Store Set-Cookie<br/>Reuse the ORIGINAL Cookie header

    Client->>www12: GET /wlpl/MOVI-AEAT/AccesoW12Sv<br/>Cookie: (same as first request)
    www12-->>Client: 200 OK + Set-Cookie: sgat_id_usr_sede (updated)

    Note over Client: Parse body as JSON envelope<br/>(non-JSON: fall back to redirect count)
```

## Failure Detection

After `AutenticaDniNieContrasteh`, the final body is parsed as the standard JSON envelope first (`OK`, `KO -8`/`-10`, other `KO`). See [DNI/NIE + SMS Activation](../authentication-flows/dni-nie-sms-activation.md#step-1-autenticadniniecontrasteh). Only when the body is not JSON does llave use the older heuristic: **zero redirects** (HTTP 200 with the login form again) means the credentials were rejected. See [DNI Auth Failure Detection](../api-reference/error-codes.md#dni-auth-failure-detection).
