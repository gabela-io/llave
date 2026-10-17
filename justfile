# Llave — common development commands

# Default: list available recipes
default:
    @just --list

# --- Build ---

# Build the CLI binary (debug)
build-cli:
    cargo build -p llave-cli

# Build the CLI binary (release)
build-cli-release:
    cargo build -p llave-cli --release

# Build the FFI shared library (debug)
build-ffi:
    cargo build -p llave-core-ffi

# Build the FFI shared library (release)
build-ffi-release:
    cargo build -p llave-core-ffi --release

# Build everything (debug)
build:
    cargo build

# Build everything (release)
build-release:
    cargo build --release

# --- Check & Lint ---

# Type-check the workspace
check:
    cargo check

# Check formatting and run clippy on the workspace
lint: fmt-check
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# --- Flutter ---

# Generate flutter_rust_bridge bindings
frb-generate:
    cd flutter && flutter_rust_bridge_codegen generate

# Build Flutter Linux desktop app (debug)
flutter-build-linux:
    cd flutter && flutter build linux

# Build Flutter Linux desktop app (release)
flutter-build-linux-release:
    cd flutter && flutter build linux --release

# Run Flutter app on Linux desktop
flutter-run:
    cd flutter && flutter run -d linux

# Run Flutter code generation (freezed, json_serializable)
flutter-codegen:
    cd flutter && dart run build_runner build --delete-conflicting-outputs

# Get Flutter dependencies
flutter-deps:
    cd flutter && flutter pub get

# --- Android ---

# Build Rust FFI for Android (all ABIs)
build-ffi-android:
    cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o flutter/android/app/src/main/jniLibs build -p llave-core-ffi --release

# Build Flutter Android APK (debug)
flutter-build-apk:
    cd flutter && flutter build apk --debug

# Build Flutter Android APK (release)
flutter-build-apk-release:
    cd flutter && flutter build apk --release

# Run Flutter on connected Android device
flutter-run-android:
    cd flutter && flutter run -d android

# Install APK on connected Android device
flutter-install-apk:
    adb install -r flutter/build/app/outputs/flutter-apk/app-debug.apk

# Build and install debug APK on connected Android device
flutter-deploy-apk: flutter-build-apk flutter-install-apk

# --- Test ---

# Run Rust tests
test:
    cargo test

# Run Flutter tests
flutter-test:
    cd flutter && flutter test

# --- Run ---

# Run the CLI (pass args after --)
run *ARGS:
    cargo run -p llave-cli -- {{ ARGS }}

# Run the CLI in release mode
run-release *ARGS:
    cargo run -p llave-cli --release -- {{ ARGS }}

# --- Clean ---

# Clean Rust build artifacts
clean:
    cargo clean

# Clean Flutter build artifacts
flutter-clean:
    cd flutter && flutter clean

# Clean everything
clean-all: clean flutter-clean

# --- Release ---

# Needs a clean tree and flutter/android/key.properties (release keystore);
# runs every check, then builds a static Linux CLI, the signed APK and
# SHA256SUMS.
# Build the signed release artifacts into dist/v<version>
release:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "llave-cli") | .version')
    if [ -n "$(git status --porcelain)" ]; then
        echo "Commit or stash your changes first." >&2; exit 1
    fi
    if [ ! -f flutter/android/key.properties ]; then
        echo "Missing flutter/android/key.properties (release keystore)." >&2; exit 1
    fi

    just test lint
    (cd flutter && flutter analyze && flutter test)
    git checkout -- flutter/analysis_options.yaml flutter/linux/flutter/generated_plugins.cmake

    cargo build -p llave-cli --release --target x86_64-unknown-linux-musl
    (cd flutter && flutter build apk --release)
    git checkout -- flutter/analysis_options.yaml flutter/linux/flutter/generated_plugins.cmake

    dist="dist/v$version"
    rm -rf "$dist" && mkdir -p "$dist"
    cp target/x86_64-unknown-linux-musl/release/llave "$dist/llave-$version-linux-x86_64"
    cp flutter/build/app/outputs/flutter-apk/app-release.apk "$dist/llave-$version.apk"

    build_tools="$(ls -d "$ANDROID_HOME"/build-tools/* | sort -V | tail -1)"
    if ! "$build_tools/aapt2" dump badging "$dist/llave-$version.apk" | grep -q "versionName='$version'"; then
        echo "The APK does not carry version $version (see flutter/android/local.properties)." >&2; exit 1
    fi
    "$build_tools/apksigner" verify --print-certs "$dist/llave-$version.apk" | grep -E 'Signer #1 certificate (DN|SHA-256)'
    if "$build_tools/apksigner" verify --print-certs "$dist/llave-$version.apk" | grep -q 'CN=Android Debug'; then
        echo "The APK is signed with the debug key; check flutter/android/key.properties." >&2; exit 1
    fi
    (cd "$dist" && sha256sum llave-* > SHA256SUMS)
    echo "Release artifacts in $dist:"
    ls -l "$dist"
