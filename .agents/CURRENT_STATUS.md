# Current status

**Status revision:** 245

**Stage:** FDE integration branch: minimal governed revision, SEA-Forge half (WP-18)

**Summary:** fde/integration: a durable world pin record with promote and re-pin over the existing approval-gated semantic_change transition (cep::world_pin, wire verbs authority_promote, authority_repin, authority_pin). A promotion needs a recorded authority decision bound to the proposal digest (PROMOTION_DENIED, AUTHORITY_UNAVAILABLE, DECISION_EXPIRED, DECISION_DIGEST_MISMATCH), a passing RealityTrace regression matrix (REGRESSION_FAILED, LOST_CAPABILITY, UNLISTED_WEAKENING), and an approver other than the proposer; refused attempts and non-promotions are retained; settled evidence stays with its world_ref after promotion and re-pin. scripts/fde-proof.d AT-12, AT-13, NT-05, NT-07, NT-08, NT-21 added; AT-12 and AT-13 report PARTIAL by design.

**Verified:**
- cargo test -p sea-forge-authority --test world_pin: 13 passed; -p sea-forge-server --test conformance_world_pin: 1 passed; 0 failed
- gates: sea-forge-server lib 189, conformance_field_case 18, conformance_interaction_events 10, conformance_sfwp 23, conformance_identity 15, conformance_cep_authority 13; sea-forge-authority lib 60, cep_transition 17; 0 failed
- scripts/fde-proof.sh --slice s1 --only NT-05, NT-07, NT-08: PASS; --only AT-12, AT-13: PARTIAL
- mutation checks seen failing then restored: treat pending or expired as approved, skip digest comparison, skip regression, retarget refs, proposer approves own proposal (identity rule off in both layers)
- cargo clippy --workspace --all-targets --all-features -D warnings and cargo fmt --check pass; scripts/check-agent-context.sh and scripts/check-fde-contract-digests.sh pass
- RealityTrace half on fde/integration 4930b7d: cargo test --workspace 308 passed, 0 failed

**Limits:**
- AT-12 and AT-13 are PARTIAL: the promotion approval is an operator at assurance session, not a human-reserved A3 decision (S3); the AT-13 GAIN comes from a synthetic S1-form scenario, the three real program episodes are WP-19; R-58 is not claimed
- A matrix listing a weakening is refused ESCALATED_APPROVAL_REQUIRED: no escalated approval class exists in the CEP path
- Not done: freeze-time check that a new episode's world is the pinned world (no contract code), saga reconciler and crash-injection sweeps (S2), proposal.raised / semantic.diff ledger records, Horizon reads of the pin, independent hardening review
- Integration branch only: not merged to main, no PR

**Next:** WP-19 Horizon read path and the S1 gate (three real program episodes, claim-ledger builder); WP-22 re-pin and full saga; WP-15 and WP-33 as before.

**Evidence:**
- .agents/reports/fde-convergence-2026-11-12/evidence/s1/wp-18.md (in the sea-rs checkout)
- crates/sea-forge-authority/src/cep/world_pin.rs
- crates/sea-forge-authority/tests/world_pin.rs
- crates/sea-forge-server/tests/conformance_world_pin.rs
- scripts/fde-proof.d/AT-12.sh
