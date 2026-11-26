# Current status

**Status revision:** 390

**Stage:** CognitiveOS #108 hardening review fixes

**Summary:** Gate stored closure replay in the authority crate on current settlement authority, with a refusal test.

**Verified:**

- Closure replay by a caller without settlement authority is refused (cep_obligation closure_replay test, red before the fix); settle and obligation final replays were already gated on this branch and a refused final replay appends no settlement state (cep_settlement 83/83, cep_obligation 23/23).
- Revoked-owner replay and status over the socket are refused (conformance_cep_authority 14/14).

**Limits:**

- The settlement gate records its own authority_decision entry on each re-check, as every gate call does.
- The audit-granted arm of status is not re-gated by settlement authority.
- Full CI, proofs, cross-platform tests and the complete server suite have not run.
- Remaining hardening-review findings are in progress on this branch.

**Next:** Finish the remaining hardening-review fixes, then run the full gates.

**Evidence:**

- crates/sea-forge-authority/src/cep/obligation.rs
- crates/sea-forge-authority/tests/cep_obligation.rs
- crates/sea-forge-authority/tests/cep_settlement.rs
