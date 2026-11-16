# macOS Packaging (packaging/macos)

## Overview
Builds the macOS install package (`.pkg`) and disk image (`.dmg`) for
SEA Forge from already-built arm64 (and optionally x86_64) release binaries.
Binaries install to `/usr/local/bin`. The package is deliberately UNSIGNED:
no Developer ID certificate is used, and none is claimed anywhere.

## Interface
- `build.sh` — POSIX `sh` build script. Usage:
  `packaging/macos/build.sh <version> <arm64dir> <x86dir|''> <outdir>`;
  exits 2 on wrong argument count or an invalid version string
  ([build.sh:usage check / version charset case]).
- Required inputs: executables `sea-forge`, `sea-forge-server`, and
  `sea-forge-jail-launcher` in `<arm64dir>`, and in `<x86dir>` when a
  universal build is requested ([build.sh:bins existence check]); repo-root
  `LICENSE` ([build.sh:dmg staging]).
- Naming: `<x86dir>` empty produces the arm64-only
  `sea-forge-<version>-macos-arm64.pkg`/`.dmg`; with both directories the
  binaries are merged with `lipo -create` into
  `sea-forge-<version>-macos-universal.pkg`/`.dmg`
  ([build.sh:label selection and lipo merge]).
- Tooling invoked: `pkgbuild --identifier ai.godspeed.sea-forge
  --install-location /usr/local` for the package, and `hdiutil create
  -format UDZO` for the disk image ([build.sh:pkgbuild / hdiutil calls]).
- The `.dmg` contains the package, `LICENSE`, and a generated
  `README-install.txt` covering install, unsigned-package handling
  (Gatekeeper, `xattr`), SHA-256/`gh attestation` verification, and uninstall
  ([build.sh:README-install.txt heredoc]).

## Invariants
- The package is unsigned and the artifacts never claim otherwise; the
  install notes state the package is NOT signed or notarized
  ([build.sh:header comment and README-install.txt heredoc]).
- The version string must match `[0-9A-Za-z.+~-]+` ([build.sh:version charset case]).
- Fails closed if any required executable is missing in the supplied binary
  directories ([build.sh:bins existence check]).

## Side Effects
- Creates a temporary staging tree via `mktemp -d`, removed by an EXIT trap
  ([build.sh:work directory and trap]).
- Writes the `.pkg` and `.dmg` into `<outdir>` (created if absent; an
  existing `.dmg` of the same name is removed first
  ([build.sh:rm of previous dmg])). No service management and no keychain
  access; packages are only built, never installed here.

## Verification
- Run: `sh -n packaging/macos/build.sh` (syntax; the real build additionally
  requires macOS tooling `pkgbuild`, `lipo`, and `hdiutil`, and is exercised
  by the release pipeline).
