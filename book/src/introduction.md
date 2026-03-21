# Llave

Llave is an open-source Rust implementation of the Spanish AEAT Cl@ve PIN authentication system.

## Terminology

| Term | Description |
|------|-------------|
| **NIF** | Numero de Identificacion Fiscal. The Spanish tax ID number (8 digits + letter for DNI, or X/Y/Z + 7 digits + letter for NIE). |
| **DNI** | Documento Nacional de Identidad. The Spanish national ID card. |
| **NIE** | Numero de Identidad de Extranjero. Foreign national identification number in Spain. |
| **Soporte** | The support/serial number printed on the physical DNI/NIE card. |
| **Fecha** | The date of validity or date of birth, formatted as `DD-MM-YYYY`. |
| **Autenticacion debil** | "Weak authentication" -- identity proven via card data (NIF + fecha + soporte) rather than a digital certificate. |
| **Clave PIN** | A temporary 4-digit PIN valid for ~5 minutes, used for one-time authentication to government services. |
| **Clave Movil** | The mobile-based authentication system. The app on the user's device. |
| **device_id** | A server-generated UUID identifying the activated device. |
| **device_password** | A client-generated UUID used as a shared secret between device and server. |
| **pin24H** | A session cookie set after SMS validation. Required for device activation on www6. Named for the original 24-hour validity window. |
| **TrazasApp** | A custom HTTP header containing serialised cookie diagnostic information, mimicking the Android app's internal telemetry. |
