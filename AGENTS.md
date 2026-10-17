# llave

Rust core + CLI + Flutter app for Cl@ve. Behaviour must match the official app 6.2.10 (es.aeat.pin24h, build 294).

## Commands

| Command | Purpose |
|---|---|
| `just test` | Rust tests (core, CLI, FFI) |
| `just lint` | rustfmt check and clippy with `-D warnings` (must stay clean) |
| `just run <args>` | Run the CLI, e.g. `just run pin --plain` |
| `cd flutter && flutter analyze` | Dart analysis (must report no issues) |
| `just frb-generate` | Regenerate flutter_rust_bridge bindings (see Gotchas) |
| `just release` | Signed release artifacts in `dist/v<version>` (see Releases) |

The Nix dev shell (direnv) provides every tool; run commands inside it.

## Architecture

- `crates/llave-core`: all protocol logic (`api.rs` HTTP client, `auth.rs` flows, `session.rs`, `config.rs` NIF rules). CLI and FFI must call core functions, never reimplement flows.
- `crates/llave-core-ffi`: two layers. `c_ffi.rs` is the C ABI used by Flutter through the hand-written `flutter/lib/src/rust/api/api.dart`; `api.rs` + generated `frb_generated.rs` are the flutter_rust_bridge layer.
- `crates/llave-cli`: the `llave` binary (clap).
- `book/`: protocol documentation; update it when behaviour changes.

## Gotchas

- `flutter_rust_bridge_codegen` lives in `~/.cargo/bin` (not on PATH) and reformats the hand-written `flutter/lib/src/rust/api/api.dart`: copy that file before `just frb-generate` and restore it after.
- `flutter analyze` / `pub get` rewrite `flutter/analysis_options.yaml` and `flutter/linux/flutter/generated_plugins.cmake`: `git checkout` both afterwards.
- clap: `-v` is the global `--verbose`; never give a subcommand argument the short `-v`. The `cli_definition_is_valid` test catches clashes.
- Every C export runs inside `ffi_guard` and documents `# Safety`.

## Releases

- llave has its own SemVer version: bump `[workspace.package] version` in `Cargo.toml` and `version` in `flutter/pubspec.yaml` together; the build number is MAJOR*10000 + MINOR*100 + PATCH (a CLI test checks both). The matched Cl@ve version is `CLAVE_APP_VERSION`/`CLAVE_APP_BUILD` in `llave-core` `api.rs`.
- Add the release to `CHANGELOG.md` and the compatibility table in `README.md`.
- `just release` needs `flutter/android/key.properties` pointing to the release keystore, which lives outside the repo. Never commit or print the keystore or its passwords.
- Tag on `main` after merging, with a signed tag (`git tag -s vX.Y.Z`), then attach `dist/vX.Y.Z/*` to the GitHub release.

## Protocol rules

- Never send real requests to AEAT servers unless I ask; there is no test environment.
- Do not implement `ClaveCancelSv`: it renounces Cl@ve entirely.
- Reference for app behaviour: decompiled 6.2.10 in `.cooking/jadx-universal` (smali in `.cooking/apktool-universal`), gitignored and local only. Verify a claim there (smali if jadx is ambiguous) before changing protocol code; findings are tracked in `.cooking/paso4-auditoria.md`.
