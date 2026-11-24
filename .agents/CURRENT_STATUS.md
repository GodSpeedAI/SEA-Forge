# Current status

**Status revision:** 387

**Stage:** CognitiveOS #108 request correlation

**Summary:** Checkpoint source-owned admission claims and request-scoped audit access for further qualification.

**Verified:**

- The legacy hashless replay regression failed before the change and passed afterward.
- Two focused admission-claim tests passed, including an incomplete ledger refusal.
- The exact-request audit policy test failed for an unsupported operation before the change and passed afterward for exact, different and unconstrained rules.
- SFWP socket conformance passed 24 tests, including owner and auditor access after restart, a simulated orphan and a foreign-owner retry.
- Case-authoring socket conformance passed 21 tests.

**Limits:**

- The implementation remains isolated and requires independent hardening review; the owner declined external disclosure of the review packet.
- The orphan test removes a projection to simulate an interrupted write; a process crash at the actual write boundary has not been proven.
- Full CI, proofs, cross-platform tests and the complete server suite have not run.
- An initial unprivileged socket test was denied loopback access; the same authorized test and socket suites passed with normal elevated access.

**Next:** Review and qualify the checkpoint, including real crash-boundary recovery and all required milestone and repository gates, before integration.

**Evidence:**

- crates/sea-forge-server/src/request_admission_claim.rs
- crates/sea-forge-server/tests/conformance_sfwp.rs
- crates/sea-forge-authority/tests/request_status_audit.rs
