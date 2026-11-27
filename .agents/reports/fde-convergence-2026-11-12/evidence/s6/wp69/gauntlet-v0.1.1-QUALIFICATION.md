# Release qualification: Gauntlet 0.1.1

Commit `fd797dc3b939322aa65adb5aa1ce6caef9c13028`. Signing: unsigned (no Apple Developer ID / Authenticode signing is configured).
Re-aggregated by the promote workflow from the release's own record assets.

| Platform | Status | Evidence | Runner image | Notes |
|---|---|---|---|---|
| Windows x64 | **QUALIFIED** | hosted-runner | win25-vs2026 20261103.298.1 | provenance: NOT_RUN (no attestation exists for this artifact yet: provenance is created by the publish job after qualification passes and verified there)<br>upgrade: NOT_RUN (no previous released version exists to upgrade from) |
| Linux amd64 (Ubuntu 22.04) | **QUALIFIED** | hosted-runner | ubuntu22 20261111.360.2 | provenance: NOT_RUN (no attestation exists for this artifact yet: provenance is created by the publish job after qualification passes and verified there)<br>upgrade: NOT_RUN (no previous released version exists to upgrade from) |
| Linux arm64 (generic, hosted) | **QUALIFIED** | hosted-runner | ubuntu22-arm64 20261111.190.1 | provenance: NOT_RUN (no attestation exists for this artifact yet: provenance is created by the publish job after qualification passes and verified there)<br>upgrade: NOT_RUN (no previous released version exists to upgrade from) |
| NVIDIA Jetson AGX Orin (real hardware only) | **QUALIFIED** | real-hardware | jetson-local | provenance: NOT_RUN (no --attest-repo given (offline or operator run); verify with gh attestation verify manually) |
| macOS arm64 | **QUALIFIED** | hosted-runner | macos15 20261112.0539.3 | provenance: NOT_RUN (no attestation exists for this artifact yet: provenance is created by the publish job after qualification passes and verified there)<br>tarball_install: NOT_RUN (macOS ships a .pkg/.dmg only (no tarball); the pkg is exercised in the install step)<br>upgrade: NOT_RUN (no previous released version exists to upgrade from) |
| macOS Intel (x86_64) | **QUALIFIED** | hosted-runner | macos15 20260824.0482.1 | provenance: NOT_RUN (no attestation exists for this artifact yet: provenance is created by the publish job after qualification passes and verified there)<br>tarball_install: NOT_RUN (macOS ships a .pkg/.dmg only (no tarball); the pkg is exercised in the install step)<br>upgrade: NOT_RUN (no previous released version exists to upgrade from) |

Overall: **every claimed platform is QUALIFIED**

## Step matrix

| Platform | checksums | provenance | tarball_install | install | upgrade | smoke | governed_run | uninstall |
|---|---|---|---|---|---|---|---|---|
| Windows x64 | PASS | NOT_RUN | PASS | PASS | NOT_RUN | PASS | PASS | PASS |
| Linux amd64 (Ubuntu 22.04) | PASS | NOT_RUN | PASS | PASS | NOT_RUN | PASS | PASS | PASS |
| Linux arm64 (generic, hosted) | PASS | NOT_RUN | PASS | PASS | NOT_RUN | PASS | PASS | PASS |
| NVIDIA Jetson AGX Orin (real hardware only) | PASS | NOT_RUN | PASS | PASS | PASS | PASS | PASS | PASS |
| macOS arm64 | PASS | NOT_RUN | NOT_RUN | PASS | NOT_RUN | PASS | PASS | PASS |
| macOS Intel (x86_64) | PASS | NOT_RUN | NOT_RUN | PASS | NOT_RUN | PASS | PASS | PASS |
