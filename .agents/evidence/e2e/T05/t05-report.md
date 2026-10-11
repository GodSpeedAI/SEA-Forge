# T05 — Settle Governed Contact With Reality (frozen edges E5A/E5B, invariant I5) — builder evidence

Plan: `.agents/plans/e2e-plan.yml` task T05 (P3, independent_adversarial;
settles E5A, E5B, I5). Frozen target:
`.agents/specs/e2e-preregistration.yml` SHA
ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f — verified
intact by `just e2e-prereg-check` BEFORE and AFTER this work.

sea-rs HEAD `006daa2540678eaaf821d873266f79914b10c9af`, recorded before
start and unchanged end-to-end (no commits). SWE_SEED HEAD `16dcce4…`
untouched; no changes were needed or made there. Nothing was marked settled
in any operational status projection (` .agents/status/`,
`.agents/CURRENT_STATUS.md`, `.agents/current_status.yml` all untouched).

## What was built

Smallest delta honoring the frozen contract, following the T04 precedent: a
native sea-rs canonical-envelope boundary module over the v1 wire family, in
`crates/sea-forge-server`. No new envelope family; sea-rs took no dependency
on SWE_SEED.

### New module — `crates/sea-forge-server/src/governed_execution_boundary.rs`

Pure, transport-free functions over decoded envelope JSON plus REAL domain
types:

- **E5A emitter** `emit_authorized_invocation(decision, work_request_id,
  declared_model_sha256, local_model_sha256,
  governed_work_request_event_id, grant_id)`:
  - authority gate FIRST (I5): the decision must be a real
    `AuthorityDecision` with verdict `Allow` AND normalized disposition
    `Allow`; deny / escalate / boundary / degraded produce NO envelope at
    all (`AuthorityNotGranted`) — there is nothing an execution environment
    could receive. Both conjuncts are load-bearing (tested by mutating a
    real decision each way).
  - stamps the canonical v1 family byte-shape (`schema_version v1`,
    uuid-v4 event id, `source_agent`/`provenance.origin` = `sea-forge`,
    occurred_at `+00:00`, content-derived idempotency key
    `sha256("|AuthorizedInvocation|<sorted payload json>")` matching the
    SWE_SEED F-10 parity contract);
  - carries every frozen required payload field — work_request_id,
    invocation_id (minted `inv_<hex>`), authority_decision_id (from the
    decision object only), operation/resource (total projection of every
    `AuthorityAction` variant), execution_constraints (projected ONLY from
    authority-side facts: granted sandbox class, decision-context timeout,
    compensating controls) — plus optional `grant_id`;
  - identity mirrors T04: declared hash must be structurally valid SHA-256,
    never the standalone fallback constant `sha256("agentic_capability_loop")`
    = `dc144cbd71a483431301ca1bf32e95c014af4edba8dbcc525f505310e30a107e`,
    never all-zero, never malformed, and MUST equal the locally resolved
    model (drift refused);
  - causality is MANDATORY (ENV-I4): the GovernedWorkRequest event id this
    invocation descends from is a required parameter and is recorded as
    `caused_by:` in `provenance.chain`.
- **E5B adjudicator** `InvocationLedger`: registry of invocations actually
  emitted and admitted here, keyed by invocation slot `(work_request_id,
  operation, resource)` with generations.
  - `admit_authorized_invocation(envelope, local_model_sha256)`: full
    wire-shape parse of EVERY frozen field (work_request_id /
    invocation_id / authority_decision_id / operation / resource /
    execution_constraints — avoiding T04's D1 gap), exclusive producer
    `sea_forge` (I3), placeholder-identity refusal, drift check, ≥1 causal
    parent required; re-admitting the same envelope is idempotent (ENV-I6);
    a NEW envelope for the same slot opens the next generation and
    supersedes all earlier ones.
  - `settle_execution_observation(envelope, local_model_sha256)`: binding
    chain with every link mandatory — exclusive producer
    `execution_environment` (I3); identity structural + placeholder +
    drift; every frozen field parsed/validated (execution_status restricted
    to the governed vocabulary completed/spawn_failed/timed_out/
    sandbox_violation/suspected_sandbox_violation; observed_effects
    required array/object; typed optional refs validated not coerced);
    duplicate delivery recognized by event id OR idempotency key BEFORE any
    consequence (`DuplicateDelivery` — first settlement stands, mutated
    redeliveries cannot rewrite it); observation must cite its invocation's
    E5A event id as causal parent (ENV-I4); invocation must be the CURRENT
    generation of its slot (late observations refuse forever);
    work_request_id equality; any self-asserted `authority_decision_id` or
    embedded fabricated `authority_decision.decision_id` must equal the
    recorded one or the observation is refused (`SelfAssertedAuthority`).
    The settlement outcome reports the LEDGER-recorded authority decision —
    the outcome type carries no authority capability whatsoever, so a
    forged decision inside a response has no effect to attach to.
- Typed error family `BoundaryError` covering both directions.

### Wiring

- `crates/sea-forge-server/src/lib.rs`: one added `pub mod
  governed_execution_boundary;` line.
- `justfile`: `T05)` arm added to the `e2e-gate` case following the
  T02/T03/T04 arms (additive hunk inside the already-uncommitted e2e
  section; no other recipe touched).

### Composition with the existing runtime guarantees (not reimplementation)

I5's internal guarantee already exists: `ActionGrant` is move-only and
unserializable, minted only by `PolicyAuthorityEngine::grant*` from an Allow
decision bound to its committed ledger record (`grant_exact` refuses
verdict ≠ Allow), and consumed by `sea_forge_runtime::execute ::
authorize_execution` before sandbox selection. T05 makes the CROSS-COMPONENT
canonical statement machine-checked on top: no AuthorizedInvocation envelope
exists for a non-Allow decision, and no observation settles without the
exact authorized invocation. The full cycle test drives the REAL engine →
REAL ledger commit → REAL grant → REAL `sea_forge_runtime::execute` →
emission-before-execution ordering → settlement.

## Falsifier → proof map

| Plan tooth / frozen falsifier | Proof |
| --- | --- |
| TOOTH 1: force deny/escalate, attempt governed side effect ⇒ none occurs | FORGE-AUTH `t05_denied_authority_produces_no_invocation_and_no_side_effect` (real default-deny policy through real engine: emit refuses `AuthorityNotGranted{Deny}`; committed decision still mints no grant), `t05_escalated_authority_produces_no_invocation_and_no_side_effect` (real escalate rule ⇒ Escalate; same refusals); plus type-level unreachability of `execute` without a grant |
| TOOTH 2: late valid observation from previous invocation cannot settle current one | FORGE-OBS `t05_late_observation_from_previous_generation_cannot_settle_current_slot` (gen1 obs vs gen2 slot ⇒ LateObservation{1→2}; unknown invocation ⇒ UnknownInvocation; current generation still settles; lateness holds after settlement too), `t05_observation_must_cite_its_own_authorized_invocation_as_parent` (stripped/misattributed lineage ⇒ CausalityMissing naming the exact E5A id) |
| TOOTH 3: forge an authority decision inside the execution response ⇒ no authority effect | FORGE-OBS `t05_forged_authority_decision_inside_response_has_no_authority_effect` (embedded fabricated decision object ⇒ SelfAssertedAuthority recording recorded-vs-claimed; foreign top-level claim refused; true claim redundant; outcome always reports LEDGER-recorded id; outcome type exposes no capability) |
| Frozen falsifier: denied/escalated invocation produces the governed side effect | TOOTH 1 tests above + `t05_governed_side_effect_runs_only_after_authorization_and_emission` (positive control: side effect occurs ONLY after emission+grant through real components) |
| Frozen falsifier: late observation from invocation A settles invocation B | TOOTH 2 tests + cross-wire battery below |
| Cross-wired observation (work_request_id mismatch) | FORGE-OBS `t05_cross_wired_work_request_is_rejected` |
| Duplicate delivery duplicates consequence | FORGE-OBS `t05_duplicate_delivery_is_consequence_free` (exact replay, mutated-status redelivery, fresh-event-id same-idempotency rewrap — all DuplicateDelivery; first fact stands) |
| Self-authorized execution attempt (E5A direction) | FORGE-AUTH `t05_forged_authorized_invocation_stamps_are_refused_at_admission` (execution-environment + swe-seed/godspeed/context-kernel forging AuthorizedInvocation refused at admission; unknown stamp fails closed) |
| Unknown producer stamps, both directions | E5A admission test above + FORGE-OBS `t05_wrong_producer_on_observations_is_rejected_both_directions` (four known forgers + mystery stamp on E5B; AuthorizedInvocation delivered into the E5B edge ⇒ WrongEventType) |
| Missing/placeholder identity | FORGE-AUTH `t05_emission_refuses_placeholder_and_drifted_model_identity` (fallback constant incl. dc144cbd…, all-zero, malformed, drift); FORGE-OBS `t05_missing_or_placeholder_identity_in_observation_is_rejected` (missing hash, pseudo-hashes, drift, placeholder invocation id, missing required fields, invalid status vocabulary, wrong-typed optional refs) |
| Both Allow-gate conjuncts load-bearing | FORGE-AUTH `t05_both_allow_conjuncts_are_load_bearing_on_real_decisions` (real decision mutated to Allow+Degraded and Deny+Allow; both refused) |
| Correlation/causality emission gates | FORGE-AUTH `t05_emission_requires_meaningful_correlation_and_causality`; FORGE-OBS `t05_admission_requires_causality_full_payload_and_is_idempotent` (orphan E5A refused; each frozen field omission refused; ENV-I6 idempotent re-admission; re-authorization supersedes) |
| Positive path over real components | FORGE-AUTH `t05_governed_side_effect_runs_only_after_authorization_and_emission`, `t05_allow_decision_emits_the_frozen_authorized_invocation`, `t05_emitted_envelope_matches_the_canonical_wire_family` (independent recomputation of idempotency key); FORGE-OBS `e5b_valid_observation_settles_against_the_exact_authorized_invocation` |
| Real upstream composition (T04 surface) | FORGE-OBS `t05_real_t04_governed_work_request_feeds_invocation_causality` (REAL SWE_SEED GovernedWorkRequest fixture output supplies correlation + causal parent through emit→admit→settle; honest skip if fixture absent — not skipped in gate runs) |

Test inventory: `convergence_t05_authorized_execution` 10 tests +
`convergence_t05_execution_observation` 10 tests, all in
`crates/sea-forge-server/tests/`.

## Verification performed

```
cargo test -p sea-forge-server            # FULL crate suite: 25 test binaries ok, 0 failed
                                          # (convergence_t05_authorized_execution: 10 passed
                                          #  convergence_t05_execution_observation: 10 passed
                                          #  convergence_t04_governed_ingress: 12 passed — prior task intact;
                                          #  conformance_m16: 16 passed, 2 pre-existing ignored release-gates left alone)
cargo clippy -p sea-forge-server --all-targets   # zero diagnostics on touched files
                                                 # (pre-existing manifest license warnings elsewhere untouched)
rustfmt --edition 2021 --check <three touched Rust files>   # clean
just e2e-gate T05                         # exit 0 (20/20 across both suites)
just e2e-prereg-check                     # PASS before AND after work (SHA ef5710… intact)
HEAD: sea-rs 006daa2540678eaaf821d873266f79914b10c9af   # unchanged, no commits
```

Pre-existing dirty state preserved byte-for-byte (git status identical to
session start plus exactly the three new T05 files and the T05 justfile arm;
T01–T04 uncommitted artifacts untouched; SWE_SEED untouched entirely; no
`cargo clean`; worktrees untouched).

## Honest scope notes (what T05 does NOT prove)

- **Confirmation state: PENDING.** proof_level P3 requires fresh independent
  adversarial confirmation; this is builder evidence only. NOTHING is marked
  settled in `.agents/status/e2e-current-status.yml` (deliberately
  untouched). E5A/E5B/I5 remain open until the verifier returns CONFIRM.
- **T05 does NOT prove operational settlement correctness or proof success.**
  Whether a resulting execution satisfies its operational settlement criteria
  (E6, task T06) or its proof contract (task T07 scope) is explicitly out of
  scope per the plan's `does_not_prove`.
- **No live cross-process transport exists** between sea_forge and an actual
  external execution environment (Delta-0 stands). The boundary is proven as
  real-emitter/real-adjudicator code driven over the wire-shape JSON family,
  with the positive cycle running REAL processes through the REAL sandbox via
  the production seam. Physical dispatch wiring (handing the emitted envelope
  to a remote executor and receiving observations back over a socket) remains
  open debt; this gate must be composed there, not reimplemented.
- **The emitter/adjudicator are library surfaces without a non-test caller
  yet**, identical to the settled T03/T04 pattern; wiring into
  `case_dispatch`'s episode path (which already performs evaluate→commit→
  grant→execute) is additive future work and needs owner approval before any
  public-interface change.
- **Slot generations derive from re-authorization events admitted at this
  boundary.** A deployment that restarts between authorization and
  observation must reload its ledger from durable records before the
  "late observation" property holds again; the in-memory registry composes
  with (does not replace) the existing append-only evidence substrate.
