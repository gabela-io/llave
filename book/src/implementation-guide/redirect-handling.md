# Redirect Handling

The DNI/NIE authentication step involves a critical redirect chain that crosses subdomain boundaries. Automatic redirect following **must be disabled** -- the client must follow each redirect manually.

## Why Manual Redirect Following Is Required

1. `reqwest` (and most HTTP clients) strip custom headers like `Cookie` and `TrazasApp` on cross-origin redirects for security reasons.
2. The redirect goes from `www2` to `www12` -- different origins.
3. The server expects cookies to be present on the redirected request.
4. Without manual redirect handling, the www12 request arrives with no cookies, and the server returns a login page instead of setting session cookies.

## Redirect Chain Detail

```mermaid
sequenceDiagram
    participant Client
    participant www2 as www2 (BUCV-JDIT)
    participant www12 as www12 (MOVI-AEAT)

    Client->>www2: POST /wlpl/BUCV-JDIT/AutenticaDniNieContrasteh?ref=...
    Note right of www2: Validates NIF + FECHA + SOPORTE<br/>against citizen database

    www2-->>Client: 302 Found<br/>Location: /wlpl/MOVI-AEAT/AccesoW12Sv<br/>Set-Cookie: JSESSIONID=...<br/>Set-Cookie: sgat_id_usr_sede=...<br/>Set-Cookie: WWW12=...<br/>Set-Cookie: WWW12V=...

    Note over Client: Re-attach ALL cookies<br/>+ TrazasApp header

    Client->>www12: GET /wlpl/MOVI-AEAT/AccesoW12Sv<br/>Cookie: sgat-language; JSESSIONID; appmovil; WWW12; WWW12V; sgat_id_usr_sede
    www12-->>Client: 200 OK<br/>Content-Type: text/html<br/>Set-Cookie: sgat_id_usr_sede=... (updated)

    Note over Client: HTML body is IGNORED<br/>Only cookies matter
```

## Implementation Steps

At each hop in the redirect chain, the client must:

1. **Capture all `Set-Cookie` headers** from the response and upsert them into the cookie store.
2. **Read the `Location` header** to determine the redirect target.
3. **Re-attach all cookies** (from the cookie store) to the new request.
4. **Re-attach the `TrazasApp` header** with updated cookie values.
5. **Send the redirected request** (typically a `GET`).

## Failure Detection

If the redirect count is 0 after `AutenticaDniNieContrasteh`, the server returned the login form again (HTTP 200, no redirect). This means the credentials (NIF, fecha, or soporte) were rejected. See [DNI Auth Failure Detection](../api-reference/error-codes.md#dni-auth-failure-detection).
