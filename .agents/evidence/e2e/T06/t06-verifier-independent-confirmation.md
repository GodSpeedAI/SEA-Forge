# T06 Independent Adversarial Verification (fresh verifier)

Verifier: fresh independent adversarial session; did not build T06.
Date: 2026-08-25
Scope: falsify frozen E6/I6/I7 against implementation; verdict CONFIRM | NOT_CONFIRM.

## Freeze state (before)

- `just e2e-prereg-check` (sea-rs): **PASS**, exit 0.
- preregistration SHA256: `ef5710893c5bcc569a4757371c54d4d2e5e6168e1df61d2d7433b6889eaa879f` (matches expected).
- plan SHA256: `0c57e54310d08b3a01d3b001ac9d8baf859a930e97e8b04267d9eba797bd0e39`.

## Frozen contract under test (read before any implementation inspection)

- `edges.E6`: OperationalSettlement, sea_forge -> swe_seed, P0. Required payload:
  work_request_id, authority_decision_id, execution_status, observed_effects,
  operational_settlement_status, evidence_refs, domain_model_ref.
  Invariants: settlement determined independently from process exit status;
  operationally successful execution does NOT imply proof success; settlement
  does not update developmental capability.
- `global_invariants.I6`: proof and operational settlement remain distinct facts
  (three-way distinction: exit status vs operational settlement vs proof).
- `global_invariants.I7`: operational settlement cannot directly promote
  developmental state.
- `settlement_semantics`: owner of operational_settlement = sea_forge;
  owner of proof = swe_seed; owner of developmental_settlement = godspeed_agent.
- `event_ownership`: OperationalSettlement producer = sea_forge exclusively.
  SettlementRecorded / CapabilityUpdated producers = godspeed_agent exclusively.
- Identity rules ENV-I1..I8 (fail-closed model identity, stable work_request_id,
  causal-parent citation, no silent rewrite, idempotent durable writes,
  content-addressed evidence where possible, conformance != truth).
- Plan teeth (preregistered falsifiers): exit_code=0 forcing settlement success
  despite failed criteria; OperationalSettlement directly creating GSA
  SettlementRecorded/CapabilityUpdated.

## Prior-boundary debt that MUST NOT be reproduced here

- T04 D1 (field-validation completeness gap: projection fields unchecked),
- T04 D2 (mandatory causality not enforced beyond one parent type),
- T04 D3 (namespace not separately checked at gate),
- T05 A9 (forged-producer envelope admits on shape alone),
- T05 A10d (re-wrap under fresh event_id w/o idempotency key double-settles),
- T05 A11 (restart replay re-admits and re-settles).

## Planned attack battery (designed from contract BEFORE reading t06-report.md)

Preregistered falsifiers:

| ID | Attack | Expected if contract holds |
|----|--------|----------------------------|
| F1a | exit_code=0 + declared criteria violated (evaluator seam) | settlement != success |
| F1b | exit_code=0 + empty/vacuous observed_effects | settlement != success |
| F1c | exit_code!=0 + all declared criteria satisfied | evaluator decides independently (not auto-fail solely on exit code) |
| F1d | exit_code=0 + execution_status=failed/failure_details present | conflict surfaced, settlement != success |
| F1e | force-success attempts at every seam (evaluator fn args, emitter input, wire validator bypass, SWE_SEED adjudicator input) | no seam lets caller dictate settlement status |
| F2a | feed OperationalSettlement into every exposed SWE_SEED/GSA-facing derivation path | no SettlementRecorded/CapabilityUpdated produced |
| F2b | type/constructor confusion: build SettlementRecorded/CapabilityUpdated directly from settlement payload/types | impossible without fabricating a GSA-authored event outside this boundary |
| F2c | registry/event-type confusion: register/consume OperationalSettlement as SettlementRecorded/CapabilityUpdated/ProofCompleted | rejected or inert |
| F2d | producer forgery -> settlement stamped producer=swe_seed (or other) | rejected |
| F2e | derivation forgery: settlement used as causal parent to mint a ProofCompleted claiming proof success | no such derivation exists on this boundary |

Three-way-distinction probes (I6):

| ID | Attack | Expected |
|----|--------|----------|
| I6a | settlement status mapped/copied into any proof-status field anywhere on surface | absent |
| I6b | proof outcome inferred from settlement by consuming side | not derivable |

Fresh compositions:

| ID | Attack | Expected |
|----|--------|----------|
| A1 | observed_effects mutated post-admission under stable identity (ENV-I5/I10) | mutation rejected/impossible (immutable upstream facts) |
| A2 | conflicting resettlement: settle, flip operational_settlement_status, resettle | second settlement cannot silently flip recorded fact (idempotency/dedup) |
| A3 | wrong work_request binding: settlement for WR-X bound to WR-Y | rejected (stable correlation) |
| A4 | replay across restart: full cycle, rebuild, redeliver identical + mutated envelopes | characterize durability vs ENV-I6 (watch T05-A11 reproduction) |
| A5 | placeholder/fallback identity: constant/pseudo model_hash; domain_model_ref omitted or pointing elsewhere | fail closed (ENV-I2; watch T04-D1/D2 reproduction) |
| A6 | causality gaps: strip E5A/E5B parents from provenance | rejected (ENV-I4; watch T04-D2 reproduction) |
| A7 | missing-required-field battery: omit each of the 7 E6 required fields one-at-a-time | each rejected (watch T04-D1 reproduction) |
| A8 | namespace/schema tampering: wrong domain namespace, schema_name/version swap | rejected (watch T04-D3 reproduction) |
| A9 | mutable path-only evidence_refs where content addressing available (ENV-I7) | characterize; content-addressed refs preferred |
| A10 | authority_decision_id fabricated/nonexistent/cross-bound | rejected or traceable; no silent acceptance of invented authority |
| A11 | duplicate delivery under fresh envelope_id w/o idempotency key + mutated status (T05-A10d echo) | dedup or refusal; no double-settlement flip |
| A12 | exit_code coercion via wrapper: run real binary exiting 0 while producing failed-effect evidence end-to-end through emitter->validator->adjudicator | settlement records failure |

Execution method: out-of-repo harness(es) in /tmp/opencode/t06-verifier with
path deps on sea-forge-server and swe-seed-core crates; empirical, not static-only.

## Results

Method: two out-of-repo cargo harnesses (`/tmp/opencode/t06-verifier/{forge-side,seed-side}`,
path deps on `sea-forge-server` / `swe-seed-core`, isolated target dir), executed
empirically against production code only. Forge side drives a REAL
`PolicyAuthorityEngine` Allow decision → real `InvocationLedger` admit/settle →
evaluator → emitter → wire validator, then hands emitted envelopes to the seed
side for cross-repo adjudication over the durable `IdempotencyLedger`. Both
runs deterministic; battery re-run twice with identical outcomes. Raw outputs
reproducible via `/tmp/opencode/t06-verifier/target/release/{forge-side,seed-side}`.

Legend: **HOLD** = attack failed to falsify (contract held). **FALSIFIED** =
counterexample found (none). **OBSERVED-debt** = characterization accepted by
design/disclosure, recorded as candidate debt, non-falsifying.

### Preregistered falsifiers

| ID | Result | Evidence |
|----|--------|----------|
| F1a | HOLD | `completed` + empty effects → evaluator `Rejected`, wire `operational_settlement_status="rejected"`; self-wire-validation passes |
| F1b | HOLD | `completed` + unrelated effects ("hello world") → `Rejected` |
| F1c | HOLD | `timed_out` even WITH criterion text in effects → `Rejected` basis `[timed_out]` (independence holds in BOTH directions) |
| F1d | OBSERVED-debt | criterion string echoed verbatim into effects ⇒ `Accepted` although `failure_details` present and nothing real done — upstream `sea_forge_settlement::evaluate_agent_output` is substring-containment (builder-disclosed "Declared-criteria semantics"); failure_details are not evaluator inputs. Not exit-derived; ENV-I8/I13 truth-gap, candidate debt |
| F1e | HOLD | redelivery of an already-settled observation settles nothing again and evaluates to `NothingSettled`; duplicate delivery projects NO envelope. (In-process fabrication of an `ObservationOutcome::Settled` struct reflects its inputs — pure-function seam, characterized; wire-level trust placement unaffected.) |
| F2a | HOLD | no function anywhere on either side composes settlement facts into a developmental event; `OperationalSettlementFacts` consumed by nothing but the adjudicator (grep + runtime) |
| F2b | HOLD | `emit_settlement_recorded`/`make_event("CapabilityUpdated")` built from settlement facts stamp `swe-seed` → producer gate `NotAuthoritative{authoritative: godspeed_agent}` — swe_seed cannot author developmental events, so no promotion path closes |
| F2c | HOLD | settlement delivered to the SettlementRecorded consume path → `WrongType`; ProofCompleted delivered to the adjudicator → `WrongType` |
| F2d | HOLD | forged stamps on OperationalSettlement refused BOTH directions: `swe_seed`, `godspeed_agent`, `execution_environment` → `NotAuthoritative{authoritative: sea_forge}` (forge-side wire validator AND seed-side registry) |
| F2e | OBSERVED | generic `derive_event` machinery can cite ANY parent under ANY child type (ProofCompleted stamped swe-seed = lawful producer) — but no production path composes it; adjudication returns facts only. Generic-machinery existence ≠ derivation; recorded |

### Three-way distinction (I6)

| ID | Result | Evidence |
|----|--------|----------|
| I6a | HOLD | proof vocabulary smuggled into `operational_settlement_status`: `proof_passed`/`passed`/`success`/`proved`/`proof_success` ALL → `InvalidSettlementStatus` (both sides) |
| I6b | HOLD | `OperationalSettlementFacts` debug-shape audit: zero occurrences of proof/capability/developmental terms; outcome type is its own two-variant `OperationalOutcome` |

### Fresh compositions

| ID | Result | Evidence |
|----|--------|----------|
| A1 | HOLD | mutation ladder: same event_id + mutated effects → `DuplicateDelivery`; fresh event_id + same content-derived key + mutated effects → `DuplicateDelivery` — first settlement stands |
| A2 | HOLD | status flip `accepted→rejected` under fresh identity, same chain (wr+decision) → `ConflictingResettlement` |
| A3 | HOLD | settlement consumed against different originating work_request_id → `CrossWiredWorkRequest{expected wr-unrelated-999, got wr-t06v-a12}` |
| A4 | HOLD | adjudicator dropped, ledger reopened from disk, exact redelivery → `DuplicateDelivery` (durable JSONL). T05-A11 NOT reproduced at this edge |
| A5 | HOLD | fallback pseudo-hash `dc144cbd…` and all-zero digest refused BOTH sides (`PlaceholderIdentity`); declared-vs-local drift refused (`DomainDrift`); `domain_model_ref.model_hash` swap → `RefIdentityMismatch`; omitted ref → `OpaquePayload` |
| A6 | HOLD | stripped parents → `CausalityMissing(expected=[E5A,E5B], recorded=[])`; well-formed junk parent only → `CausalityMissing`; malformed parent id → T01 `Boundary(Causality)`; empty expected chain → `EmptyChainBinding`. T04-D2 NOT reproduced (mandatory binding to KNOWN chain ids, not mere presence). Note: forge-side wire check enforces ≥1 parent only — composing strength lives on the consumer side (asymmetry noted) |
| A7 | HOLD | each of the seven frozen fields omitted one-at-a-time → `OpaquePayload` naming it, BOTH sides; three-at-once reports the complete list. T04-D1 NOT reproduced |
| A8 | HOLD | payload-namespace AND domain_model_ref-namespace swaps → `WrongNamespace` both sides; schema_version `v2` → `MalformedEnvelope`. T04-D3 NOT reproduced |
| A9 | OBSERVED-debt | mutable path-only `file:///tmp/evil/...` evidence_refs ACCEPTED by both sides' validators (ENV-I7 is SHOULD-level); the EMITTER itself always emits `sha256:<digest>` + event-id pointers (verified on real output). Candidate debt: tighten to content-addressed-only where substrate permits |
| A10 | OBSERVED-debt | fully-formed settlement with fabricated `authority_decision_id="dec-FABRICATED-not-in-any-ledger"` adjudicates `First` — consumer has no authority ledger to bind authenticity (producer-authority trust placement, T05-A9 class; placeholder decision ids ARE refused). Producer-side authenticity is pinned upstream by the T05 SelfAssertedAuthority gate. Candidate debt: carry verifiable decision provenance |
| A11 | HOLD | fresh event_id + NO idempotency key + mutated effects on an already-settled chain → `ConflictingResettlement` (durable chain keys close the T05-A10d hole at this edge) |
| A12 | HOLD | END-TO-END cross-repo: REAL engine Allow → emit E5A → admit → settle `completed` observation whose effects fail declared criteria → emitter produces `operational_settlement_status="rejected"` → SWE_SEED adjudicates `First(Rejected)`. Exit-zero coercion impossible across the full production chain |

### Prior-boundary debt reproduction check (mandated)

- T04-D1 (field-completeness): **not reproduced** — all seven fields enforced with typed shape checks both sides (A7).
- T04-D2 (mandatory causality): **not reproduced** — binding to known chain ids mandatory, junk/orphan/stripped parents all refused (A6).
- T04-D3 (namespace): **not reproduced** — checked at payload AND domain_model_ref level, both sides (A8).
- T05-A9 (forged admission): **partially analogous trust placement remains** (A10 decision-id acceptance), strictly narrower than T05's; producer stamps themselves fail closed here.
- T05-A10d (stealth re-wrap): **closed** at this edge (A11).
- T05-A11 (restart replay): **closed** at this edge for the consumer ledger (A4); emitter-side InvocationLedger remains in-memory — builder-disclosed T05-scope debt, unchanged.

### Divergences vs builder claims (t06-report.md)

Builder claims verified TRUE: tooth-1/tooth-2 proofs reproduce independently;
14+13 test counts match actual runs; gates green; prereg SHA intact; scope
notes (containment criteria semantics, in-memory emitter-side ledger, no live
transport, library-without-runtime-caller status) are honest and were
confirmed rather than contradicted. Zero-behavior-change claim on
`governed_execution_boundary.rs`: files are untracked so git cannot prove the
delta; verified instead by full-file inspection (zero settlement-return logic
in that module) plus T05 suites passing post-change (convergence_t05_*: 20/20).

Undisclosed minor observations (all non-falsifying):
1. Wire validators accept non-content-addressed path-style evidence_refs (A9) — ENV-I7 is SHOULD-level; emitter never produces them.
2. Consumer cannot verify `authority_decision_id` authenticity beyond presence/placeholder rules (A10).
3. Legacy pre-T01 `consume_settlement_recorded` lacks producer validation — a swe-seed-stamped forged SettlementRecorded payload would be read by that legacy log-only reader; it CREATES nothing and is unreachable from the E6 boundary (type-refused), out-of-scope observation for the T01-era surface.
4. Forge-side wire causality check is ≥1-parent only; strictness concentrated consumer-side (combined property holds).

## Verdict

**CONFIRM.**

Basis, per the required conjunction:
1. **Frozen requirements survive review**: every E6 payload requirement and invariant clause was attacked through every exposed seam on both sides; none yielded.
2. **Production-path implementation composing real machinery**: the boundary is composed of the real `PolicyAuthorityEngine`, real `InvocationLedger` settle path, kernel `SettlementStatus`/settle basis vocabulary, content-addressing over canonical JSON, and the durable `IdempotencyLedger` substrate — my harness drove the complete chain over these real components with zero mocks AT THIS BOUNDARY (harness-fabricated envelopes were attacks, not scaffolding). Caveat carried from builder disclosure, matching settled T03–T05 precedent: neither module has a runtime dispatch caller yet; physical transport composition is later-task work by plan.
3. **Gates green**: `cargo test -p sea-forge-server --test convergence_t06_operational_settlement` → 14 passed, exit 0; `cargo test -p swe-seed-core --test convergence_t06_operational_settlement` → 13 passed, exit 0; `just e2e-gate T06` → exit 0 (both suites inside). Re-run twice this session, identical results.
4. **No attack falsified**: 30+ empirical probes across F1/F2/I6/A families; zero falsifications.
5. **No known counterexample remains**: the three-way distinction (execution result vs operational settlement vs proof) survived direct attack, cross-repo composition, mutation ladders, restart, and forgery both directions.
6. **Nothing rests solely on synthetic/mock behavior at this boundary**: tooth-1 was proven end-to-end over the real engine and real ledgers, cross-repo, twice.

Non-falsifying debts recorded above (F1d echo-gaming of containment criteria,
A9 path-only refs, A10 decision-id trust placement, legacy consumer producer
gap) belong in `.agents/OBSERVED_DEBT.md` ownership of the builder/operator;
per discipline this verifier does not implement fixes.
