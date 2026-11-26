# Current status

**Status revision:** 400

**Stage:** CognitiveOS #108 pass-3 server fixes

**Summary:** Uniform status for non-owners, claim lookup on the verified snapshot, truthful spent-id flag and deterministic test injection.

**Verified:**

- request.get_status returns the never-seen reply to every non-owner without the audit grant, including legacy records and unreadable claim ledgers (conformance_sfwp 26/26; red before: request_status_denied).
- Claim lookup uses verified_snapshot_locked and refuses a symlinked entries file and a missing lock file (unit tests); request_id_spent now reports no_side_effect false; the failure is injected by a directory at the correlation tmp path, server_child is ignored and fails without its environment, and a second server on the same cell root is refused.
- lib 208 passed 1 ignored; case_authoring 21, lifecycle 10, FDE scenarios 49 (FDE_WP21A_FINAL=1), ledger tests, contract digests, workbench contracts gate, clippy -D warnings and fmt clean.

**Limits:**

- Request ids remain cell-global: per-owner scoping of claims and correlation needs a status contract change and an owner-qualified audit grant, recorded as debt with measured per-request cost (one directory, three files, one rename, two fsyncs).
- Repeated denied status polls are audited once per process and actor.
- Full CI, proofs and cross-platform tests have not run.

**Next:** Independent re-review, then integrate.

**Evidence:**

- crates/sea-forge-server/src/lib.rs
- crates/sea-forge-server/src/request_admission_claim.rs
- crates/sea-forge-server/src/admission_crash_tests.rs
- crates/sea-forge-server/tests/conformance_sfwp.rs
- docs/decisions/ADR-016-minimal-settlement-receipt.md
- .agents/OBSERVED_DEBT.md
