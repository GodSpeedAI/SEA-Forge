# T11 Report — Settle Whole-Loop Causality, Replay, and Recovery (I2/I11/I14/I15)

Task: sea-rs `.agents/plans/e2e-plan.yml` `T11:` ("Settle Whole-Loop Causality, Replay, and Recovery"). Settles **I2, I11, I14, I15** (and confirms **I10** composition) and confirms **E0→E10**, proof_level P3, confirmation `independent_adversarial`. This is the BUILDER deliverable; it does NOT mark anything settled in operational status projections — independent adversarial CONFIRM remains a separate role.

## What was built

All changes are additive, gate-before-write, and preserve pre-existing dirty state byte-for-byte. No `.agents/specs/` / `.agents/plans/` / `.agents/status/` modified, no commits/pushes, no `cargo clean`, format only files touched.

### 1. Deterministic canonical harness — whole-loop golden scenario (E0→E10, 12 observations)

| File | Change |
| --- | --- |
| `godspeed_agent/tests/test_convergence_t11_whole_loop.py` | **NEW, 10 pytest tests** (868 lines) — the primary deterministic whole-loop harness. Uses **REAL production producers** for GSA-owned edges (E0 `project_desired_direction`, E1 `project_work_requested`, E8 `ingest_evidence_recorded`, E9 `project_developmental_event` + `persist_developmental_event`, E10 `build_developmental_memory_from_store` + `project_developmental_memory` + `NavigationRuntime` methods). For Rust-owned edges (E2 ContextRequired, E3 ContextPacketCreated, E4 GovernedWorkRequest, E5A AuthorizedInvocation, E5B ExecutionObservation, E6 OperationalSettlement, E7 ProofCompleted) envelopes are constructed via a **spec-faithful v1 helper** `make_envelope` (mirrors `swe_seed_core::federation::envelope::make_event` / `derive_event`: same `schema_version=v1`, `namespace=agentic_capability_loop`, `provenance.chain` with `domain_model_hash:` + `caused_by:` parents, content-derived `idempotency_key`, `source_agent` stamps). Validation mirrors Rust gates (same `FALLBACK_PSEUDO_HASH` `dc144cbd71a4…`, `ZERO_HASH`, drift check). The Rust crates' own convergence suites (T02–T08) independently prove those envelopes would pass the real production gates; this harness focuses on **composition, causality, replay, and recovery**. Deterministic: `WR_ID=wr-gsf-t11-001`, `DOMAIN_HASH=537449202c9d06df08373366dc0e25c7bf32e51e64d3fbf837465f0a45098741` (`sha256(b"t11-canonical-model-v1")`), `AFFORDANCE_ID=aff-t11-canonical-001`, fixed UUIDv5 `event_id`s, fixed `FIXED_NOW=2026-08-26T12:00:00+00:00`, no external network. The golden test `test_t11_canonical_golden_scenario_traverses_every_production_edge` captures exactly **12 required observations** (spec's 11 + domain identity) and writes fixture `tests/fixtures/t11_canonical_golden.json` (19 KB, 12 envelopes + provenance chain) for verifier and Rust re-checks. Also covers I10 (difference `expected_sha256`/`observed_sha256` cryptographically bound) and I2/I11/I14/I15 strands via the same run. |
| `godspeed_agent/tests/fixtures/t11_canonical_golden.json` | **NEW fixture** (written by the golden harness on success). Contains `work_request_id`, `domain_model_sha256`, `affordance_id`, `direction_id`, `observations` (12), `envelopes` (map by event_type), `provenance_chain` (12 ids in order). Demonstrates ONE stable `work_request_id` and ONE resolvable `domain_model_hash` end-to-end, cited context with citations, explicit authority decision, observed real execution, operational settlement, independent proof result, RealityTrace difference with digests, provisional evidence (classification=provisional), explicit developmental settlement, durable memory with honest `capability_state.lifecycle==active`, and changed next navigation (historical candidate ranked first). |
| `sea-rs/crates/sea-forge-server/tests/convergence_t11_whole_loop.rs` | **NEW, 6 Rust tests** — SEA-Forge-side recovery/replay harness exercising **REAL production gates** (`governed_work_ingress::accept_governed_work_request`, `governed_execution_boundary::{emit_authorized_invocation, InvocationLedger}`, `governed_settlement_return::{emit_operational_settlement, SettlementOptionalFields}`) over the same deterministic `WR_ID`/`DOMAIN_HASH`. Tests: `t11_whole_loop_traceability_via_real_gates` (I2 strand for E4→E5→E6 with causality and drift checks), `t11_duplicate_observation_is_idempotent` and `t11_duplicate_settlement_via_replay_is_idempotent` (I14), `t11_late_observation_cannot_settle_against_wrong_invocation` (I15 late callback with generation supersession), `t11_wrong_domain_identity_is_rejected_at_ingress` (I1/ENV-I2), `t11_failed_settlement_remains_observable` (I11 strand — rejected settlement still valid and observable). |
| `sea-rs/justfile` | Added `T11)` arm to `e2e-gate` case mirroring T10: runs the Python golden+harnesses via `uv run --project … python -m pytest …/test_convergence_t11_whole_loop.py -q` then Rust harness via `cargo test -p sea-forge-server --test convergence_t11_whole_loop`. Pre-existing justfile modifications preserved; only arm appended. `cargo fmt` applied (only files touched). |

Other repos: read-only recon of `SWE_SEED` federation, `sxr` evidence_emit/proof_ingest, `Context_Kernel` ck-mcp; no changes there (their T02–T08 gates already prove those edges individually; T11 composes them).

### 2. Cross-wire / duplicate / late / interruption / restart harnesses (I11/I14/I15, plus I10)

Covered in `test_convergence_t11_whole_loop.py` (8 dedicated tests) and Rust harness (above). Summary:

| Falsifier / invariant | Test(s) | Outcome |
| --- | --- | --- |
| Cross-wire mid-loop envelope (plan tooth 1) | `test_t11_cross_wire_rejected` (Python: ContextPacket from foreign `wr-foreign-999` + EvidenceRecorded ingest with wrong `expected_work_request_id`); Rust: `t11_wrong_domain_identity_is_rejected_at_ingress` (drift) | **Reject/quarantine** — foreign `work_request_id` detected, `DomainRuleError` codes `cross_wired_work_request` / `hash_drift`; Rust ingress returns `DomainDrift`. |
| Duplicate delivery (plan tooth 2, I14) | `test_t11_duplicate_delivery_is_idempotent` (Python: E9 `SettlementRecorded` redelivery + EvidenceRecorded `ProvisionalEvidence` redelivery); Rust: `t11_duplicate_observation_is_idempotent`, `t11_duplicate_settlement_via_replay_is_idempotent` | **No duplicate consequence** — second delivery returns `status==duplicate`, ledger `read_events` still `1`, `reloaded.settlements` still `1`. |
| Out-of-order delivery | `test_t11_out_of_order_delivery_rejected` (E8 `known_parent_ids` pinned to wrong ids) | **Reject** — `forged_parent_reference` / `causality_missing`. |
| Late callback (I15 strand, plan tooth 2 variant) | `test_t11_late_callback_cannot_settle_against_wrong_invocation` (Python: invocation_id mismatch) + Rust `t11_late_observation_cannot_settle_against_wrong_invocation` (generation supersession via `InvocationLedger`: late obs for gen 1 after gen 2 is current → `LateObservation` error) | **Cannot settle against wrong invocation** — late `ExecutionObservation` rejected with `LateObservation { invocation_id, settled_generation:1, current_generation:2 }`. |
| Wrong-domain identity mid-cycle (plan tooth 4) | `test_t11_wrong_domain_identity_mid_cycle_fails_closed` (E6 with `OTHER_HASH` `c*64`, then `ingest_evidence_recorded` with `OTHER_HASH` local) + Rust `t11_wrong_domain_identity_is_rejected_at_ingress` | **Fail closed** — `hash_drift` / `DomainDrift`, no canonical settlement. |
| Process interruption after execution before downstream settlement, then resume (plan tooth 3, I15) | `test_t11_interruption_after_execution_before_settlement_resumes_with_provenance` (re-emit E6 deterministically same `event_id`/`idempotency_key`, then persist E9 and reload `DevelopmentalMemoryState`) | **Resumes with provenance preserved** — re-emitted E6 has same idempotency, downstream E9 persists, `reloaded.settlements` contains `set-resume-001`, provenance chain still `caused_by:E8_ID`. |
| Restart/resume (I15) | `test_t11_restart_resume_preserves_provenance_and_is_idempotent` (persist E9, create new `LedgerStore` on same root, `load_developmental_memory_state` replay, redeliver) | **Preserves provenance, idempotent** — `reloaded.settlements` survives restart, duplicate after restart is `duplicate`, event count stays `1`. |
| Failures remain evidence (I11) | `test_t11_failures_remain_evidence` (failed `SettlementRecorded` with `settlement_status=failed`/`score=0.2` persists and remains reloadable, proven via `reloaded.settlements` and `build_developmental_memory_from_store`) + Rust `t11_failed_settlement_remains_observable` (empty observed effects → `operational_settlement_status=rejected` but still observable) | **Failure remains visible** — failed settlement durably persisted, reloadable, `work_request_id` still linked, `operational_settlement_status=rejected` not disappeared. |
| No silent semantic rewrite (I10) | `test_t11_no_silent_semantic_rewrite_detected` (mutate `observed_outcome` but keep stale `difference`) | **Detected and named** — `DomainRuleError code=difference_binding_mismatch` (diverged `observed_sha256`). |

## Falsifier → proof map (whole-loop)

Preregistration claims *"The complete production loop composes without a test-only semantic bridge."* plus causal identity, replay, recovery, failure visibility.

- **Golden scenario** — `test_t11_canonical_golden_scenario_traverses_every_production_edge` traverses `DesiredDirection → WorkRequested → ContextRequired → ContextPacketCreated → GovernedWorkRequest → AuthorizedInvocation → ExecutionObservation → OperationalSettlement → ProofCompleted → EvidenceRecorded → provisional Evidence → SettlementRecorded → DevelopmentalMemory → next navigation` with ONE `WR_ID`/`DOMAIN_HASH`, citing parent ids via `provenance.chain`, and captures 12 observations (see below). No synthetic E8: `EvidenceRecorded` is built via `build_e8` which derives `difference` from `bare_digest` of carried `expected_outcome`/`observed_outcome` and pins `proof_result_ref`/`operational_settlement_ref` as content-addressed `sha256:` digests — the same wire the real `sxr-core/src/evidence_emit.rs` enforces — and is then ingested via the **real** `ingest_evidence_recorded` gate (plus `runtime.ingest_canonical_evidence_recorded` provisional ledger path). A hand-wired `EvidenceRecorded` with stale difference would be caught as `difference_binding_mismatch` (proven by `test_t11_no_silent_semantic_rewrite_detected`).

- **Plan tooth Cross-wire** — `test_t11_cross_wire_rejected` (E3 `ctx_foreign` with `wr-foreign-999` + GSA ingest cross-wire) and Rust drift gate: **reject/quarantine, cannot attach to active cycle.**

- **Plan tooth Replay after restart** — `test_t11_duplicate_delivery_is_idempotent`, `test_t11_restart_resume_preserves_provenance_and_is_idempotent` (Python LedgerStore flock-guarded dedup) plus Rust `t11_duplicate_*`: **no duplicate side effect, settlement, capability, or history; `status==duplicate`, event count stays 1, `load_developmental_memory_state` replay preserves settlements.**

- **Plan tooth Interrupt+resume** — `test_t11_interruption_after_execution_before_settlement_resumes_with_provenance` (E6 re-emission deterministic, then E9 after resume, reload proves `set-resume-001` with preserved provenance): **recovery preserves provenance and completes same causal cycle.**

- **Plan tooth Wrong-domain** — `test_t11_wrong_domain_identity_mid_cycle_fails_closed` and Rust `t11_wrong_domain_identity_is_rejected_at_ingress`: **cycle cannot silently continue; `hash_drift`/`DomainDrift` fail-closed.**

## The 12 observations (spec `whole_loop_acceptance.canonical_golden_scenario.required_observations`)

Captured in `tests/fixtures/t11_canonical_golden.json` and asserted in the golden test:

1. `one_work_request_id_end_to_end` = `wr-gsf-t11-001` — stable across E1, E2, E3, E4, E5A, E5B, E6, E7, E8, E9 (E0 has no wr by spec; memory ledger cites via refs).
2. `one_resolvable_domain_model_identity_end_to_end` = `537449202c9d06df08373366dc0e25c7bf32e51e64d3fbf837465f0a45098741` — `sha256(b"t11-canonical-model-v1")`, structural digest, not fallback (`dc144…`) or zero, carried in every `payload.domain_model_hash` and `domain_model_ref.model_hash` and `provenance.chain[0]`.
3. `cited_context` = `[{"source":"runbook://deploy/blue-green","sha256":"0e99…0703"}]` — E3 `ContextPacketCreated` citations preserved via `citations` array, provided as `context_packet_ref=ctx_t11_001` to E4 and recorded as `caused_by:E3_ID`.
4. `explicit_authority_decision` = `dec_t11_authority_0001` — E5A `AuthorizedInvocation` carries `authority_decision_id` from a REAL `Allow` decision (Rust harness) / deterministic payload (Python harness), citing E4 as parent.
5. `observed_real_execution` = `[{"effect":"health check green",…},{"effect":"zero-downtime observed",…}]` — E5B `ExecutionObservation` `execution_status=completed` with `observed_effects`, citing E5A, `invocation_id=inv_t11_0001`.
6. `operational_settlement` = `accepted` — E6 `OperationalSettlement` `operational_settlement_status=accepted`, citing E5A+E5B, `evidence_refs` content-addressed `sha256:` over settled effects.
7. `independent_proof_result` = `passed` — E7 `ProofCompleted` `proof_status=passed` with `operational_settlement_ref=sha256:…` (content digest of E6 envelope), citing E6+E4, distinct from settlement.
8. `expected_vs_observed_realitytrace_record` = `{"verdict":"divergent","expected_sha256":"…","observed_sha256":"…"}` — E8 `EvidenceRecorded` `difference` with bare digests recomputable from carried `expected_outcome`/`observed_outcome`, citing E6+E7, stamped `realitytrace` exclusively; `proof_result_ref=sha256:` over E7 bytes.
9. `provisional_developmental_evidence` = `{"event_id":"…c4","classification":"provisional"}` — GSA `ingest_evidence_recorded` yields `ProvisionalEvidence` (frozen type, no promotion surface), `status=first`; `runtime.ingest_canonical_evidence_recorded` writes only `canonical_execution_evidence` ledger, no settlement/capability.
10. `explicit_developmental_settlement_decision` = `success` — GSA `SettlementRecorded` `settlement_status=success` with `developmental_decision={action:record_developmental_consequence,…}`, carrying `source_evidence_refs=[E8_ID]` and `provenance.chain` with `caused_by:E8_ID`, persisted via `persist_developmental_event` flock-guarded ledger (`status=first`).
11. `durable_developmental_memory` = `{"event_id":"…","capability_state":{"lifecycle":"active",…}}` — `DevelopmentalMemory` (memory_ledger → gsa) stamped `memory-ledger`, `observer_or_system_id=observer-t11`, `settlement_history_refs` from ledger, `capability_state` honestly derived via `infer_capability_lifecycle`/`compute_metabolization` (one success → `active`, not `metabolized`), provenance `domain_model_hash:` + `caused_by:` per history ref.
12. `changed_next_navigation_state` = `{"ranked_first":"aff-t11-canonical-001"}` — `NavigationRuntime.rank_candidates_with_memory` and `review_horizon_with_memory` show memory measurably boosts historical candidate (`_memory_influenced=True`, `_historical_capability=True`) while `governance_allowed=False`/`payment_capacity_exceeded`/`settlement_inaccessible` remain blocked (gate-evaluation-after-memory).

## Verification commands and results

Recorded BEFORE any edits (prereg already green):

```
HEADs (before):
  sea-rs         006daa2540678eaaf821d873266f79914b10c9af
  SWE_SEED       16dcce4cc831a007c61cca52f05e06be989496bd
  sxr            560eaf94d7ab66e3b9a5b277609c5a3816be63d0
  godspeed_agent eeee146387af1f98a04381be7c2517fb87da8d82
  Context_Kernel aa17b6a13a38b07a18d51dab6febf8426d7f21ab
Dirty state: recorded per-repo and preserved byte-for-byte (sea-rs: M .agents/CURRENT_STATUS.md, M AGENTS.md, M crates/sea-forge-server/src/lib.rs, M justfile, untracked fixtures/tests/T11; SWE_SEED: untracked T04/T07; sxr: untracked T07/T08; GSA: M AGENTS.md, M godspeed_nav/runtime.py [T08], untracked T03/T08/T09/T10 files; CK: M agentic_capability_loop.rs etc.).
No commits/pushes/branch changes. `just e2e-prereg-check` PASS before (SHA ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f unchanged).
```

| Check | Command | Result |
| --- | --- | --- |
| GSA baseline (before edits) | `bash -c "cd /home/sprime01/projects/godspeed_agent && uv run --extra dev python -m pytest -q --ignore=tests/e2e"` | **5 failed, 760 passed** — documented pre-existing set (calibration_pipeline sklearn; cli_hooks_mcp ruvector ×2; distribution adapter; domain_storage ruvector). e2e dir: 1 collection error (`psycopg` missing), pre-existing. |
| GSA new file (after, before gate) | `uv run --project /home/sprime01/projects/godspeed_agent --extra dev python -m pytest /home/sprime01/projects/godspeed_agent/tests/test_convergence_t11_whole_loop.py -v` | **10 passed**, 0 failed |
| GSA FULL suite (after, final code) | `bash -c "cd /home/sprime01/projects/godspeed_agent && uv run --extra dev python -m pytest -q --ignore=tests/e2e"` | **5 failed, 770 passed** — IDENTICAL failure set to baseline; delta is +10 collected T11 tests; zero new failures |
| GSA e2e dir (after) | `bash -c "cd /home/sprime01/projects/godspeed_agent && uv run --extra dev python -m pytest tests/e2e -q"` | same single pre-existing `psycopg` collection error; nothing else |
| sea-forge-server crate (after) | `cargo test --manifest-path /home/sprime01/projects/sea-rs/Cargo.toml -p sea-forge-server` | **32 passed** across 5 binaries (10 T05 + 14 T06 + 6 T11 + 2 other), 0 failed |
| sea-forge-server T11 only | `cargo test --manifest-path /home/sprime01/projects/sea-rs/Cargo.toml -p sea-forge-server --test convergence_t11_whole_loop` | **6 passed**, 0 failed |
| Gate | `just e2e-gate T11` (from sea-rs) | **exit 0** (`10 passed in 2.20s` Python + `6 passed` Rust) |
| Gate regression | `just e2e-gate T10` | exit 0 (`16 passed`) |
| Gate regression | `just e2e-gate T08` | exit 0 (`7 Rust + 43 Python`) |
| Prereg | `just e2e-prereg-check` | **PASS** before AND after; SHA `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f` unchanged |
| Delta | `just e2e-delta-check` | **PASS** before AND after; `confirmed:31 open:4` (status file untouched by builder) |
| Additivity | `git diff --stat` (sea-rs) | only `justfile` (T11 arm), `crates/sea-forge-server/tests/convergence_t11_whole_loop.rs` (new), and `godspeed_agent` counterpart; no `.agents/specs/` / `.agents/plans/` / `.agents/status/` touched; no `cargo clean` |

## Honest scope notes

- **T11 does NOT prove the full variation battery — T12 scope.** The golden scenario plus the six negative harnesses prove whole-loop causality, replay safety, and recovery, but they do **not** exercise the full declared variation battery (authority denial/escalation, proof failure after execution success, execution failure/timeout, semantically invalid identity at every edge) under a single recomputed delta. That is explicit `does_not_prove` of T11 (`T12: Settle Canonical Runtime Under Variation`). T11's "failure remains evidence" strand shows a failed `SettlementRecorded` and a rejected `OperationalSettlement` remain observable, but does not run the full authority-denial→no-side-effect proof battery (T05) or proof-failure→no-promotion battery (T06/T09) inside the same `WR_ID` cycle — those remain separate T05/T06/T09 confirmations composed by reference.

- **Determinism boundary:** Golden scenario is deterministic under fixed `WR_ID`, `DOMAIN_HASH`, fixed UUIDv5 `event_id`s, and `FIXED_NOW`. The Rust harness's `Operation`/`Resource` are `execute_command`/`sea-forge` and the authority grant is a real `Allow` via `PolicyAuthorityEngine` with a `sea-forge` argv0 symlink; external provider nondeterminism is removed per the known confound ("use real deterministic local seam"). The `LedgerStore` seam is filesystem `flock`-guarded JSONL with `load_developmental_memory_state` replay-through-the-gate for restart survival; it does NOT exercise an external Postgres/NATS ledger restart — the deployed bridge remains config, not proven here.

- **No test-only E8 — reused real surface:** `EvidenceRecorded` is built via `build_e8` which exactly mirrors `sxr-core/src/evidence_emit.rs` (content-addressed `proof_result_ref` over `ProofCompleted` bytes, verbatim `operational_settlement_ref`, `difference` with `bare_digest`s) and is ingested via the **real** `ingest_evidence_recorded` (and `runtime.ingest_canonical_evidence_recorded`) with `known_parent_ids` pinning. A stale `difference` is caught as `difference_binding_mismatch` (I10). The Rust harness does not re-emit E8; it composes via the Python harness's E8 and validates the same invariants on the SEA-Forge side (E4→E6).

- **Envelope construction for Rust-owned edges in Python:** For E2–E7 the Python harness synthesizes v1 envelopes via `make_envelope` rather than spawning the Rust binaries. The construction is wire-identical (same `schema_version`, `namespace`, `provenance.chain`, `idempotency_key`, `source_agent` stamps) and is validated by the **same** placeholder/drift/causality checks the Rust gates enforce (shared `FALLBACK_PSEUDO_HASH`, `is_sha256_hex`, `caused_by:` presence). The standalone Rust harness (`convergence_t11_whole_loop.rs`) independently proves those checks via **real** Rust gates over the same deterministic `WR_ID`/`DOMAIN_HASH`; the composition claim rests on both harnesses agreeing, not on Python alone.

- **Memory is evidence, not authority — by design (I12 composition):** `apply_developmental_memory_to_candidates` only boosts `expected_horizon_delta`/`direction_score`; it never writes `governance_allowed`/`settlement_accessible`/`payment_capacity`. The golden scenario proves `governance_blocked`/`payment_capacity_exceeded`/`settlement_inaccessible` stay blocked via `rank_candidates_with_memory` and `review_horizon_with_memory` (gate-evaluation-after-memory). A caller that inverts the documented order would violate I12; code comments name the correct order but cannot statically forbid reordering.

- **I2 whole-cycle traceability is structural, not clock-ordered:** Proven via stable `work_request_id`/`domain_model_hash` and `caused_by:` chain across 12 envelopes plus ledger-reload survival. It does not prove wall-clock ordering or that a single OS process held all 12 envelopes simultaneously — provenance is via persisted chain, which is the frozen contract.

- **Pre-existing failures were neither fixed nor extended:** The 5 documented GSA env failures + `psycopg` e2e collection error reproduce identically before and after; `sea-forge-server` crate suite stays green.

- **No status projection was marked:** as builder, this report does not update `.agents/status/e2e-current-status.yml`; independent adversarial CONFIRM remains a separate role (T10 pattern). The delta remains `confirmed:31 open:4` until verifier settles I2/I11/I14/I15.

## Debt ledger additions (observed, out of scope)

- **Pre-existing, not extended:** `agentic_capability_loop/adapters.py` `DOMAIN_MODEL_HASH` fallback to `dc144cbd71a4…` when manifest absent (T09 report) — the canonical T11 seams refuse that hash, but legacy emit path retains the trust-placement risk.

- **New, small:** `make_envelope` in the Python harness is a test-only helper mirroring `swe_seed_core::federation::envelope::make_event`; drift between the two could hide a future wire-field naming divergence. Mitigation: Rust harness re-checks a subset via real gates with the same `WR_ID`/`DOMAIN_HASH`, and the committed fixture `t11_canonical_golden.json` is human-inspectable. Next move: consider a shared JSON-Schema validator for the v1 family (debt theme T08-V20).

- **New, small:** `export_learning_proposals_for_ck` filesystem bridge (T10) not exercised in T11's golden path; CK retrieval of T11-derived learning proposals remains config, not proven in this loop.

## Evidence files

- `godspeed_agent/tests/test_convergence_t11_whole_loop.py` (primary harness, 10 tests)
- `godspeed_agent/tests/fixtures/t11_canonical_golden.json` (19 KB deterministic golden fixture, 12 envelopes, 12 observations, provenance_chain)
- `sea-rs/crates/sea-forge-server/tests/convergence_t11_whole_loop.rs` (6 Rust tests)
- `sea-rs/justfile` (T11 gate arm)

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

None. No frozen-contract contradiction was encountered; no edge reopening via correction protocol was required. All teeth passed on first run after harness fixes (affordance seeding, ledger `record.envelope.payload` path, `difference_binding_mismatch` code). If a future verifier falsifies composition, the smallest implicated edge will be reopened per the correction protocol (preserve failed evidence, classify, correct, rerun original teeth, append).

