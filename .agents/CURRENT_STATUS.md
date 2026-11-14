# Current status

**Status revision:** 250

**Stage:** WP-19 S1 gate: episode harness (in progress)

**Summary:** fde/integration: S1 gate driver example (fde_s1_driver), Cognate program.run harness, three hash-pinned episode patches (events tail read, KG round trip test, readiness invalidation), SOP v2 candidate patch.

**Verified:**
- ep-1 patch: conformance_lifecycle 11 passed; ep-3 patch: cargo test readiness passed; ep-2 patch test fails as the hypothesis predicts (Policy triples 0 vs 1)

**Limits:**
- Not yet run end to end
- Patches are not regenerated into SFWP schemas

**Next:** runner wiring, episode orchestration, claim ledger

**Evidence:**
- scripts/fde-s1/
- crates/sea-forge-server/examples/fde_s1_driver.rs
