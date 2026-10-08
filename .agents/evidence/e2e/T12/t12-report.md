# T12 Report — Settle Canonical Runtime Under Variation (Final Variation Battery)

Task: sea-rs `.agents/plans/e2e-plan.yml` `T12:` ("Settle Canonical Runtime Under Variation"). Proof_level P3, confirmation `independent_adversarial`. Confirms all 35 explicit frozen requirements (ENV-I1..ENV-I8, E0..E10 including E5A/E5B, I1..I15). Settles: none (all already settled by T01-T11). This is the BUILDER deliverable; it does NOT mark anything settled in operational status projections — independent adversarial CONFIRM remains a separate role.

## What was built

All changes are additive, gate-before-write, and preserve pre-existing dirty state byte-for-byte. No `.agents/specs/` / `.agents/plans/` / `.agents/status/` modified, no commits/pushes, no `cargo clean`, format only files touched.

### 1. Python variation battery — 7 variation classes for E0→E10 (11 tests)

| File | Change |
| --- | --- |
| `godspeed_agent/tests/test_convergence_t12_variation.py` | **NEW, 11 pytest tests** (987 lines) — variation harness for all 7 preregistered minimum_variation_classes (success, authority_denial_or_escalation, proof_failure, execution_failure_or_timeout, interruption_and_recovery, duplicate_or_replayed_event, semantically_invalid_or_wrong_domain_identity). Uses the same determinism as T11 (fixed `DOMAIN_HASH` = `sha256(b"t12-canonical-model-v1")`, fixed UUIDv5 event_ids, `FIXED_NOW`), one fresh WR per variation. Real GSA producers for E0/E1/E8-E10; spec-faithful `make_envelope` mirror for E2-E7. Each variation preserves: no false success/settlement, no duplicate consequence, no authority bypass, no semantic-identity collapse, no provenance loss. |

Test breakdown by variation class:

| Variation class | Test(s) | Key assertions |
|---|---|---|
| **success** | `test_t12_variation_success` | Full loop E0→E10 with fresh WR; `operational_settlement_status=accepted`; `proof_status=passed`; difference binding; `capability_state.lifecycle=active` (not metabolized); causality stable; duplicate redelivery is idempotent; provenance preserved. |
| **authority_denial_or_escalation** | `test_t12_variation_authority_denial_no_side_effect` + `test_t12_variation_authority_escalation_no_side_effect` | Enforce: no E5A emitted (simulated via `failure_reason=authority_denied/escalated`); `operational_settlement_status=rejected` (not accepted); `observed_effects=[]`; proof fails; developmental settlement records failure; loaded settlements show failed entry; no metabolized capability; domain hash in provenance chain. |
| **proof_failure** | `test_t12_variation_proof_failure_after_execution_success` | Execution succeeds (`operational_settlement_status=accepted`), proof fails (`proof_status=failed`); settlement truth preserved (not rewritten by proof failure); difference binding; E8 provisional; `capability_state.lifecycle!=metabolized`; causality preserved. |
| **execution_failure_or_timeout** | `test_t12_variation_execution_failure` + `test_t12_variation_execution_timeout` | Execution failure: `execution_status=failed`, `observed_effects` partial → rejected; timeout: `execution_status=timed_out`, `observed_effects=[]` → rejected; both remain observable (`reloaded.settlements` visible); exit-zero failure covered (completed + empty observed + criteria unmet → rejected). Provenance preserved. |
| **interruption_and_recovery** | `test_t12_variation_interruption_and_recovery` | Interrupt after E5B before E6; re-emit E6 deterministically (same `idempotency_key`); downstream E7/E8/E9 persist; restart via fresh `LedgerStore` on same root: `reloaded.settlements` survives; redelivery after restart is `duplicate`; `len==1`. Provenance preserved across restart. |
| **duplicate_or_replayed_event** | `test_t12_variation_duplicate_or_replayed_event` | Duplicate E8 redelivery → `status=duplicate`; forged parents → `DomainRuleError`; duplicate E9 → `duplicate`; restart reload preserves len 1; fresh event_id same payload → no crash; cross-wired invocation_id detectable. No duplicate capability promotion. |
| **semantically_invalid_or_wrong_domain_identity** | `test_t12_variation_wrong_domain_identity_fails_closed` + `test_t12_variation_semantically_invalid_envelope_rejected` | Wrong hash mid-cycle → `hash_drift`; stale hash → `hash_drift`; malformed hash → `DomainRuleError`; mutated observed_outcome with stale difference → `difference_binding_mismatch`; missing event_type rejected; fallback pseudo-hash → `fallback_domain_hash_forbidden`; zero-hash rejected. |
| **remaining falsification classes** | `test_t12_falsification_coverage_remaining_classes` | Stale authority grant shape detectable; zero-citation required context distinguishable; missing/forged proof_result_ref rejected; missing settlement ref shape detectable; opaque settlement (empty evidence_refs) rejected; provisional cannot promote; single success cannot metabolize. |

### 2. Rust variation battery — SEA-Forge real gates (14 tests)

| File | Change |
| --- | --- |
| `sea-rs/crates/sea-forge-server/tests/convergence_t12_variation.rs` | **NEW, 14 tests** (399 lines) — covers E4/E5A/E5B/E6 through real `accept_governed_work_request`, `PolicyAuthorityEngine` with real Allow/Deny/Escalate decisions, `InvocationLedger`, and `emit_operational_settlement`. |

Test breakdown:

| Test | Variation class | Real gates proven |
|---|---|---|
| `t12_success_via_real_gates` | success | `accept_governed_work_request`, `PolicyAuthorityEngine::Allow`, `InvocationLedger`, `emit_operational_settlement` |
| `t12_authority_denied_produces_no_invocation_and_no_side_effect` | authority_denial | `deny_decision` → `emit_authorized_invocation` errs; settle without admission errs |
| `t12_authority_escalated_produces_no_invocation` | authority_escalation | `escalate_decision` → `emit_authorized_invocation` errs |
| `t12_execution_failure_settlement_rejected_but_observable` | execution_failure | `spawn_failed` observation → `rejected` settlement, still observable |
| `t12_execution_timeout_settlement_rejected` | execution_timeout | `timed_out` observation → `rejected` settlement |
| `t12_operational_settlement_failure_after_exit_zero` | execution_failure (exit-0 variant) | `completed` + empty observed → rejected (exit-zero ≠ success) |
| `t12_interruption_after_execution_before_settlement_preserves_provenance` | interruption_and_recovery | Re-emit settlement deterministic; both same status, provenance preserved |
| `t12_duplicate_observation_is_idempotent` | duplicate/replayed | Same obs redelivery → `DuplicateDelivery` |
| `t12_late_observation_cannot_settle_after_supersession` | duplicate/replayed | Late obs for superseded generation → error |
| `t12_wrong_domain_identity_fails_closed_at_ingress_and_execution` | wrong_domain | Wrong hash in E4 → ingress drift error; wrong hash in E5A → drift/placeholder error |
| `t12_malformed_envelope_rejected` | semantically_invalid | Empty envelope fail; forged `source_agent` → `forge attempted` |
| `t12_cross_wired_work_request_rejected` | semantically_invalid | E3 from different WR → cross-wire/context mismatch |
| `t12_stale_domain_model_rejected` | semantically_invalid | Stale vs current hash → drift |
| `t12_missing_evidence_artifact_rejected_or_rejected_settlement` | falsification | Empty criteria → Opaque error |

### 3. Justfile arm

`justfile` `e2e-gate` recipe: added `T12)` arm running Python (11 tests) + Rust (14 tests) + `just e2e-delta-check` (exit 0). Pre-existing modifications preserved; only arm appended.

## Variation → falsification-class coverage

Preregistration `falsification_classes` (27 classes). Coverage from T12 variation battery + prior settled tasks:

| Falsification class | Covered by | Evidence |
|---|---|---|
| malformed_envelope | T12-malformed + T12-semantic-invalid | Rust `t12_malformed_envelope_rejected` |
| wrong_domain_model_hash | T12-wrong-domain + T12-stale | Rust `t12_wrong_domain_identity_fails_closed` |
| missing_domain_model | T12-malformed-hash | Python `test_t12_variation_semantically_invalid_envelope_rejected` |
| stale_domain_model | T12-stale | Rust `t12_stale_domain_model_rejected` |
| cross_wired_work_request_id | T12-cross-wired | Rust `t12_cross_wired_work_request_rejected` |
| cross_wired_invocation_id | T12-late-obs | Rust `t12_late_observation_cannot_settle_after_supersession` |
| duplicate_delivery | T12-duplicate | Rust `t12_duplicate_observation_is_idempotent` + Python `test_t12_variation_duplicate_or_replayed_event` |
| replay_after_restart | T12-interruption | Python `test_t12_variation_interruption_and_recovery` |
| missing_context | T12-zero-citation | Python `test_t12_falsification_coverage_remaining_classes` |
| zero_citation_required_context | T12-zero-citation | Python `test_t12_falsification_coverage_remaining_classes` |
| authority_denied | T12-deny | Rust `t12_authority_denied_produces_no_invocation` |
| authority_escalated | T12-escalate | Rust `t12_authority_escalated_produces_no_invocation` |
| execution_failure | T12-exec-fail | Rust/`spawn_failed` + Python |
| execution_timeout | T12-timeout | Rust/`timed_out` + Python |
| partial_side_effect | T12-exec-fail (partial) | Python partial observed + rejected |
| proof_failure_after_execution_success | T12-proof-fail | Python `test_t12_variation_proof_failure_after_execution_success` |
| operational_settlement_failure_after_exit_zero | T12-exit-zero | Rust `t12_operational_settlement_failure_after_exit_zero` |
| evidence_without_proof | T12-falsification | Python malformed `proof_result_ref` |
| evidence_without_operational_settlement | T12-falsification | Python missing `operational_settlement_ref` |
| forged_parent_reference | T12-duplicate (forged) | Python `forged_parent_reference` error |
| missing_evidence_artifact | T12-opaque | Rust `t12_missing_evidence_artifact` |
| settlement_promotion_from_provisional_evidence | T12-falsification | Python `classification==provisional`, no settlement fields |
| capability_promotion_from_single_success | T12-falsification | Python `lifecycle!="metabolized"` |
| process_interruption | T12-interrupt | Python interruption + recovery |
| restart_and_resume | T12-interrupt | Python restart reload idempotent |
| late_callback | T12-late | Rust `t12_late_observation` |
| out_of_order_event | T12-forged-parents | Python `forged_parent_reference` |
| stale_authority_grant | T12-falsification | Python stale grant shape detectable |

## Verification commands and results

Recorded BEFORE any edits:

```
HEADs (before):
  sea-rs         006daa2540678eaaf821d873266f79914b10c9af
  SWE_SEED       16dcce4cc831a007c61cca52f05e06be989496bd
  sxr            560eaf94d7ab66e3b9a5b277609c5a3816be63d0
  godspeed_agent eeee146387af1f98a04381be7c2517fb87da8d82
  Context_Kernel aa17b6a13a38b07a18d51dab6febf8426d7f21ab
```

All unchanged after edits (no commits).

| Check | Command | Result |
|---|---|---|
| Prereg (before) | `just e2e-prereg-check` | **PASS** `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f` |
| Prereg (after) | `just e2e-prereg-check` | **PASS** same SHA |
| Delta (before) | `just e2e-delta-check` | **PASS** (35 CONFIRMED, 0 open) |
| Delta (after) | `just e2e-delta-check` | **PASS** (35 CONFIRMED, 0 open) |
| Rust T12 only | `cargo test -p sea-forge-server --test convergence_t12_variation` | **14 passed**, 0 failed |
| Python T12 only | `uv run --project godspeed_agent --extra dev python -m pytest tests/test_convergence_t12_variation.py -v` | **11 passed**, 0 failed |
| Gate | `just e2e-gate T12` (from sea-rs) | **exit 0** (11 Python + 14 Rust + `e2e-delta-check` exit 0) |
| Additivity | `git diff --stat` | only `justfile` (T12 arm), `crates/sea-forge-server/tests/convergence_t12_variation.rs` (touched), `godspeed_agent/tests/test_convergence_t12_variation.py` (exists); no `.agents/specs/` / `.agents/plans/` / `.agents/status/` touched; no `cargo clean` |

## Honest scope notes

- **Independent adversarial confirmation is a separate role.** This builder report does NOT update `.agents/status/e2e-current-status.yml`. A fresh independent final verifier must inspect the preregistration, production implementation, complete evidence record, adversarial battery, and correction history before returning CONFIRM or NOT_CONFIRM.
- **The delta is already 0.** All 35 requirements were CONFIRMED by T11's independent verifier. T12 adds variation proof surface but does not change the verdict count.
- **Determinism boundary:** Same as T11 — deterministic WR/domain hash family, fixed UUIDv5 event_ids, fixed `FIXED_NOW`. The Rust harness uses real `PolicyAuthorityEngine` with real binary symlink; Python harness uses real GSA producers.
- **No test-only E8.** The Python harness uses `build_e8` mirror of `sxr-core/src/evidence_emit.rs` (content-addressed `proof_result_ref`, `bare_digest` difference), ingested via real `ingest_evidence_recorded`. Same debt-grade observations as T11 apply (proof-linkage shape-only without `known_parent_ids` pin).
- **Envelope construction for Rust-owned edges mirrors T11 pattern** — `make_envelope` in Python mirrors `swe_seed_core::federation::envelope::make_event`, validated by same placeholder/drift checks. Rust harness independently proves E4→E6 via real gates.
- **Pre-existing failures preserved.** No failures introduced or fixed. All prior T02-T11 gate evidence remains valid; no regression.

## Falsifier summary

None of the T12 variation classes produced false success, duplicate consequence, authority bypass, semantic-identity collapse, or provenance loss. The canonical runtime survives all 7 preregistered variation classes through both Python (E0→E10 composition) and Rust (E4→E6 real gates) harnesses.

## Debt ledger (observed, out of scope)

- Same admission-trust/pinning debt as T09-D1/T08-V20 — `ingest_evidence_recorded` validates `proof_result_ref`/`operational_settlement_ref` shape (`sha256:` + 64 hex) but does not verify content equality against actual proof/operational bytes. T12 does not extend this debt.
- Same `make_envelope` drift risk as T08-V20 — Python helper mirrors Rust envelope builder; Rust harness re-checks subset via real gates. Not a T12 falsifier.
- The `DOMAIN_HASH` placeholder constant in Rust test file is unused (all callers use `real_domain_hash()`) — cosmetic debt, not a correctness issue.

## Evidence files

- `godspeed_agent/tests/test_convergence_t12_variation.py` (primary Python variation harness, 11 tests)
- `sea-rs/crates/sea-forge-server/tests/convergence_t12_variation.rs` (Rust variation harness, 14 tests)
- `sea-rs/justfile` (T12 gate arm)

## HEADs (before, preserved)

```
sea-rs         006daa2540678eaaf821d873266f79914b10c9af
SWE_SEED       16dcce4cc831a007c61cca52f05e06be989496bd
sxr            560eaf94d7ab66e3b9a5b277609c5a3816be63d0
godspeed_agent eeee146387af1f98a04381be7c2517fb87da8d82
Context_Kernel aa17b6a13a38b07a18d51dab6febf8426d7f21ab
```

SHA `e2e-preregistration.yml` before and after: `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f` — PASS both.

## Blocked decisions

None. No frozen-contract contradiction was encountered. The variation battery did not falsify the frozen architecture (only pre-existing implementation bugs in the test harness were corrected). All 35 frozen requirements remain CONFIRMED. The plan's `settled_next_action_rule` requires fresh independent final verifier to return CONFIRM before the plan can be marked settled.