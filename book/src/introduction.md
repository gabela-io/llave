# Llave

Llave is an open-source Rust implementation of the Spanish AEAT Cl@ve mobile authentication system (Cl@ve PIN and Cl@ve Móvil).

This book documents the protocol as used by the official Android app **Cl@ve v6.2.10 (build 294)**, `es.aeat.pin24h`, and how llave implements it. Behaviour seen only by llave against the server, and not found in the app, is marked as empirical.

## Terminology

| Term | Description |
|------|-------------|
| **NIF** | Número de Identificación Fiscal, the Spanish tax ID (DNI: 8 digits + letter; NIE: X/Y/Z + 7 digits + letter; special K/L/M + 7 digits + letter). |
| **DNI** | Documento Nacional de Identidad. The Spanish national ID card. |
| **NIE** | Número de Identidad de Extranjero. Foreign national identification number in Spain. |
| **Fecha** | `FECHA` form field: the DNI validity date (`dd-MM-yyyy`), or the date of birth for K/L DNIs, M NIEs and the `-8`/`-10` retries. Empty for X/Y/Z NIEs. |
| **Soporte** | `SOPORTE` form field: the support number printed on an X/Y/Z NIE card. Empty for DNIs. |
| **Autenticación débil** | "Weak authentication": identity proven with card data (NIF + fecha or soporte) instead of a certificate. |
| **Cl@ve PIN** | A short-lived PIN for one login. It is requested on the AEAT website; the app only reveals the pending PIN (`ClaveRequestPinSv`). |
| **Cl@ve Móvil** | Login approved from the app: the website creates a request (`peticion`) that the user confirms or rejects on the device. |
| **device_id** | Server-assigned identifier of the activated device, returned by `ClaveActivateAuthenticationSv`. |
| **device_password** | Client-generated shared secret, sent as `user_password`: SHA-512 (uppercase hex, 128 chars) of 256 random alphanumeric characters. |
| **appmovil** | Cookie that marks a mobile-app session; may be set by `ClaveStartingSv`, otherwise requested with `SetCookieAppMovil`. |
| **pin24H** | Cookie set by `ValidarClaveMovilSMS` after the SMS code is validated. Required for device activation on www6. |
