# Debian Packaging (packaging/deb)

## Overview
Builds the Debian package `sea-forge_<version>_amd64.deb` for the governed
capability-execution kernel (CLI and daemon) from already-built release
binaries. The package ships the four binaries the isolated backend and MCP
bridge expect to find beside the running executable.

## Interface
- `build.sh` — POSIX `sh` build script. Usage:
  `packaging/deb/build.sh <version> <bindir> <outdir>`; exits 2 on wrong
  argument count or an invalid version string
  ([build.sh:usage check / version charset case]).
- Required inputs: executables `sea-forge`, `sea-forge-server`,
  `sea-forge-workload-exec`, and `sea-forge-mcp-connect` in `<bindir>`
  ([build.sh:bins list and existence check]); repo-root `LICENSE`
  ([build.sh:install of copyright file]).
- Output: `<outdir>/sea-forge_<version>_amd64.deb`, built with
  `dpkg-deb --build --root-owner-group`, followed by an informational
  `dpkg-deb --info` print ([build.sh:dpkg-deb build]).
- Package layout: binaries to `/usr/bin` (mode 0755), LICENSE to
  `/usr/share/doc/sea-forge/copyright` ([build.sh:root staging]).

## Invariants
- `Depends` is computed from the binaries with `dpkg-shlibdeps` (NEEDED
  libraries resolved against the installed package database), never
  hand-written ([build.sh:header comment and dpkg-shlibdeps probe]).
- The computed dependencies must include `libc6`, or the build aborts
  ([build.sh:libc6 check]).
- The version string must match `[0-9A-Za-z.+~-]+`; anything else is rejected
  ([build.sh:version charset case]).
- The script fails closed if any required executable is missing
  ([build.sh:missing executable check]).

## Side Effects
- Creates a temporary staging tree and scratch directory via `mktemp -d`,
  removed by an EXIT trap ([build.sh:work directory and trap]).
- Writes the `.deb` into `<outdir>` (created if absent) and prints package
  metadata ([build.sh:dpkg-deb build]).
- Runs `dpkg-shlibdeps` against the supplied binaries
  ([build.sh:dpkg-shlibdeps probe]). No network access and no service
  management; the package is only built, never installed here.

## Verification
- Run: `sh -n packaging/deb/build.sh` (syntax; the real build additionally
  requires the four release binaries and Debian tooling `dpkg-shlibdeps` /
  `dpkg-deb`, and is exercised by the release pipeline).
