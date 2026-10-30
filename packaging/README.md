# Packaging (packaging)

## Overview
Platform packaging and packaging-adjacent verification for SEA Forge. This
directory contains a direct script plus one build-script package per target
platform: the Debian package builder, the macOS package/disk-image builder,
and a Windows Inno Setup bundle. A functional smoke test of the installed
binaries lives directly in this directory.

## Interface
- `smoke-cell.sh` — functional smoke of installed SEA-Forge binaries on
  Linux and macOS. Usage: `packaging/smoke-cell.sh <sea-forge>
  <sea-forge-server>`; exits 2 on wrong argument count
  ([smoke-cell.sh:usage check]). Creates a fresh cell in a temp directory,
  runs `<cli> --version` and `<cli> runs --root <cell>`, starts
  `sea-forge-server` against the cell with `SEA_FORGE_ROOT` set, waits up to
  60 s for a socket to appear under the cell, then stops the daemon
  ([smoke-cell.sh:main sequence]).
- Debian package builder: [./deb/README.md] - Builds
  `sea-forge_<version>_amd64.deb` from release binaries with
  `dpkg-shlibdeps`-computed dependencies; ships `sea-forge`,
  `sea-forge-server`, `sea-forge-workload-exec`, `sea-forge-mcp-connect`.
- macOS package builder: [./macos/README.md] - Builds the unsigned
  `sea-forge-<version>-macos-{arm64,universal}.pkg` and `.dmg` (pkgbuild /
  lipo / hdiutil); ships `sea-forge`, `sea-forge-server`,
  `sea-forge-jail-launcher` to `/usr/local/bin`.
- windows: (no contract written) - `build.ps1` and the `sea-forge.iss`
  Inno Setup script for the Windows installer; not documented in this pass.

## Invariants
- `smoke-cell.sh` fails loudly if the server exits before creating its socket
  or if no socket appears under the cell within 60 s
  ([smoke-cell.sh:socket wait loop]).
- `smoke-cell.sh` cleans up completely on exit: kills the server and removes
  the temp cell via an EXIT trap ([smoke-cell.sh:cleanup trap]).
- Build scripts validate their version strings and fail closed on missing
  binaries ([deb/README.md], [macos/README.md]).

## Side Effects
- `smoke-cell.sh` creates and removes a temporary cell directory, starts and
  stops a real `sea-forge-server` process, and captures server output to
  temp log files ([smoke-cell.sh:work dir, background server, cleanup trap]).
- The build scripts write `.deb` / `.pkg` / `.dmg` artifacts into their
  output directory ([deb/README.md], [macos/README.md]).

## Verification
- Run: `sh -n packaging/smoke-cell.sh packaging/deb/build.sh packaging/macos/build.sh`
  (syntax). The real gates are the release pipeline builds and
  `packaging/smoke-cell.sh` itself against installed binaries; no `just`
  recipe covers this directory.
