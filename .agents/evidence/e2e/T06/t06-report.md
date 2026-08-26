# T06 — Settle Operational Settlement Return (frozen edge E6, invariants I6/I7) — builder evidence

Plan: `.agents/plans/e2e-plan.yml` task T06 (P3, independent_adversarial;
settles E6, I6, I7). Frozen target:
`.agents/specs/e2e-preregistration.yml` SHA
ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f — verified
intact by `just e2e-prereg-check` during and AFTER this work (the file was
never opened for writing; the same SHA was bound before T05 began).

sea-rs HEAD `006daa2540678eaaf821d873266f79914b10c9af`, SWE_SEED HEAD
`16dcce4cc831a007c61cca52f05e06be989496bd`, both recorded at session start
and unchanged end-to-end (no commits). NOTHING is marked settled in any
operational status projection (`.agents/status/`, `.agents/CURRENT_STATUS.md`,
`.agents/current_status.yml`, `.agents/specs/`, `.agents/plans/` all
untouched).

## What was built

Smallest delta honoring the frozen contract, following the T04/T05 precedent:
one native sea-rs canonical-envelope boundary module plus one SWE_SEED
adjudication module over the same v1 wire family. No new envelope family; no
cross-repo dependency in either direction (sea-rs still does not depend on
SWE_SEED, and SWE_SEED still does not depend on sea-rs — parity is enforced
through a cross-repo golden fixture, T04 pattern reversed).

### Producer side — sea-rs (`crates/sea-forge-server`)

New module `src/governed_settlement_return.rs`, pure and transport-free,
composed from REAL upstream components:

- **Evaluator** `evaluate_operational_settlement(outcome, declared_criteria)`
  — takes ONLY an [`ObservationOutcome::Settled`] from the T05
  `InvocationLedger` (a `DuplicateDelivery` evaluates nothing: nothing
  settled, nothing exists) and compares the ledger-settled observed effects
  against the settlement criteria DECLARED on the governed request chain
  (E4 `settlement_criteria`). Composition with existing machinery:
  per-criterion satisfaction uses `sea_forge_settlement::evaluate_agent_output`
  over sorted-key canonical JSON of exactly the settled effects, and
  non-completed statuses reuse `sea_forge_settlement::settle()`'s durable
  basis vocabulary (`spawn_failed` / `timed_out` / `jail_violation` /
  `suspected_jail_violation`). Status reuses the kernel's own
  `SettlementStatus`. **`completed` (the exit-zero analogue) contributes
  nothing on its own**: acceptance requires EVERY declared criterion attested
  by observed effects, so exit-zero-with-violated-criteria settles `Rejected`
  (plan tooth 1). Empty/blank declared criteria are refused
  (`OpaqueCriteria`) — vacuous acceptance is unrepresentable.
- **Emitter** `emit_operational_settlement(...)` — evaluates then projects
  one atomic return (no caller can pair outcome A with evaluation B):
  - stamps the canonical v1 family byte-shape (`schema_version v1`,
    uuid-v4 event id, `source_agent`/`provenance.origin` = `sea-forge`,
    occurred_at `+00:00`, content-derived idempotency key
    `sha256("|OperationalSettlement|<sorted payload json>")` — F-10 parity);
  - carries ALL SEVEN frozen required payload fields — work_request_id,
    authority_decision_id (from the LEDGER record only), execution_status,
    observed_effects, operational_settlement_status (`accepted`/`rejected`),
    evidence_refs, domain_model_ref `{namespace, model_hash}` — plus
    invocation_id and typed optional pass-throughs (case_id, run_id,
    artifact_refs, transcript_ref, failure_reason);
  - identity gates identical to T04/T05: structural SHA-256 validity, never
    the fallback constant `dc144cbd71a483431301ca1bf32e95c014af4edba8dbcc525f505310e30a107e`,
    never all-zero or malformed, drift vs locally-resolved model refused;
  - causality MANDATORY (ENV-I4): BOTH the AuthorizedInvocation (E5A) and
    ExecutionObservation (E5B) event ids of the actual chain are required
    parameters recorded as `caused_by:` entries;
  - ENV-I7: evidence_refs are content-addressed where the substrate permits —
    `sha256:<digest>` over the canonical settled effects — plus precise
    pointers into the immutable upstream chain;
  - namespace checked on the wire battery (T04-D3 closed);
  - self-validation: the built envelope must survive the full
    [`validate_operational_settlement_wire`] battery before it is returned.
  - stable identity semantics: re-projecting the SAME settled facts yields
    the SAME idempotency key with fresh event ids (ENV-I6); ANY mutation of
    projected facts yields a DIFFERENT identity — which the receiver treats
    as a conflicting re-settlement, never a fresh fact.
- Shared wire validator `validate_operational_settlement_wire`: parse +
  validate EVERY required field from wire shape (no T04-D1 gap), vocabulary
  gates (execution_status; operational_settlement_status refuses proof-shaped
  verdicts like `proof_passed`), evidence-ref shape rules, domain_model_ref
  internal-consistency gate, typed optional refusals, ≥1 causal parent.
- Typed error family `SettlementReturnError` (+ lossless `From<BoundaryError>`
  for the shared family gates).

Wiring: `src/lib.rs` gained exactly one line (`pub mod
governed_settlement_return;`). `src/governed_execution_boundary.rs` gained
visibility widenings only (`fn` → `pub(crate) fn` on 14 family helpers + 2
consts) so T06 shares ONE source of truth for wire conventions — zero behavior
change; the full T04/T05 suites prove this.

### Consumer side — SWE_SEED (`crates/swe-seed-core`)

New module `src/federation/operational_settlement.rs` next to the federation
module, mirroring `work_ingress.rs` style:

- **`OperationalSettlementAdjudicator::adjudicate(envelope,
  originating_work_request_id, expected_chain, local_model_sha256)`** —
  binding chain, every link mandatory, all BEFORE anything is recorded:
  1. canonical v1 shape + loop namespace + exact event type;
  2. composed T01 boundary gate (`validate_envelope`): producer authority
     (`sea_forge` exclusively per the frozen registry), non-placeholder
     identity, drift against OUR local resolution, well-formed parent ids;
  3. duplicate/conflict recognition FIRST: redelivery under stable identity
     (event id folded through SHA-256, or content-derived idempotency key)
     ⇒ `DuplicateDelivery`, consequence-free; a NEW envelope claiming an
     ALREADY-adjudicated chain (same work_request_id + authority_decision_id)
     ⇒ `ConflictingResettlement` — mutated observed_effects cannot re-attest
     a settled chain into success;
  4. all seven frozen fields parsed/validated (complete missing list),
     placeholder identities refused, vocabularies enforced, evidence refs
     shape-checked (ENV-I7), `domain_model_ref.model_hash` must equal the
     declared identity (no silent rewrite, I10), typed optionals refused not
     coerced;
  5. causality binding: EVERY supplied known-chain id must appear among the
     recorded `caused_by:` parents; empty binding refused;
  6. correlation binding to the ORIGINATING work_request_id (ENV-I3);
  7. crash-consistent durable admission via the existing T01
     `IdempotencyLedger` (envelope identities + chain keys share one
     append-only JSONL), so duplicate AND conflict detection SURVIVE RESTART.
- **Operational-facts-only output**: `Adjudication::First(OperationalSettlementFacts)`
  carries exactly {settlement_event_id, work_request_id,
  authority_decision_id, invocation_id?, execution_status,
  operational_outcome, observed_effects, evidence_refs}. The two-variant
  `OperationalOutcome` (`Accepted`/`Rejected`) is its own type — not a proof
  verdict, not a developmental settlement. Nothing capability- or proof-shaped
  is representable on the returned type (I6/I7 surface statement,
  machine-checked by exhaustive destructuring in tests).
- Wiring: `federation/mod.rs` — one `pub mod` line + one re-export block,
  additive inside the pre-existing uncommitted WIP (preserved byte-for-byte).

### Gate binding — `justfile`

`T06)` arm added to the `e2e-gate` case following the T02–T05 arms: runs the
sea-rs emitter suite first (regenerating the cross-repo golden fixture into
`SWE_SEED/crates/swe-seed-core/tests/fixtures/t06_operational_settlement.json`
from REAL emitter output over a REAL authority decision), then the SWE_SEED
consumer suite adjudicating that fixture. Additive hunk inside the
already-uncommitted e2e section; no other recipe touched.

## Falsifier → proof map

| Plan tooth / frozen falsifier | Proof |
| --- | --- |
| TOOTH 1: exit code 0 while violating declared criteria ⇒ settlement records failure | FORGE `t06_tooth1_exit_zero_with_violated_criteria_records_failure` (observation carrying explicit `"exit_code": 0` + `completed` through the REAL InvocationLedger ⇒ evaluation Rejected, envelope says `rejected`, basis names every unsatisfied criterion); `t06_exit_zero_alone_cannot_produce_acceptance_even_with_status_completed`; consumer side `t06_mutated_reattestation_of_the_same_chain_is_refused` closes the flip-after-the-fact route |
| TOOTH 2: consuming OperationalSettlement directly as developmental settlement ⇒ impossible / no promotion | SEED `t06_tooth2_operational_settlement_cannot_promote_developmental_state` (producer registry refuses sea_forge stamps on SettlementRecorded/CapabilityUpdated even when copied verbatim from the received settlement; deriving children FROM the settlement parent cannot mint either event under any stamp); `t06_legacy_gsa_settlement_consumer_refuses_an_operational_settlement`; facts-only output type proven by exhaustive destructuring in `e6_valid_operational_settlement_adjudicates_to_operational_facts_only` |
| Operational settlement independent of exit status (E6 invariant) | TOOTH 1 tests + evaluator accepts ONLY on per-criterion attestation (`t06_partial_attestation_is_rejection_not_success`); evaluation input is the ledger-settled observation, never a process result (`t06_evaluation_requires_a_ledger_settled_observation`) |
| Success smuggling via mutated observed_effects vs immutable upstream facts | FORGE `t06_emitted_envelope_matches_the_canonical_wire_family` (mutated effects change stable identity); SEED duplicate-by-event-id/payload-key recognition (`t06_duplicate_delivery_is_consequence_free`, incl. mutated payload under the SAME event id) + chain-conflict refusal (`ConflictingResettlement`), both restart-safe (`t06_duplicate_and_conflict_detection_survives_restart`) |
| Wrong work_request binding | SEED `t06_wrong_work_request_binding_is_refused_and_records_nothing` (refusal records nothing durably) |
| Producer forgeries, both directions | FORGE `t06_wrong_producer_stamps_are_refused_at_wire_validation` (four known forgers + mystery stamp + cross-type delivery + schema bump); SEED `t06_producer_forgeries_are_refused_both_directions` (swe_seed / execution_environment stamps, unknown agent) |
| Replay / duplicate delivery | SEED exact replay, fresh-id-same-content re-wrap, mutated-payload same-id redelivery — all consequence-free, ledger unchanged |
| Placeholder identities | FORGE `t06_emission_refuses_placeholder_and_drifted_model_identity` (dc144cbd… constant, all-zero, malformed, uppercase, drift); SEED pseudo-hash/drift/correlation placeholders (`t06_namespace_schema_and_identity_tampering_is_refused`) |
| Causality gaps | FORGE `t06_emission_requires_the_actual_invocation_chain_as_causality` (blank/whitespace/placeholder E5A/E5B ids refused); SEED `t06_causality_binding_to_the_known_invocation_chain_is_mandatory` (empty binding refused; unknown chain id named as unbound; refusals record nothing) |
| Missing fields / schema tampering | FORGE removal loop over ALL seven required fields + typed-shape battery (`t06_every_projected_field_is_validated_on_the_wire`); SEED omission battery naming each field + vocabulary/shape batteries (`t06_missing_required_fields_are_refused_with_the_complete_list`) |
| Namespace check (T04-D3 closed) | payload-namespace gate on both sides (FORGE battery; SEED payload-level and domain_model_ref-level swaps ⇒ `WrongNamespace`) |
| Proof-contract collapse attempts (I6 direction) | proof-shaped verdicts smuggled into `operational_settlement_status` refused (`t06_proof_verdicts_cannot_smuggle_through_the_settlement_status_field`: proof_passed/proof_succeeded/capability_granted/promoted/success); criteria-vs-proof distinctness inherited unchanged from the T04 ingress (`CollapsedObligations`), re-exercised via the real T04 ingress in `t06_criteria_come_from_the_declared_governed_request_chain` |
| Positive path over real components | FORGE `e6_full_governed_chain_settles_operationally_and_emits_the_frozen_envelope` (REAL PolicyAuthorityEngine → T05 emit/admit/settle → evaluate → emit → full wire battery; content-addressed evidence digest independently recomputed); `t06_criteria_come_from_the_declared_governed_request_chain` feeds REAL T04-ingress-parsed declared criteria; SEED `e6_golden_fixture_from_sea_forge_is_adjudicated` consumes the REAL emitter's golden fixture (regenerated each gate run; honest skip only if absent — not skipped in gate runs) |

Test inventory: `convergence_t06_operational_settlement` 14 tests in
`crates/sea-forge-server/tests/` + 13 tests in
`SWE_SEED/crates/swe-seed-core/tests/`.

## Verification performed

```
FORGE: cargo test -p sea-forge-server   # FULL crate suite, run twice (pre- and
                                        # post-format): 28 test binaries ok, 0 failed
                                        # (convergence_t06_operational_settlement: 14 passed;
                                        #  convergence_t05_*: 20 passed, convergence_t04: 12 passed
                                        #  — prior tasks intact; conformance_m16 keeps its 2
                                        #  pre-existing ignored release-gates)
SEED:  cargo test -p swe-seed-core      # FULL crate suite, run twice: 58 test binaries
                                        # ok, 0 failed (convergence_t06_operational_settlement:
                                        # 13 passed; t01–t04 suites green)
GATE:  just e2e-gate T06                # exit 0 (14 + 13 tests; golden fixture regenerated
                                        # then adjudicated in-run)
PREREG: just e2e-prereg-check           # PASS (SHA ef571089…93f intact), repeated after all
                                        # changes: PASS again
fmt:   rustfmt --edition 2021 --check on all six touched Rust files  # clean
clippy: cargo clippy -p sea-forge-server --all-targets and -p swe-seed-core --all-targets
                                       # zero diagnostics on touched files
                                       # (pre-existing warnings elsewhere untouched)
HEADs: sea-rs 006daa2540678eaaf821d873266f79914b10c9af,
       SWE_SEED 16dcce4cc831a007c61cca52f05e06be989496bd  # unchanged, no commits
```

Pre-existing dirty state preserved byte-for-byte: SWE_SEED spec-0020 WIP and
all T01–T04 federation WIP untouched (mod.rs delta is strictly my additive
lines inside already-dirty hunks); sea-rs working set untouched except the
listed files; no `cargo clean` (target stayed at ~4.9 GB); worktrees
untouched; nothing committed.

## Honest scope notes (what T06 does NOT prove)

- **Confirmation state: PENDING.** proof_level P3 requires fresh independent
  adversarial confirmation; this is builder evidence only. E6/I6/I7 remain
  open until the verifier returns CONFIRM.
- **T06 does NOT prove SWE_SEED proof succeeds, RealityTrace observes the
  result, or GSA records developmental settlement** — those are E7/E8/E9
  (tasks T07+). This surface returns operational facts only; the proof plane
  stays downstream and untouched by construction here.
- **No live cross-process transport exists** between sea_forge and SWE_SEED
  (Delta-0 stands). As in T03–T05, the boundary is proven as real-emitter
  output → golden fixture → real consumer gate, plus unit teeth both sides.
  Physical dispatch wiring (handing the emitted envelope across an actual
  socket/bus) remains open debt; this gate must be COMPOSED there, not
  reimplemented.
- **Both surfaces are library modules without a non-test caller yet**
  (identical to the settled T03/T04/T05 pattern). Wiring the evaluator into
  `case_dispatch`'s episode path and the adjudicator into a SWE_SEED receive
  path are additive future work needing owner approval before any
  public-interface change.
- **Durability scope:** consumer duplicate/conflict detection is durable via
  the JSONL idempotency ledger (restart-proven by test), but the EMITTER-side
  ledger state feeding evaluation lives in the T05 in-memory
  `InvocationLedger`; a deployment that restarts between authorization and
  settlement must reload that ledger from durable records before the late-
  observation property holds again (same A11-class note as T05).
- **Declared-criteria semantics:** E4 declares criteria as free-form strings;
  this evaluator composes the existing containment-based machinery
  (`evaluate_agent_output`) over serialized observed effects. Richer typed
  criterion schemas would be a preregistration change, out of scope here.
- **`Escalated` is intentionally absent** from this edge's settlement-status
  wire vocabulary: behind the T05 Allow gate no escalation reaches settlement;
  if a future edge needs it, that is a frozen-contract revision, not a local
  default.
