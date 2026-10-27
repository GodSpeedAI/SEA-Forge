# Current status

**Status revision:** 204

**Stage:** ARM64 Jetson packaging: implementation complete (build.sh arch param, matrix packaging job, per-arch markers, publisher contract); pushing for the packaging run

**Summary:** Probe verified both hosted arm64 runners; ubuntu-22.04-arm has glibc 2.35 = the Jetson ABI, so the arm64 job builds natively (no cross-compilation, dpkg-shlibdeps derivation is payload-faithful). build.sh now takes a validated deb-arch parameter and gates every staged executable against the claimed ELF machine before packaging. The linux job is a matrix (amd64 ubuntu-22.04 / arm64 ubuntu-22.04-arm) with target-specific ELF interpreter handling (/lib/ld-linux-aarch64.so.1 for arm64), per-arch built-from markers, and per-arch artifacts. The release publisher requires both arm64 and amd64 artifacts, checks both markers, and extends SHA256SUMS.

**Verified:**

- probe run 40591820969: ubuntu-22.04-arm aarch64 glibc 2.35; yaml parse ok; bash -n ok

**Limits:**

- Packaging run pending; Jetson Milestone C is on-device

**Next move:** Commit, push, trigger package.yml on the branch, collect artifacts and hashes.

**Evidence:**

- .github/workflows/arm-probe.yml run 40591820969
