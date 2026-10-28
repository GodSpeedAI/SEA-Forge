# Current status

**Status revision:** 212

**Stage:** ARM64: stale amd64 deb name in the smoke reinstall step fixed; capability-harvest: R4 test env leak fixed

**Summary:** ARM64 run 7's arm64 job reached the smoke reinstall step (all gates and the first install test PASSED — the system-glibc build works) and failed only on a leftover hardcoded amd64 .deb name in the second install step. The capability-harvest 22.04 gate caught the R4 test leaking SEA_FORGE_TEST_UNSHARE=/bin/false process-wide (parallel real-chain tests inherited it); the override is now mutex-scoped and removed after the test.

**Verified:**

- yaml ok; sandbox lib 31 green after the env fix

**Limits:**

- Reruns pending

**Next move:** Push both fixes; runs green -> artifacts + codex pass 3.

**Evidence:**

- runs 40608173352 / 40605391104 logs
