{
  inputs,
  pkgs,
  system,
  ...
}:
# Lean shell for CI: what `just test`, `just lint`, the Flutter checks and
# the book need, without the Android SDK or the reverse-engineering tools of
# the default shell.
let
  fenixPkgs = inputs.fenix.packages.${system};
  toolchain = fenixPkgs.stable.withComponents [
    "cargo"
    "clippy"
    "rustc"
    "rustfmt"
  ];
in
pkgs.mkShell {
  packages = [
    toolchain
    pkgs.just
    pkgs.jq
    pkgs.flutter
    pkgs.mdbook
  ];
}
