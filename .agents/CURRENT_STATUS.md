# Current status

**Status revision:** 209

**Stage:** ARM64 packaging: run 6 diagnosed the real ABI finding — Nix glibc 2.39 becomes a strong GLIBC_2.39 requirement on aarch64; arm64 now builds on the runner's system glibc 2.35 with a GLIBC floor check

**Summary:** Run 6: amd64 fully green (install, smoke, clean ubuntu:22.04 + debian:12 containers); arm64 progressed past all gates to the install test, which caught GLIBC_2.39 not found — the Devbox/Nix toolchain's glibc becomes a strong requirement on aarch64 (on amd64 it is weak-versioned, which is why that leg always ran). The arm64 leg now builds with the runner's native rustup toolchain (same pinned channel) and system gcc, binding glibc 2.35 so dpkg-shlibdeps derives the honest floor; a new GLIBC-floor check refuses any required version above 2.35 in the packaged binaries. amd64 keeps the Devbox path (its weak references pass; unification noted for later).

**Verified:**

- yaml parse ok; run 40602941845: amd64 clean ubuntu:22.04 + debian:12 installs pass

**Limits:**

- Run 7 pending; Jetson Milestone C on-device

**Next move:** Green run -> artifacts + hashes -> Jetson instructions.

**Evidence:**

- run 40602941845 logs
