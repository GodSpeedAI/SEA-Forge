# Current status

**Status revision:** 388

**Stage:** CognitiveOS #108 request correlation

**Summary:** Prove fail-closed orphan recovery after a real process death at the admission write boundary.

**Verified:**

- The Unix test starts a server child, verifies the committed admission claim and absent correlation record, then kills the child with SIGKILL at the test-only boundary pause.
- After restart over the same root, the same request is refused as idempotency_unverifiable, its status is denied, and no case or run exists.
- The focused crash test failed before the test-only pause was added and passed afterward, including the final no-run assertion.
- The preceding checkpoint passed SFWP socket conformance 24/24, case-authoring socket conformance 21/21, correlation unit tests 15/15, two claim unit tests and the exact-request audit policy test.

**Limits:**

- The boundary pause is compiled only into Unix test builds; this test does not establish power-loss durability or exercise non-Unix platforms.
- The isolated implementation requires independent hardening review; the owner declined external disclosure of the review packet.
- Full CI, proofs, cross-platform tests and the complete server suite have not run.

**Next:** Independently review and qualify the isolated checkpoints and run the remaining required milestone and repository gates before integration.

**Evidence:**

- crates/sea-forge-server/src/admission_crash_tests.rs
- crates/sea-forge-server/tests/conformance_sfwp.rs
- crates/sea-forge-authority/tests/request_status_audit.rs
