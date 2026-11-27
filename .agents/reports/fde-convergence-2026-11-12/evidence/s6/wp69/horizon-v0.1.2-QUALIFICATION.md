# Qualification: horizon-v0.1.2

Commit `3e05877d069d44a43dac56ae4ea8e972a84fb01f`. READY: every claimed platform is QUALIFIED.

| Platform | Status | NOT_RUN / FAIL reasons |
|---|---|---|
| Windows x64 (`windows-x64`) | QUALIFIED | - |
| Linux x64 (Ubuntu 22.04) (`linux-x64`) | QUALIFIED | - |
| Linux arm64 (generic, hosted ubuntu-22.04-arm) (`linux-arm64`) | QUALIFIED | - |
| NVIDIA Jetson (Ubuntu 22.04 arm64, real hardware only) (`linux-arm64-jetson`) | QUALIFIED | - |
| macOS arm64 (`macos-arm64`) | QUALIFIED | - |
| macOS Intel x64 (`macos-x64`) | QUALIFIED | - |

A platform is QUALIFIED only when every required step PASSed in a record made for this exact commit (docs/how-to/qualify-desktop-install.md). Upgrade is NOT_RUN: no previous release exists.
