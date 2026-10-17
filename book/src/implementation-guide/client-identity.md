# Client Identity

The AEAT server recognises the client from HTTP headers and from a few form fields. If these are wrong, requests fail silently or the WAF rejects them.

## User-Agent Format

Built by `DeviceUtils.getUserAgent()`:

```
APPMovil/Cl@ve/v{version}({build})/{Build.VERSION.RELEASE}/Android/{System.getProperty("http.agent")}
```

Value used by llave (app 6.2.10, build 294):

```
APPMovil/Cl@ve/v6.2.10(294)/14/Android/Dalvik/2.1.0 (Linux; U; Android 14; SM-S928U Build/UP1A.231005.007)
```

| Component | Value | Source |
|-----------|-------|--------|
| App identifier | `APPMovil/Cl@ve` | Hardcoded |
| Version | `v6.2.10(294)` | `getVersionAppForUserAgent()` |
| OS version | `14` | `Build.VERSION.RELEASE` |
| Platform | `Android` | Hardcoded |
| Dalvik agent | `Dalvik/2.1.0 (Linux; U; Android 14; SM-S928U Build/UP1A.231005.007)` | `http.agent`; uses the bare `Build.MODEL` |

> **Critical**: The full Dalvik suffix, including the parenthetical with Linux,
> device model and build number, is **required**. With a truncated User-Agent
> (e.g. just `APPMovil/Cl@ve/v6.2.10(294)/14/Android/Dalvik/2.1.0`), the WAF
> answers with a "Página no habilitada en internet público" error page.

## Form Fields Identifying the Client

Most `MOVI-P24H` endpoints repeat the client identity in the body:

| Field | Value | App source |
|-------|-------|------------|
| `version_app` | `6.2.10` | `DeviceUtils.getVersionApp()`. Must match the User-Agent version |
| `sistema_operativo` | `A` | `getSoReduced()` (Android) |
| `version_os` | `14` | `Build.VERSION.RELEASE` |
| `modelo` | `Samsung SM-S928U` | `getDeviceName()` (see below) |

`modelo` is **not** the bare model. `getDeviceName()` returns the capitalised `Build.MANUFACTURER` + `" "` + `Build.MODEL`. If the model already starts with the manufacturer name, it returns just the capitalised model. llave sends `Samsung SM-S928U`. The same fields form the JSON body of `SetCookieAppMovil`, which sets the `appmovil` cookie.

Error `500600` means the server considers the app version outdated (llave: `LlaveError::AppVersionOutdated`).

## Default Headers

| Header | Value |
|--------|-------|
| `User-Agent` | Full app User-Agent (see above) |
| `Accept` | `application/json` |
| `Accept-Language` | `es_ES` |
| `Cookie` | Explicit per-request header with every stored cookie except `JSESSIONID` (see [Cookie Management](./cookie-management.md)) |
| `Content-Type` | `application/x-www-form-urlencoded` for form endpoints; `application/json; charset=UTF-8` for JSON endpoints (`SetCookieAppMovil`, certificate registration); **absent** for `ObtenerClaveMovilSMS` (empty body) |

The app also sends `If-Modified-Since`, but only when it downloads the carousel.

## The TrazasApp Header

**Not sent.** Cl@ve 6.2.10 has no `TrazasApp` header, `getCookiesInApp` or `cookiesWww*Gestor` structure. Earlier versions of this book said a JSON cookie-diagnostic header went out "with every request", but it cannot be confirmed for any app version, and llave has removed it.

## Transport

The app's `SecuredHttpClient` settings, which llave mirrors:

| Setting | Value |
|---------|-------|
| TLS | 1.2 or newer |
| Connect timeout | 30 s |
| Read / write timeout | 190 s (llave: 190 s overall request timeout) |
| Redirects | Followed like OkHttp, at most 20 (see [Redirect Handling](./redirect-handling.md)) |
| Cookie store | None in the HTTP client; cookies are managed by hand |
