# Current status

**Status revision:** 193

**Stage:** capability-harvest: REQUIRED CI GATE GREEN (round 8) — windows junction-test semantics corrected for the release-pin reality; artifact-name collision fixed

**Summary:** Round 8: the full ci.yml gate (lint, test, msrv, workbench, macos, authority-opa) is green, including the active-cgroup lane (conformance_m1 19 with the network-denial assertion, setsid_lifecycle 9, runtime 5) — the registry-scoped sweep and the spawn/finish jail split hold on real runners. Windows progressed to the safe_write step: sea-forge-os unit tests pass (the release-pin rename works, the hard-link replace is green); the one failure is the W6 junction test asserting path-based create_dir fails under the pin — on Server 2025 path-based entry creation is DACL-only, not share-gated (the same kernel distinction the rename conflict exposed), so the test is rewritten to assert the true load-bearing semantics: write-class HANDLES on the pinned anchor are refused (the FSCTL_SET_REPARSE_POINT gate), a planted junction entry must not redirect the contained write, and the outside target stays empty. kernel-security 22.04's sandbox-suite FAIL left no evidence because both matrix jobs uploaded artifacts under the same name (24.04 overwrote 22.04); the name now includes the matrix image; 24.04 passed and the suite exits 0 in the equivalent local configuration.

**Verified:**

- gh run 40440809377: gate success — every required job green
- windows-target and host clippy -D warnings green on the rewritten test; fmt clean

**Limits:**

- Round 9 must confirm: windows lane green end-to-end, and the 22.04 socket-gate verdict with retained evidence

**Next move:** Push round 9; on green, freeze and run the codex adversarial review.

**Evidence:**

- gh runs 40440809377 / 40440803522 / 40440809219
- .agents/OBSERVED_DEBT.md
