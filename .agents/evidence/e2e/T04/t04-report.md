# T04 — Governed Work Submission (frozen edge E4) — builder evidence

Plan: `.agents/plans/e2e-plan.yml` task T04 (P3, independent_adversarial;
settles E4). Frozen target: `.agents/specs/e2e-preregistration.yml` SHA
ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f — verified
intact by `just e2e-prereg-check` BEFORE and AFTER this work.

## What was built

Smallest delta honoring the frozen E4 contract across both repositories.
No preregistration/plan/status file was modified; no commits were made
(sea-rs HEAD `006daa2540678eaaf821d873266f79914b10c9af`, SWE_SEED HEAD
`16dcce4cc831a007c61cca52f05e06be989496bd`, unchanged end-to-end).

### Producer side — SWE_SEED (`crates/swe-seed-core`)

- `src/federation/governed_submission.rs` (new): canonical GovernedWorkRequest
  emission surface built on the T01 substrate. `build_governed_work_request()`
  takes the accepted T03 `WorkRequestContract`, the adjudicated T03-era E3
  `ContextPacketCreated` envelope, typed semantic inputs (`GovernedSubmission`,
  `ProofContract`), and a `VerifiedDomainIdentity`. It:
  - re-validates BOTH causal parents through the composed T01 boundary gate
    (`validate_envelope`: exclusive producers godspeed_agent/context_kernel,
    non-placeholder identity, drift against OUR verified hash);
  - refuses cross-wired packets and contract/envelope correlation mismatches
    BEFORE any envelope exists (`SubmissionError::CrossWiredContextPacket`,
    `ContractEnvelopeMismatch`);
  - refuses opaque command-only submissions at the type level (missing
    intent / actor / proof criterion are unrepresentable-but-checked
    `IncompleteIntent` errors);
  - carries ALL eight frozen required payload fields (work_request_id,
    affordance_id, actor{actor_id,role?}, intent, context_packet_ref,
    domain_model_ref, proof_contract, settlement_criteria) plus the optional
    route_id / artifact_expectations / authority_context / payment_budget
    pass-throughs; `context_packet_ref` names the packet's real
    `context_packet_id` (CK egress contract, event-id fallback);
  - derives through `derive_event` with parents `[WorkRequested,
    ContextPacketCreated]` so causality (ENV-I4) and correlation stability
    (ENV-I3) are substrate-enforced; stamp comes from `make_event`
    (`swe-seed`, the exclusive authoritative producer of GovernedWorkRequest
    per the T01 registry, `producers.rs:54`).
- `src/federation/mod.rs`: module declaration + re-exports only (additive on
  top of the pre-existing uncommitted WIP, preserved byte-for-byte).
- `tests/convergence_t04_governed_submission.rs` (new, 10 tests) incl.
  `t04_writes_golden_fixture_for_sea_forge_ingress`, which regenerates the
  cross-repo golden fixture from REAL emitter output into
  `sea-rs/crates/sea-forge-server/tests/fixtures/t04_governed_work_request.json`
  (env-overridable `SEA_RS_ROOT`, honest skip if absent — T03 pattern). The
  fixture records the producing model digest out-of-band
  (`domain_model_sha256`) so the consumer drift-checks an independently
  resolved identity rather than the request's own claim.

### Consumer side — sea-rs (`crates/sea-forge-server`)

Least-invasive existing ingress surface: a sibling of
`swe_seed_reconciliation.rs` (the existing SWE_SEED-facing module named by the
plan's hot_context), pure and transport-free. NO server NDJSON protocol change
(adding a `Request` verb is a public-interface change requiring owner
approval) and no invented transport.

- `src/governed_work_ingress.rs` (new): `accept_governed_work_request(request,
  context_packet, local_model_sha256)` acceptance gate:
  1. canonical v1 shape + exclusive producer authority — only `swe_seed` may
     emit GovernedWorkRequest; the referenced packet must genuinely be
     `context_kernel`'s ContextPacketCreated (I3, fail-closed vocabulary);
  2. canonical domain identity (ENV-I1/I2, I1): declared hash must be a real
     SHA-256 digest, never the standalone fallback constant / all-zero /
     malformed pseudo-identity, and MUST equal SEA-Forge's locally resolved
     canonical model — the model authority will evaluate against IS the
     declared model (drift rejected);
  3. semantic work intent battery: all required fields present and non-empty
     (actor object with actor_id, ≥1 settlement criterion, proof_contract
     object carrying a criterion); failures report the complete missing list
     as `OpaqueCommand`;
  4. distinct obligations: whatever sits in the proof_contract slot may never
     equal the settlement_criteria (`CollapsedObligations`) — I6 begins here;
  5. context binding: packet correlates to the SAME work_request_id,
     `context_packet_ref` names exactly that packet, packet identity passes
     the same gates, and the packet's event id is recorded among the request's
     `caused_by:` causal parents (ENV-I4) or `CausalityMissing`.
  Returns the parsed `GovernedWorkIntent`; any failure means no governed
  request exists.
- `src/lib.rs`: one added `pub mod governed_work_ingress;` line.
- `tests/convergence_t04_governed_ingress.rs` (new, 12 tests): consumes the
  golden fixture through the real gate plus adversarial mutations.
- `tests/fixtures/t04_governed_work_request.json` (new): real SWE_SEED
  emitter output (regenerated by the producer suite each gate run).

### Gate binding — `justfile`

`T04)` arm added to the `e2e-gate` recipe case following the T02/T03 arm
pattern: runs the SWE_SEED emitter teeth (regenerating the fixture) then the
SEA-Forge consumer gate. Additive hunk inside the already-uncommitted e2e
section; no other recipe touched.

## Falsifier → proof map

| Plan tooth / frozen falsifier | Proof |
| --- | --- |
| Opaque command-only request cannot settle as canonical GovernedWorkRequest | SEED `t04_opaque_command_only_submission_cannot_be_built`; FORGE `t04_opaque_command_only_request_cannot_settle` (argv/command-only payload → `OpaqueCommand` listing every missing semantic field), `t04_empty_semantic_fields_are_opaque_commands_too` |
| ContextPacketCreated from another work_request_id is rejected | FORGE `t04_cross_wired_context_packet_is_rejected`, `t04_swapped_in_foreign_packet_is_rejected`; SEED `t04_cross_wired_context_packet_is_rejected_before_emission` |
| Valid but DIFFERENT DomainForge model than declared → reject/quarantine | FORGE `t04_different_domain_model_is_rejected` (local resolution ≠ declared → `DomainDrift`), `t04_drift_on_the_referenced_context_packet_is_rejected_too`; SEED `t04_different_domain_model_is_rejected_before_emission` |
| Placeholder/fallback pseudo-identities (ENV-I2/I1) | FORGE `t04_placeholder_pseudo_identity_is_rejected` (fallback constant, all-zero, malformed); SEED `t04_fallback_pseudo_identity_cannot_reach_the_boundary` (type-gated: `VerifiedDomainIdentity` cannot be built from them) |
| Wrong producer stamps (I3) | FORGE `t04_non_swe_seed_producer_stamp_is_rejected` (request forgers incl. self-stamped packet); SEED `t04_wrong_producer_stamp_on_upstream_envelopes_is_refused` (SWE_SEED self-producing E1, forged E3) |
| proof_contract vs settlement_criteria remain distinct facts (I6 direction) | FORGE `t04_proof_contract_may_not_collapse_into_settlement_criteria` (collapse → `CollapsedObligations`, hollow contract refused); SEED `t04_proof_contract_and_settlement_criteria_remain_distinct_facts` |
| Lost causal lineage / reference mismatch (ENV-I4) | FORGE `t04_lost_causal_lineage_is_rejected`, `t04_context_ref_to_another_packet_is_rejected`; SEED happy path asserts both parents recorded |
| Positive path: real production chain E1→(T02-era E3)→E4 | SEED `e4_complete_governed_submission_carries_the_frozen_contract` (all eight fields, both parents, verified identity); FORGE `e4_golden_fixture_from_swe_seed_producer_is_accepted` (real cross-repo emitter output through the real gate) |

## Verification performed

```
SEED: cargo test -p swe-seed-core            # full crate: all suites green, 0 failed
                                            # (convergence_t04_governed_submission: 10 passed)
FORGE: cargo test -p sea-forge-server        # full crate: 25/25 test binaries ok, 0 failed
                                            # (convergence_t04_governed_ingress: 12 passed)
GATE:  just e2e-gate T04                     # exit 0
PREREG: just e2e-prereg-check                # PASS before AND after work (SHA intact)
fmt:   rustfmt --check --edition 2021 on all five touched Rust files  # clean
clippy: cargo clippy -p swe-seed-core --all-targets and -p sea-forge-server
       --all-targets                          # zero diagnostics on touched files
                                            # (pre-existing warnings elsewhere untouched)
HEADs: sea-rs 006daa2…, SWE_SEED 16dcce4…    # recorded before start, unchanged (no commits)
```

Pre-existing dirty state preserved byte-for-byte (verified via git status
diff against session start): SWE_SEED spec-0020 gateway/docs WIP and all
T01–T03 federation WIP untouched; sea-rs T00–T03 uncommitted artifacts
untouched. No `cargo clean`; worktrees untouched; nothing committed.

## Honest scope notes (what T04 does NOT prove)

- **Confirmation state: PENDING.** proof_level P3 requires fresh independent
  adversarial confirmation; this is builder evidence only. NOTHING was marked
  settled in `.agents/status/e2e-current-status.yml` (deliberately untouched,
  as instructed).
- **No live cross-process transport exists** between the repos (Delta-0
  stands). Per the T03 precedent the boundary is proven as real-emitter
  output → golden fixture → real consumer gate, plus unit teeth both sides.
  Physical sink/dispatch wiring into SWE_SEED's emit path and SEA-Forge's
  socket protocol remains open debt for the transport that eventually carries
  E4; this gate must be composed there, not reimplemented.
- **The gate is not yet wired into the server NDJSON `Request` enum**
  (public-interface change needing owner approval); it is a library-level
  ingress surface beside the existing SWE_SEED reconciliation module.
- **Downstream semantics are out of scope:** authorization/side effects
  (E5A/E5B, I5, task T05), operational settlement return (E6, T06), and the
  DomainForge `evaluate()` verdict itself are NOT proven. T04 proves only
  that SEA-Forge's ingress receives semantically complete, correctly bound,
  exclusively-produced work whose declared model equals the locally resolved
  canonical model — the precondition authority evaluation needs.
- Fixture regeneration is deterministic in semantics but volatile in
  event_id/occurred_at (same property as the accepted T03 fixture pattern).
