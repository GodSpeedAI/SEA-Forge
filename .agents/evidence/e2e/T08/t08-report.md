# T08 Report — Settle Expected-vs-Observed Developmental Evidence (E8/I8/I10)

Builder run. Proof level P3 target; confirmation requires independent
adversarial attack (NOT performed here — nothing below is "settled"; operational
status projections were not touched).

## What was built

### 1. Emitter — `/home/sprime01/projects/sxr` (RealityTrace)

| File | Change |
| --- | --- |
| `sxr-core/src/evidence_emit.rs` | NEW. Canonical v1 `EvidenceRecorded` emitter (`emit_evidence_recorded`). Exclusive producer stamp `realitytrace`. Projects ONLY from T07's `ComparisonInputs`: `expected_outcome` verbatim from the ingested ProofCompleted, `observed_outcome` verbatim from the RESOLVED settlement's observed effects, `difference` derived (verdict + bare canonical-JSON sha256 of both sides), `proof_result_ref` = content digest of the exact ingested proof envelope bytes, `operational_settlement_ref` verbatim (already content-addressed), `domain_model_ref` = locally resolved model identity. `caused_by:` chain cites the E7-bound settlement event id + ProofCompleted event id. Fail-closed refusals (`EmitRefusal`): missing expected/observed binding (no emission without inspectable comparison), placeholder/malformed identity (incl. `dc144cbd71a4…`, all-zero), blank/placeholder correlation ids, malformed event ids / parents / timestamps, malformed refs, cited-vs-resolved settlement mismatch. Pure crate: event id + timestamp are explicit parameters; deterministic under fixed inputs. Unit tests included. |
| `sxr-core/src/lib.rs` | One added line: `pub mod evidence_emit;` (file was already dirty pre-task; change is additive). |
| `sxr-core/tests/convergence_t08_evidence_emission.rs` | NEW. 7 tests. Composes the REAL chain: T07 `ingest()` → `ComparisonInputs` → REAL emitter. Regenerates the cross-repo golden fixture `tests/fixtures/t08_evidence_recorded.json`. Teeth (emission half): no emission without resolved observation; missing/null expected refused; placeholder identities and forged/malformed refs refuse before any envelope exists; inconsistent resolution inputs refuse. Determinism test. |
| `sxr-core/tests/fixtures/t08_evidence_recorded.json` | NEW golden fixture, written by the real emitter at test time. sha256 `24ba9a1fd7353853eb74f62d570c317633cf27c2bb6e741201a5108a2f1995e7`. Carries envelope + context (`domain_model_sha256`, `work_request_id`, `affordance_id`, `proof_event_id`, `expected_settlement_event_id`, `known_parent_ids` for out-of-band parent pinning). |

### 2. Consumer — `/home/sprime01/projects/godspeed_agent` (GodSpeed-Agent)

| File | Change |
| --- | --- |
| `godspeed_nav/evidence_ingest.py` | NEW. Canonical EvidenceRecorded ingestion gate (`ingest_evidence_recorded`) mirroring the frozen contract: exclusive-producer check (canonical agent mapping; forge attempts fail closed; provenance origin must agree with the stamp), identity placeholder rejection using the SAME Rust constant set (`FALLBACK_PSEUDO_HASH`, all-zero) via the existing `verify_domain_model_hash`, drift refusal, payload+ref namespace tampering refusal, complete nine-required-field battery reported together, typed field validation (content-addressed sha256 refs; difference shape), cryptographic comparison binding (recorded `difference` digests MUST recompute from the carried expected/observed values — mutated values cannot ride a stale difference), causality presence (+ optional out-of-band `known_parent_ids` pinning → `forged_parent_reference`), cross-wired `work_request_id` refusal, duplicate/replay idempotency (event id, content digest, idempotency key), T07-style claim stability keyed `(work_request_id, proof_result_ref)` with named divergence and first-record-stands. Produces `ProvisionalEvidence` — a frozen dataclass of plain data with NO promotion/settlement/capability method of any kind. Explicitly threaded `EvidenceIngestState` (JSON-round-trippable for restart survival). Wrong event types (SettlementRecorded/CapabilityUpdated/ProofCompleted) are refused outright — this gate never ingests GSA's own outbound events. |
| `godspeed_nav/runtime.py` | ONE additive method on `NavigationRuntime`: `ingest_canonical_evidence_recorded(...)` composing the pure gate with the runtime's provisional evidence path. Writes ONLY a new `canonical_execution_evidence` ledger via `append_event_if_absent(dedup_field="evidence_event_id")`; mirrors the legacy boundary's executable-affordance fail-closed check; returns inert record dicts. No code path to `record_settlement`, SettlementRecorded, CapabilityUpdated, trajectories, repetition schedules, or capability state. No pre-existing lines modified. |
| `tests/test_convergence_t08_evidence_recorded.py` | NEW. 43 pytest tests consuming the REAL sxr fixture through the REAL gate. Named teeth tests plus negatives (details below). |

### 3. Gate — `/home/sprime01/projects/sea-rs`

- `justfile`: added `T08)` arm to the `e2e-gate` case (mirrors T03's uv invocation style): runs sxr-core `convergence_t08_evidence_emission` (regenerates the fixture) then GSA `tests/test_convergence_t08_evidence_recorded.py` via `uv run --project … --extra dev python -m pytest … -q`. justfile carried pre-existing modifications; only the arm was appended.

### 4. SWE_SEED

Untouched (read-only recon of `federation/proof_completed.rs` conventions).

## Falsifier → proof map

| Frozen falsifier / tooth | Proof |
| --- | --- |
| Tooth 1: EvidenceRecorded missing expected OR observed consequence accepted | sxr: `t08_tooth1_no_emission_exists_when_the_observed_consequence_never_resolved`, `t08_tooth1_missing_or_null_expected_binding_refuses_emission` (emission half). GSA: `test_tooth1_missing_expected_outcome_is_rejected`, `test_tooth1_missing_observed_outcome_is_rejected`, `test_tooth1_blank_observed_value_cannot_bind`, parametrized `test_tooth1_every_frozen_required_field_is_mandatory` over all eight payload fields (gate half). |
| Tooth 2: forged parent/proof/settlement refs accepted | GSA: `test_tooth2_parent_citation_outside_pinned_upstream_ids_is_a_forged_reference`, `test_tooth2_malformed_parent_id_in_chain_is_refused`, `test_tooth2_zero_causality_is_refused`, `test_tooth2_provenance_origin_contradicting_the_producer_stamp_forges`, `test_tooth2_forged_proof_and_settlement_refs_are_rejected` (malformed/non-content-addressed refs). sxr: emitter refuses malformed refs and cites only ids its own T07 gate bound (`t08_identity_placeholders_and_malformed_refs_…`, positive-path causality assertions). See honest scope notes for the resolution limit at GSA. |
| Tooth 3: receipt alone creates SettlementRecorded/CapabilityUpdated | GSA: `test_tooth3_receipt_creates_no_settlement_or_capability_anywhere` — real NavigationRuntime + real store: derived ledgers (`settlements`, `trajectories`, `capabilities`, `repetition_schedules`) remain zero; legacy `execution_evidence` untouched; exactly one provisional record in `canonical_execution_evidence`; redelivery stores nothing. Machine-checkable API surface: `test_provisional_evidence_exposes_no_promotion_surface` (enumerates every public attribute; asserts no promotion-shaped callable exists; attempts `record_settlement`/`promote_capability`/`emit_settlement_recorded`/… → AttributeError; value graph contains no callables/runtime references). |
| Wrong producers both directions | GSA `test_wrong_producers_are_refused_in_both_directions`: six impostor stamps refused; SettlementRecorded/CapabilityUpdated/ProofCompleted into the gate → `wrong_event_type`. sxr emitter stamps `realitytrace` exclusively (positive-path assertions). |
| Placeholders / drift / namespace-schema tampering | GSA `test_placeholder_identity_hashes_are_refused_exactly_as_in_rust` (Rust constant set incl. `dc144cbd71a4…` + all-zero), `test_drift_against_locally_resolved_model_is_refused`, `test_domain_model_ref_rewriting_declared_identity_is_refused`, `test_placeholder_correlation_strings_are_refused`, `test_namespace_and_schema_tampering_is_refused`. sxr mirror tests in module unit tests + `t08_identity_placeholders_…`. |
| Cross-wired work_request_id | GSA `test_cross_wired_work_request_is_refused`. sxr: correlation bound one layer upstream by the REAL T07 gate inside the composed test. |
| Duplicate / replay | GSA `test_duplicate_delivery_and_identical_rewrap_are_consequence_free`, `test_duplicate_recognition_survives_a_restart_of_the_gate_state` (state JSON round-trip), runtime redelivery assertions inside tooth 3. sxr: determinism + replay covered by T07's own suite feeding the same facts. |
| I10 mutation probe (same identity, mutated expected/observed/difference) | GSA `test_i10_mutated_observation_with_stale_difference_is_detected_and_named` (cryptographic binding catches stale-digest rewrite, names `observed_sha256`), `test_i10_consistently_restacked_divergent_resubmission_hits_claim_stability` (attacker restamps digests consistently → claim stability still refuses BY NAME, first record intact byte-for-byte). Upstream facts are additionally pinned by reference+digest on the sxr side (T07 ClaimMismatch machinery untouched). |

## Verification commands and results

Recorded BEFORE any edits:

```
HEADs:
  sea-rs         006daa2540678eaaf821d873266f79914b10c9af
  sxr            560eaf94d7ab66e3b9a5b277609c5a3816be63d0
  godspeed_agent eeee146387af1f98a04381be7c2517fb87da8d82
  SWE_SEED       16dcce4cc831a007c61cca52f05e06be989496bd
Dirty state: recorded per-repo (see session log); preserved byte-for-byte.
Preregistration SHA (before AND after): ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f — PASS both.
```

| Check | Command | Result |
| --- | --- | --- |
| sxr new tests | `cargo test -p sxr-core --test convergence_t08_evidence_emission` | 7 passed; 0 failed |
| sxr FULL crate suite | `cargo test -p sxr-core` | 55 passed across 6 binaries; 0 failed (19 lib + 3 ac_core_001 + 12 t07 + 7 t08 + 7 envelope_build + 7 snapshot_threshold) |
| rustfmt (touched crate) | `cargo fmt -p sxr-core -- --check` | clean |
| clippy (touched crate, all targets) | `cargo clippy -p sxr-core --all-targets` | clean (one redundant-closure warning fixed during the run) |
| GSA new tests | `uv run --extra dev python -m pytest tests/test_convergence_t08_evidence_recorded.py -q` | 43 passed |
| GSA FULL suite (excl. `tests/e2e`) | `uv run --extra dev python -m pytest -q --ignore=tests/e2e` | **5 failed, 650 passed** — the EXACT documented pre-existing set (ruvector/sklearn/distribution env issues: calibration_pipeline sklearn, cli_hooks_mcp ruvector ×2, distribution collection adapter, domain_storage ruvector). NO new failures. |
| GSA e2e dir | `pytest tests/e2e -q` | 1 collection error (`psycopg` missing) — the documented pre-existing env issue; nothing else. |
| Gate | `just e2e-gate T08` (from sea-rs) | exit 0 (7 Rust + 43 Python green) |
| Prereg | `just e2e-prereg-check` | PASS before and after; SHA `ef571089…879f` unchanged |

## Honest scope notes

1. **T08 does NOT prove GSA SHOULD promote anything.** Ingestion yields strictly
   provisional records; whether any evidence qualifies for developmental
   settlement, repetition planning, or capability update is GodSpeed-Agent's
   independent evaluation — T09 scope. The known confound stands: a later
   explicit GSA settlement action may validly promote qualifying evidence.
2. **Digest-pinning coverage, disclosed precisely.** Pinned where the substrate
   permits: `proof_result_ref` = content digest of the exact ingested
   ProofCompleted envelope bytes; `operational_settlement_ref` verbatim
   content-addressed (resolvable against delivered settlement bytes at the sxr
   seam, enforced by T07's own gate). NOT resolvable at GSA: the E8 transport
   does not deliver the upstream envelopes themselves, so the Python gate can
   verify refs structurally (well-formed content-addressed digests) and bind
   the three comparison values cryptographically to each other, but CANNOT
   recompute `proof_result_ref`/`operational_settlement_ref` against upstream
   bytes. Parent-id pinning in the golden flow is OUT-OF-BAND transport context
   (`known_parent_ids` in the fixture), exercised as a falsifier surface; in
   production GSA has no independent knowledge of those ids under the frozen
   topology — that residual trust placement is disclosed rather than papered
   over.
3. **Routing decision:** the pre-existing `adapters.handle_evidence_recorded`
   serves the Family-A loose-event shape (`proof_result_id`,
   `expected_result`, `observed_result`) and FAILS CLOSED on canonical E8
   envelopes (missing-field error); it was left untouched (additive-only
   discipline; changing it would alter behavior for existing callers/tests).
   The canonical E8 entry points are `godspeed_nav.evidence_ingest` +
   `NavigationRuntime.ingest_canonical_evidence_recorded`.
4. **Optional E8 fields** (`authority_decision_ref`, `artifact_refs`,
   `trace_refs`, `evidence_refs`, `reliability_inputs`) are not emitted by the
   sxr surface because `ComparisonInputs` carries none of them — nothing is
   invented. `work_request_id` likewise is not part of `ComparisonInputs`; the
   emitter takes it as an explicitly gated parameter, and the composed test
   drives BOTH surfaces from one real ingested cycle so the value provably
   originates in the T07-bound fact set (T07 itself cross-checks it against
   the settlement).
5. **Difference model:** verdict is exactly canonical-JSON equivalence of the
   two projected values plus their bare sha256 digests. No semantic field-level
   diffing is invented; given the pinned refs, any party can reconstruct and
   inspect the full comparison.
6. Nothing was committed/pushed anywhere; no spec/plan/status file was touched;
   `target/` was not cleaned; all pre-existing dirty state remains.

## Debt ledger additions (observed, out of scope)

- **Pre-existing, not extended:** `agentic_capability_loop/adapters.py`
  `DOMAIN_MODEL_HASH` falls back to the namespace pseudo-hash
  (`dc144cbd71a4…` — the exact constant every convergence gate forbids) when
  the SEA manifest is absent. The T08 gate refuses that hash as identity, so
  composition requires an explicitly resolved digest; the fallback remains a
  standing trust-placement risk in the legacy emit path (same family as
  T05-A9/T06 minors).
- **New, small:** `NavigationRuntime.ingest_canonical_evidence_recorded` keeps
  its claim-stability state in memory (`EvidenceIngestState`); durable
  duplicate detection relies on the ledger dedup key. Restart-survival of the
  claim ledger would require persisting gate state — deferred until a
  deployment actually needs it (the pure state object already round-trips
  through JSON, proven by test).
