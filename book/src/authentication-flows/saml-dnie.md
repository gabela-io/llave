# SAML / DNIe Authentication

> **Not used by the Cl@ve app 6.2.10.** The class described here,
> `es.gob.fnmt.dniedroid.net.http.clave.ClaveAuthentication`, ships inside the
> bundled FNMT **DNIeDroid** library, but no `es.aeat.pin24h` code calls it. The
> app uses the DNIe only as an NFC client certificate for
> [certificate registration](./certificate-registration.md#dnie-via-nfc). This
> page documents the library code for reference.

`ClaveAuthentication` automates a SAML login through `pasarela.clave.gob.es` with the DNIe certificate. It walks a chain of HTML forms.

## Flow (Proxy2 Path)

When the starting URL contains `https://pasarela.clave.gob.es/Proxy2/ServiceProvider`:

```
1. Start with HTML from pasarela.clave.gob.es/Proxy2/ServiceProvider
2. Parse HTML forms iteratively:
   Step 1: Find form matching initial attributes -> POST
   Step 2: Find form "idpRedirect" -> POST (empty field set to "AFIRMA")
   Step 3: Find form "redirectForm" -> POST
   Step 4: Find form "redirectForm" -> POST
   Step 5: Find form "redirectForm" -> POST
```

## Alternative Flow (non-Proxy2 Path)

```
   Step 1: Find initial form -> POST
   Step 2: Find form "nationalRedirect" -> POST
   Step 3: Find form id="AuthenticateCitizen" -> POST
   Step 4: Find form id="prepareResponseAfirma" -> POST
   Step 5: Find form "redirectForm" -> extract redirectTo value
```

## How Each Step Works

Each step:
1. Parses the returned HTML
2. Finds the target `<form>` element
3. Extracts all `<input>` fields as key-value pairs
4. Extracts the action URL (relative actions are prefixed with `https://pasarela.clave.gob.es`)
5. POSTs the form data
6. Uses the response HTML for the next step

## Cryptography

There is no custom crypto beyond TLS mutual authentication with the DNIe certificate:
- The eID card provides the `SSLSocketFactory` (`DNIeSSLSocketFactory`) for client-certificate authentication.
- Forms carry `SAMLResponse` fields that relay the assertion.
- The `Referer` header is set to `https://pasarela.clave.gob.es/` during the flow.
