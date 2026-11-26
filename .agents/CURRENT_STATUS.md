# Current status

**Status revision:** 399

**Stage:** CognitiveOS #108 pass-3 authority fixes

**Summary:** Make obligation receipts projectable, refuse unprojectable receipts before the settlement commits, and gate settlement before existence errors.

**Verified:**

- Obligation receipt lineage is checked against the committed ALLOW decision of the same verified snapshot; a real-socket test with a stand-in RealityTrace binary issues an obligation receipt (red before: receipt_reference_mismatch after the commit) and a consistently rewritten malformed ALLOW world is refused with no settlement packet committed (red before: one packet committed).
- The settlement gate is evaluated before the no-decision, no-evidence and pending errors (red before: an unauthorized caller got 'no committed allow decision'); an allow is not recorded by the first evaluation, so polling appends nothing.
- conformance_cep_authority 22/22, sea-forge-authority tests pass.

**Limits:**

- Full CI, proofs and cross-platform tests have not run.
- Request ids remain cell-global; see the open debt entry.

**Next:** Server-side pass-3 fixes: claim lookup, status uniformity, effect flag and test injection.

**Evidence:**

- crates/sea-forge-authority/src/cep.rs
- crates/sea-forge-authority/src/cep/obligation.rs
- crates/sea-forge-authority/tests/cep_settlement.rs
- crates/sea-forge-server/tests/conformance_cep_authority.rs
