{
  inputs,
  pkgs,
  system,
  ...
}:
let
  fenixPkgs = inputs.fenix.packages.${system};
  toolchain = fenixPkgs.combine [
    (fenixPkgs.stable.withComponents [
      "cargo"
      "clippy"
      "rust-src"
      "rustc"
      "rustfmt"
    ])
    fenixPkgs.targets.aarch64-linux-android.stable.rust-std
    fenixPkgs.targets.armv7-linux-androideabi.stable.rust-std
    fenixPkgs.targets.x86_64-linux-android.stable.rust-std
    fenixPkgs.targets.i686-linux-android.stable.rust-std
  ];

  androidComposition = pkgs.androidenv.composeAndroidPackages {
    platformVersions = [
      "34"
      "35"
      "36"
    ];
    buildToolsVersions = [
      "34.0.0"
      "35.0.0"
      "36.0.0"
    ];
    includeNDK = true;
    ndkVersions = [ "28.2.13676358" ];
    cmakeVersions = [ "3.22.1" ];
    includeEmulator = false;
    includeSystemImages = false;
    includeSources = false;
  };
  androidSdk = androidComposition.androidsdk;
in
pkgs.mkShell (
  {
    buildInputs = [
      toolchain
      fenixPkgs.rust-analyzer
      pkgs.just

      pkgs.openssl
      pkgs.pkg-config
      pkgs.dbus

      # Flutter + desktop/mobile development
      pkgs.flutter
      pkgs.dart

      # Flutter Linux desktop dependencies
      pkgs.gtk3
      # glib/gio private deps — their .pc files must be on PKG_CONFIG_PATH
      # because Flutter's cmake probes them transitively via Requires.private.
      pkgs.glib.dev
      pkgs.libsysprof-capture
      pkgs.pcre2
      pkgs.util-linux.dev # mount.pc, required by gio-2.0
      pkgs.libselinux # libselinux.pc, required by gio-2.0
      pkgs.libsepol # libsepol.pc, required by libselinux
      pkgs.fribidi # fribidi.pc, required by pango
      pkgs.libthai # libthai.pc, required by pango
      pkgs.libdatrie # libdatrie.pc, required by libthai
      pkgs.libxdmcp # xdmcp.pc, required by xcb
      pkgs.pcre # libpcre.pc (v1), required by some transitive deps
      pkgs.libepoxy # epoxy.pc, required by gtk3
      pkgs.libdeflate # libdeflate.pc, required by libtiff-4
      pkgs.lerc # Lerc.pc, required by libtiff-4
      pkgs.libgcrypt # libgcrypt.pc, required by libsecret-1
      pkgs.libgpg-error # gpg-error.pc, required by libgcrypt
      pkgs.libsecret # flutter_secure_storage on Linux
      pkgs.jsoncpp # flutter_secure_storage on Linux
      pkgs.cmake
      pkgs.ninja
      pkgs.clang

      # APK reverse engineering tools
      pkgs.apktool
      pkgs.jadx
    ]
    ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
      androidSdk
      pkgs.jdk17
      pkgs.cargo-ndk
    ]
    ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
      pkgs.libiconv
      pkgs.darwin.apple_sdk.frameworks.Security
      pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
    ];

    RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";

    shellHook = ''
          # Ensure flutter_rust_bridge_codegen is available
          if ! command -v flutter_rust_bridge_codegen &>/dev/null; then
            echo "Installing flutter_rust_bridge_codegen..."
            cargo install flutter_rust_bridge_codegen@2.11.1
          fi

          # Android SDK setup (Linux only)
          if [ -n "''${ANDROID_SDK_ROOT:-}" ]; then
            AAPT2="$(ls -1d "$ANDROID_SDK_ROOT/build-tools/"*/aapt2 2>/dev/null | sort -V | tail -1)"
            if [ -n "$AAPT2" ]; then
              export GRADLE_OPTS="-Dorg.gradle.project.android.aapt2FromMavenOverride=$AAPT2"
            fi
            if [ -d "flutter/android" ]; then
              FLUTTER_SDK_PATH="$(dirname "$(dirname "$(command -v flutter)")")"
              cat > flutter/android/local.properties <<PROPS
      sdk.dir=$ANDROID_SDK_ROOT
      ndk.dir=$ANDROID_NDK_HOME
      flutter.sdk=$FLUTTER_SDK_PATH
      PROPS
            fi
          fi
    '';
  }
  // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
    ANDROID_SDK_ROOT = "${androidSdk}/libexec/android-sdk";
    ANDROID_HOME = "${androidSdk}/libexec/android-sdk";
    ANDROID_NDK_HOME = "${androidSdk}/libexec/android-sdk/ndk/28.2.13676358";
    NDK_HOME = "${androidSdk}/libexec/android-sdk/ndk/28.2.13676358";
    JAVA_HOME = "${pkgs.jdk17.home}";
  }
)
