# Current status

**Status revision:** 385

**Stage:** CognitiveOS #108 request correlation

**Summary:** Reject aliased correlation identifiers before reading another request record.

**Verified:**

- Normal and obligation replay tests fail before the change for an operator without settlement authority and pass after it.
- A service whose settlement grant is removed is refused, while existing authorized normal and obligation replay tests pass.
- The colliding-request-ID regression failed before the exact stored-ID check and passed after it using the canonical crate test.

**Limits:**

- Authenticated admission ownership and exact-request auditor policy remain unimplemented.
- Service replay and correlation changes are isolated and not independently reviewed or integrated.
- Full CI, proofs and real-socket authorization journeys have not run.

**Next:** Implement source-owned admission binding, crash-safe orphan handling and default-deny request-scoped audit authority, then validate at the socket boundary.

**Evidence:**

- crates/sea-forge-authority/tests/cep_settlement.rs
- crates/sea-forge-authority/tests/cep_obligation.rs
