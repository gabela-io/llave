# Changelog

All notable changes to llave are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and llave uses
[Semantic Versioning](https://semver.org/) for its own version. Each release
also names the official Cl@ve app version (`es.aeat.pin24h`) whose behaviour
it matches.

## [0.1.0] - Unreleased

Matches Cl@ve **6.2.10** (build 294).

### Added
- DNI/NIE + SMS activation, the only activation path of the official app: registration check, date-of-birth retry (codes -8/-10) and the right document field per NIF type (validity date, date of birth or support number).
- Reveal the pending Cl@ve PIN, with its validity countdown.
- Approve or reject Cl@ve Móvil logins with the app's security check (organisation, verification code, "did you start this?"), from polling or a QR code.
- History of confirmed and rejected requests, account data and device deactivation.
- CLI (`llave`): `activate`, `pin`, `pending`, `listen`, `confirm`, `reject`, `qr`, `history`, `my-data`, `check`, `dni-auth`, `status`, `deactivate`, `logout`. Credentials are kept in a PIN-sealed file (Argon2id + AES-256-GCM) with the kernel keyring as a cache. `llave --version` names the matched Cl@ve version.
- Android and Linux app: waiting for a login instead of push notifications, PIN card, QR scanner, PIN-sealed session with optional biometrics, Spanish and English.
- Protocol documentation (mdbook) for the flows, cookies, redirects and storage of Cl@ve 6.2.10.

[0.1.0]: https://github.com/gabela-io/llave/releases/tag/v0.1.0
