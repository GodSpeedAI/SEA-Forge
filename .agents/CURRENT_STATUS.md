# Current status

**Status revision:** 249

**Stage:** FDE review fixes: F3 F4 F8 F9 F10 world pin and request caps (WP-18, WP-14)

**Summary:** fde/integration: promotion is bound to the matrix the approver saw (matrix_id and held-out rule claims), an empty or non-covering matrix is refused, the server validates the matrix against the vendored regression-matrix.v1; promote and re-pin run under one lock with a re-check and a per-position ledger key (PIN_STALE for the loser), a replay after a re-pin is PIN_SUPERSEDED, the baseline is explicit configuration (pinned_baseline) and an ambiguous one refuses to move the pin; refused-attempt records are size-capped and rate-limited; case_record_append payloads are capped at 256 KiB; recover_field_case takes the case lock.

**Verified:**
- cargo clippy --workspace --all-targets --all-features --locked -D warnings and cargo fmt --check pass
- server lib 190, conformance_field_case 31, conformance_interaction_events 10, conformance_world_pin 4, conformance_sfwp 23, conformance_identity 15, conformance_cep_authority 13; authority lib 60, world_pin 21, cep_transition 17; 0 failed
- scripts/fde-proof.sh --slice s1 --only AT-04, NT-01, NT-16, NT-05, NT-07, NT-08: PASS; AT-12, AT-13: PARTIAL as before
- mutations seen failing then restored for each guard

**Limits:**
- SEA-Forge cannot recompute matrix transitions without RealityTrace; the approver's matrix_id covers which matrix, not that it is true
- Freeze does not refuse a world that is not the pinned one: no usable code in case-record-append.v1 (missing WORLD_NOT_PINNED)
- Execution-side I-1 gate unwired: no entry point carries n or episode_ref

**Next:** WP-19 Horizon read path and the S1 gate; WP-20 delegation payload episode_ref

**Evidence:**
- .agents/reports/fde-convergence-2026-11-12/evidence/s1/wp-18.md (in the sea-rs checkout)
- .agents/reports/fde-convergence-2026-11-12/evidence/s1/wp-14.md (in the sea-rs checkout)
- crates/sea-forge-authority/src/cep/world_pin.rs
- crates/sea-forge-server/src/cep_authority.rs
- crates/sea-forge-server/contracts/rt/regression-matrix.v1.json
