# SAML / DNIe Authentication

The `ClaveAuthentication` class in the DNIe library handles SAML-based authentication through `pasarela.clave.gob.es`.

## Flow (Proxy2 Path)

```
1. Start with HTML from pasarela.clave.gob.es/Proxy2/ServiceProvider
2. Parse HTML forms iteratively:
   Step 1: Find form matching initial attributes -> POST
   Step 2: Find form "idpRedirect" -> POST (sets value "AFIRMA")
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
4. Extracts the action URL
5. POSTs the form data
6. Uses the response HTML for the next step

## Cryptography

No custom crypto beyond TLS mutual auth with the DNIe certificate:
- The eID card provides the `SSLSocketFactory` for client certificate authentication
- Forms contain `SAMLResponse` fields for assertion relay
- The `Referer` header must be set to `https://pasarela.clave.gob.es/` during SAML flows

## Authentication Levels

SAML/DNIe is the **strongest** authentication level:

| Level | Method | Description |
|-------|--------|-------------|
| Weak | DNI/NIE + card data | `AutenticaDniNieContrasteh` |
| Medium | Cl@ve PIN | Server-generated, device-bound (`ClaveRequestPinSv`) |
| Strong | Cl@ve Movil | Push notification + confirm (`ClaveAuthenticateSv`) |
| **Strongest** | **DNIe certificate** | **Mutual TLS via SAML** |
