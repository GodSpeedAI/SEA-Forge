# Current status

**Status revision:** 401

**Stage:** CognitiveOS #108 delta-check authority fixes

**Summary:** One projectability predicate checked before and after commit, settlement authority ahead of every pre-gate error, and bounded operation ids before the gate.

**Verified:**

- An inconsistent obligation run set is refused receipt_nonfinal_refused before it commits and stays obtainable through authority_settle; a contradicted obligation yields a rejected receipt; a second receipt is a replay refusal.
- Unauthorized callers get the same gate refusal on receipt, settle (RealityTrace not configured), and closure paths for real and never-decided operations; malformed operation ids record nothing, well-formed denials are audited.
- Three of the new socket tests failed on the previous sources and pass now; conformance_cep_authority 25/25, sea-forge-authority 60+ tests, lib 208 passed 1 ignored, sfwp 26, case_authoring 21, lifecycle 10, FDE scenarios 49, ledger tests, digests, contracts gate, clippy -D warnings, fmt.

**Limits:**

- The decision envelope id is deterministic from the ALLOW row's operation id, not stored in the row; documented in ADR-016.
- Preflight and commit are not atomic (documented).
- Request ids remain cell-global (open debt).
- Full CI, proofs and cross-platform tests have not run.

**Next:** Independent re-review, then integrate.

**Evidence:**

- crates/sea-forge-authority/src/cep.rs
- crates/sea-forge-authority/src/cep/obligation.rs
- crates/sea-forge-server/src/cep_authority.rs
- crates/sea-forge-server/tests/conformance_cep_authority.rs
- docs/decisions/ADR-016-minimal-settlement-receipt.md
