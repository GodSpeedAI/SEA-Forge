# Current status

**Status revision:** 396

**Stage:** CognitiveOS #108 receipt integrity

**Summary:** Add a locked, symlink-refusing, chain-verified pinned ledger snapshot for receipt verification.

**Verified:**

- verified_snapshot_locked reads once under a required shared lock, verifies chain, MMR and legacy files, and refuses a symlinked stream path, a missing lock file and an edited earlier row (sea-forge-ledger verified_snapshot 4/4).

**Limits:**

- The symlink refusal is check-then-open, not an anchored no-follow open; hard links are not detected; the chain and MMR are unsigned local files.
- The receipt verb is not yet wired to it in this commit.
- Full CI, proofs and cross-platform tests have not run.

**Next:** Bind receipt references to the committed ALLOW record and wire verification.

**Evidence:**

- crates/sea-forge-ledger/src/types.rs
- crates/sea-forge-ledger/tests/verified_snapshot.rs
