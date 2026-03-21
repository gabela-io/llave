# Client Identity

The AEAT server validates the client identity through HTTP headers. Getting these wrong causes silent failures or WAF rejections.

## User-Agent Format

The format must exactly match the official Android app:

```
APPMovil/Cl@ve/v{version}({build})/{os_version}/Android/{dalvik_agent}
```

Concrete value used:

```
APPMovil/Cl@ve/v6.2.5(288)/14/Android/Dalvik/2.1.0 (Linux; U; Android 14; SM-S928U Build/UP1A.231005.007)
```

| Component | Value | Source |
|-----------|-------|--------|
| App identifier | `APPMovil/Cl@ve` | Hardcoded in app |
| Version | `v6.2.5(288)` | `APP_VERSION` = `6.2.5`, build 288 |
| OS version | `14` | Android 14 |
| Platform | `Android` | Hardcoded |
| Dalvik agent | `Dalvik/2.1.0 (Linux; U; Android 14; SM-S928U Build/UP1A.231005.007)` | `System.getProperty("http.agent")` on Android |

> **Critical**: The full Dalvik user-agent suffix (including the parenthetical
> with Linux, device model, and build number) is **required**. A truncated
> User-Agent (e.g., just `APPMovil/Cl@ve/v6.2.5(288)/14/Android/Dalvik/2.1.0`)
> causes the server's WAF to reject the request with a "Pagina no habilitada en
> internet publico" (page not enabled on public internet) error page. This was
> one of the hardest issues to diagnose during reverse engineering.

## Default Headers

Every request includes:

| Header | Value |
|--------|-------|
| `User-Agent` | Full Android app user agent (see above) |
| `Accept` | `application/json` |
| `Accept-Language` | `es_ES` |
| `Cookie` | All managed cookies (see [Cookie Management](./cookie-management.md)) |
| `TrazasApp` | JSON cookie diagnostic object (see below) |

## The TrazasApp Header

Every request includes a custom `TrazasApp` header containing a JSON object that mirrors the Android app's internal `getCookiesInApp()` method. This header serialises the current cookie state for server-side diagnostics and legitimacy checks.

### Structure

```json
{
  "cookiesWww1Gestor":          "{CERT_WWW1} y {CERT_WWW1V}",
  "cookiesWww6Gestor":          "{pin24H} y {pin24V}",
  "cookiesWww12Gestor":         "{WWW12} y {WWW12V}",
  "cookiesAppMovilGestor":      "{appmovil}",
  "cookiesSgatLanguageGestor":  "{sgat-language}",
  "cookiesWww1Local":           "{CERT_WWW1} y {CERT_WWW1V}",
  "cookiesWww6Local":           "{pin24H} y {pin24V}",
  "cookiesWww12Local":          "{WWW12} y {WWW12V}",
  "cookiesAppMovilLocal":       "{appmovil}",
  "cookiesSgatLanguageLocal":   "{sgat-language}"
}
```

- **`*Gestor`** fields: Cookie values from the app's cookie jar (runtime).
- **`*Local`** fields: Cookie values from the device's keychain storage (persisted). In practice, both contain the same values.
- **`" y "` separator**: Cookie value pairs are joined with the literal string ` y ` (Spanish for "and").
- **`"null"`**: When a cookie is not set, its value is the literal string `"null"`.

### Example (early in flow, before SMS validation)

```json
{
  "cookiesWww1Gestor":          "null y null",
  "cookiesWww6Gestor":          "null y null",
  "cookiesWww12Gestor":         "4C8937...== y 20260310-15105322",
  "cookiesAppMovilGestor":      "eyJhcHAiOiJjbGF2ZSI...",
  "cookiesSgatLanguageGestor":  "es_ES",
  "cookiesWww1Local":           "null y null",
  "cookiesWww6Local":           "null y null",
  "cookiesWww12Local":          "4C8937...== y 20260310-15105322",
  "cookiesAppMovilLocal":       "eyJhcHAiOiJjbGF2ZSI...",
  "cookiesSgatLanguageLocal":   "es_ES"
}
```

### Implementation Notes

While the server may not strictly validate every field of the `TrazasApp` header, omitting the header entirely may cause requests to be rejected. It is sent with every request as a form of telemetry.
