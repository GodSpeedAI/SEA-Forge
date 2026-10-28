# Current status

**Status revision:** 214

**Stage:** capability-harvest + ARM64: ARM64 packaging fully green (all four legs, run 40614104830); capability-harvest env-leak fix committed and pushed

**Summary:** ARM64 mission delivered: the arm64 .deb and tarball are built natively on ubuntu-22.04-arm (glibc 2.35), install-tested, smoke-tested on clean arm64 containers, independently verified (ELF AArch64, /lib/ld-linux-aarch64.so.1, max GLIBC_2.34, zero rpath/nix), and downloadable from run 40614104830 artifact pkg-linux-arm64. The release publisher now requires the arm64 assets. The capability-harvest 22.04 gate failure was the R4 test's process-wide unshare override leaking into parallel real-chain tests; the override is now mutex-scoped and removed after the test (034bbed8 lineage, push pending as the final commit).

**Verified:**

- run 40614104830: all four legs success; arm64 install-test log lines captured verbatim
- independent artifact verification on this workstation: built-from = run SHA, AArch64 ELF, correct interpreter, GLIBC<=2.34, zero rpath/nix
- sandbox lib 31 green with the env-scoped R4 test; fmt clean

**Limits:**

- Final capability-harvest lanes (env fix) pending; codex pass 3 after green
- ARM64 Milestone C (governed E2E) is pending on-device execution on the Jetson

**Next move:** Push the env fix, lanes green -> codex pass 3 -> consolidated final report.

**Evidence:**

- .agents/plans/2026-10-27-arm64-jetson-packaging.md
- run 40614104830
- /tmp/arm64-artifacts
