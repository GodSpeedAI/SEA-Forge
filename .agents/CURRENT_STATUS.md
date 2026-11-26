# Current status

**Status revision:** 389

**Stage:** CognitiveOS #108 hardening review fixes

**Summary:** Re-check current settlement authority before returning a stored settlement outcome by correlation replay or owner status.

**Verified:**

- A revoked owner is refused with cep_evidence_refused on replay of the same request id and on request_get_status, with no new non-decision ledger entries and an unchanged correlation record; an authorized owner still gets the replay (conformance_cep_authority, red before the fix, 14/14 after).
- The settle outcome now carries operation_id so a stored outcome can be re-gated; legacy outcomes without it are refused fail closed.

**Limits:**

- The settlement gate records its own authority_decision entry on each re-check, as every gate call does.
- The audit-granted arm of status is not re-gated by settlement authority.
- Full CI, proofs, cross-platform tests and the complete server suite have not run.
- Remaining hardening-review findings are in progress on this branch.

**Next:** Finish the remaining hardening-review fixes, then run the full gates.

**Evidence:**

- crates/sea-forge-server/tests/conformance_cep_authority.rs
- crates/sea-forge-server/src/cep_authority.rs
- crates/sea-forge-authority/src/cep.rs
