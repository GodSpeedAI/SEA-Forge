# E. Integration architecture

Baseline for implementation. Each decision names its owner, the existing mechanism reused, and the smallest
change. `SOP §` cites `sop/GS-FDE-SOP-001-v0.4.md`; `M§` cites `Mission.md` task sections. Decision IDs (`AD-n`)
are referenced from the plan and the EGM record.

## 1. Ownership (unchanged, made explicit)

| Concern | Sole writer | Readers / projectors | Never |
|---|---|---|---|
| Meaning: `.sea`, `world_ref`, concept identity, policy semantics, semantic diff | DomainForge | all | no other repo parses `.sea` or mints a `world_ref` |
| Case, plan, gates, episodes' frozen references, approvals, dispositions, world registry and promotion, settlement decision | SEA-Forge / Casework | Horizon, Workbench, Cognate (via SFWP / authority socket) | Horizon and Workbench hold no case truth, only rebuildable projections |
| Runs, events, continuations, assignments' execution, artifacts, operator/agent identity contract | Cognate | Workbench, Horizon | Cognate does not decide authority |
| Expectation, observation, evidence, residuals, observed interaction representation, structural comparison, diagnosis, proposal records | RealityTrace | SEA-Forge (corroboration), Horizon | RealityTrace never executes, authorizes, or edits a model |
| Operator navigation, decision presentation, journal, handoff rendering | Horizon | — | "the cockpit never decides" |
| Agent work sessions, terminal, browser, spend display | Cognate Workbench | Horizon (deep links, assign) | no case or approval truth |
| Retrieval with citations | Context Kernel | Workbench, SWE_SEED, Horizon | not canonical truth |
| Frozen-reference execution verification | Gauntlet, SWE_SEED | RealityTrace (as observations) | no authority |

## 2. Identities (one spelling each)

| Identity | Format | Minted by | Carried in |
|---|---|---|---|
| `world_ref` | `world:<name>@sha256:<64hex>` | DomainForge (`application/world.rs`) | every case, episode reference, event, RealityTrace record, Horizon frame |
| `declaration_key` | `concept:<uuidv5>` (DomainForge `ConceptId`) | DomainForge | interaction events; RealityTrace subject refs (existing `concept:` prefix) |
| `case_id`, `plan_item_id`, `run_id` | existing Casework values (no grammar change) | SEA-Forge | events, RealityTrace `--extensions`, Horizon frames |
| `episode_ref` | `episode:<case_id>:<plan_item_id>:<n>@sha256:<digest of canonical frozen reference>` | SEA-Forge at freeze | RealityTrace `question.declared` / `threshold.declared`, Cognate run `correlationId` |
| `assignment_id` | Casework delegation id (existing `agent_run.delegate`) | SEA-Forge | Cognate `correlationId = case:<case_id>/assign:<assignment_id>`, Workbench session |
| RealityTrace ids | existing ULIDs and `question:`, `threshold:`, `observation:`, `evidence:`, `settlement:`, `residual:` prefixes | RealityTrace | Horizon, evidence bundle |
| `decision_id` | SEA-Forge CEP decision, scoped by `operation_id` | SEA-Forge | events, approvals, world-transition record |
| Principal | Cognate identity contract: operator, device, session, agent, execution, attempt | Cognate identity | every authenticated request |

**AD-1.** No new identifier grammar is introduced except `episode_ref`, which is a composition of existing ids
plus a digest. Hashing uses SHA-256 over RFC 8785 canonical JSON; CEP-0008 is amended to say so (closes the
undefined-algorithm debt; Context Kernel already implements it).

## 3. The Field Case in Casework (AD-2)

A Field Case is a Casework case whose plan comes from template `fde.field_case@1`. No new aggregate.

| SOP concept | Casework construct (existing unless marked NEW) |
|---|---|
| Field Case (`SOP §1.1`) | `CasePlan` with `template_ref = "fde.field_case@1"` |
| Case content groups (`SOP §4.2`) | NEW `field_case` document in `cases/<id>/field_case.json`, schema-validated, versioned by ledger event `field_case.revised`; every field carries `status ∈ declared, observed, inferred, proposed, unknown` and a source ref |
| Activity (`SOP §6`) | discretionary `PlanItem` (`max_instances > 1`, no `depends_on` ordering), `markers.activity = embed…compound` |
| Episode (`SOP §2.1`) | one run of a plan item (`HorizonItem.run_ids` already models repeats) plus NEW ledger event `episode.frozen` holding the canonical reference and digest, and `episode.kind ∈ evaluated, exploratory` |
| Case Assignment | existing delegation (`agent_run.delegate`, `delegation.preview`), extended with the `SOP §7.4` brief as a typed payload |
| Evidence gate (`SOP §8`) | `ItemKind::Milestone` item per gate plus NEW ledger event `gate.assessed {gate, state, evidence_refs, rationale, assessor}`; state `not_applicable` requires an approval when the gate is hard |
| Epistemic result | NEW `episode.settled {episode_ref, result ∈ S,F,U,C, reliability, independence, settlement_ref}`; written only by a principal that is not the episode's executor (enforced, reuses `proposed_by` separation-of-duties check) |
| Disposition (`SOP §7.5`) | NEW `disposition.recorded` behind the existing approval path; `suspend` parks the case (`ParkHumanTask`) |
| Case lifecycle axis | derived, no enum change: `open = Active \| AwaitingApproval` without a suspend disposition; `suspended` = parked by disposition; `closed = Completed \| Terminated` |
| Strategy decision (`SOP §4.4`) | NEW `strategy.recorded` event, revisable |
| Continuation (`SOP §10.1`) | NEW read verb `case.continuation` computed from the ledger plus RealityTrace and Cognate references; never stored as truth |

New SFWP verbs: `case.field.get`, `case.field.revise`, `case.episode.freeze`, `case.episode.settle`,
`case.gate.assess`, `case.dispose`, `case.strategy.record`, `case.continuation`. Each is a typed frame with golden
fixtures in `sfwp/testdata/frames.json`, mirrored in the Go client and generated TypeScript contracts.

**Invariant I-1 (freeze before move).** `agent_run.delegate` and any governed operation tagged with an evaluated
episode is refused unless `episode.frozen` for that `episode_ref` precedes it in the ledger.
**Invariant I-2 (no self-settlement).** `case.episode.settle` is refused when the caller's principal chain
intersects the executor's.
**Invariant I-3 (frozen means frozen).** A second `episode.frozen` for the same `episode_ref` with a different
digest is refused; changed criteria create episode `n+1`.

## 4. The declared model (AD-3)

- Source: `sop/GS-FDE-SOP-001-v0.4.md`, promoted to `sea-rs/docs/sop/` and hash-pinned.
- Model: `sea-rs/.sea/interaction/fde-sop/` following the existing IDM method (model, `source-translation-map.md`,
  `assumptions-and-unknowns.md`, `validation/`). World name `godspeed-fde-sop`.
- Expression in today's grammar (no grammar change): roles and `role_binding` for `SOP §3.1`; entities and
  resources for case, episode, assignment, gate, evidence; one `operation` per navigation step and activity with
  `intent`, `access policy_governed by … at precondition|postcondition`, `evidence operation_trace`; `policy`
  declarations (Obligation / Prohibition) for the invariants (freeze before move, no self-settlement, hard gates,
  authorization at the action boundary, exploratory labelling); `flow` for evidence and handoff movement;
  `metric` for payment currencies.
- Each assertion row in the translation map carries `SOP §` and status (`declared | inferred | unknown`).
- Validation: real `domainforge validate` and `domainforge envelope --emit cep`; record source hash, `world_ref`,
  compiler version. `domainforge test` (a stub that exits 0) is not used as evidence.
- The Casework template `fde.field_case@1` is generated from the model's operations and gates and stores the
  `world_ref` it was generated from, so template and model cannot drift silently.

## 5. Semantic exhaust (AD-4)

One event contract, `godspeed.interaction_event/1`, a CEP-0008 semantic envelope profile:

```
world_ref, declaration_key            # which model, which element
actor {principal, role}, operation, resource, context {case_id, plan_item_id, episode_ref?, assignment_id?}
correlation_id, causation_id
authority {decision_id | "not_required" | "unknown"}
expectation_ref?                       # RealityTrace question/threshold ids, present for evaluated episodes
outcome {status, effect_refs}          # observed, never inferred
evidence_refs[], settlement_ref?
source {system, record_id, recorded_at}  # pointer to the untouched source record
```

Emitters (existing surfaces, extended):
- SEA-Forge: case ledger events and authority decisions (closes the "no production `EvidenceRecorded` producer" gap).
- Cognate: run and invocation events through the existing RealityTrace observation outbox (`packages/realitytrace`);
  finish `feat/realitytrace-real-snapshot`.
- Workbench: existing `ObservationDelivery`.
- Gauntlet and SWE_SEED: verification records and `ProofCompleted` (existing E7 path).

Ingest: RealityTrace `ingest` using the `agent-event-v1` path already on its main (#13), extended with this
profile. An event missing `world_ref`, `declaration_key` or source pointer is stored as `ingest.rejected` or as an
unanchored occurrence; it can never support a verified semantic conclusion (M§7).

Ordering proof: the frozen reference is written as `question.declared` and `threshold.declared` before the run;
RealityTrace's `allowed_predecessors` and hash chain make later rewriting detectable (M§3, M§7).

## 6. Observed interaction representation and comparison (AD-5)

New RealityTrace crate `sxr-observed` and CLI group `realitytrace observed`:

- `observed derive --world <world_ref> --scope case:<id> [--until <record>]` computes, from ledger records only:
  nodes (one per `declaration_key`, plus `undeclared:<sha256 of operation+resource kind>` for unanchored
  behaviour), directly-follows edges per correlation, and per node/edge: occurrence ids, frequency, ordering
  statistics, actor-role and context distribution, authority outcomes, coverage (which emitters reported) and an
  uncertainty note. Output is deterministic canonical JSON stored as `representation.recorded` in CAS with the
  input record-set hash. Same inputs give the same hash; removing or changing a record changes it.
- `observed compare --declared <snapshot_id> --observed <representation_id> [--mapping <file>]` emits one
  `residual.computed` per difference with class
  `confirmed | contradicted | observed_undeclared | declared_unobserved | insufficient`, the affected
  `declaration_key`s, occurrence ids, context, and the rule that fired.
- Classification is delegated to DomainForge, which owns policy semantics: the tool generates an instances overlay
  (`instance … of …` facts for each occurrence, importing the declared model) and runs `domainforge validate`.
  Three-valued results map directly: true to `confirmed`, false to `contradicted`, unknown to `insufficient`.
  Occurrences with no declared element are `observed_undeclared`; declared elements with zero occurrences are
  `declared_unobserved` when their emitter reported coverage and `insufficient` when it did not.
- The overlay is a computed artifact with its own hash; it is never hand-edited (M§4).
- A spike (plan WP-10) proves the overlay-plus-validate path on a ten-occurrence fixture before the rest is built.
  Fallback if DomainForge cannot evaluate policies over generated instances: the classifier evaluates the same
  declared policies from DomainForge's AST JSON inside `sxr-observed`, and the gap is filed against DomainForge.

Diagnosis and proposal reuse existing record kinds: `diagnostic.hypothesis`, `diagnostic.settled`,
`proposal.raised` (which already requires a DomainForge semantic diff).

## 7. Governed structural revision (AD-6)

1. `proposal.raised` in RealityTrace references the residuals and the diagnosis.
2. Candidate `.sea` on a branch with a `ConceptChange` declaration; DomainForge validates and produces the
   semantic diff; RealityTrace records `semantic.snapshot` and `semantic.diff`.
3. Regression: `observed compare` re-run with the candidate against held-out prior occurrences. Any prior
   `confirmed` that becomes `contradicted` fails the candidate.
4. Authorization: SEA-Forge approval-gated world transition (existing Stage 10 mechanism) with a CEP world-
   transition record `transition_kind = semantic_change`. The approver must differ from the proposer; a human-
   reserved approval requires an A3-UV assertion bound to the operation digest (§8). Denied, unknown or
   unavailable authority leaves the registry unchanged.
5. Promotion pins the new `world_ref` in the world registry. The old world stays resolvable; every prior record
   keeps its original `world_ref`. Recovery is re-pinning the previous world through the same gated transition.
6. A subsequent episode is frozen under the new `world_ref`; the proof compares what it can now distinguish.

World-registry persistence (currently in-memory, debt M-06) becomes durable as part of this work, since
"preserve the previous version" cannot rest on process memory.

## 8. Identity and authority (AD-7)

Adopt the owner-selected design in `Horizon-gaps/.agents/reports/2026-11-08-unified-human-agent-identity-authority.md`
(profile S1, operation classes P0–P3, assurance A3-UV) and finish its stages:

| Stage | Owner | State | Remaining |
|---|---|---|---|
| 0B interim containment | Horizon | built on `horizon-desk-gaps`; enrolment proven against a test double; production refuses | D3 route-scope table, D5 saga and reconciliation, PR split |
| 1 shared identity | Cognate | `packages/identity` on `horizon/shared-identity`, unconsumed | rename to `@godspeedai/cognate-identity`, D2 evidence-verified issuance and `pending_activation`, wire Horizon and Workbench (removing Workbench's duplicate WebAuthn/CBOR) |
| 2 SEA-Forge authority bridge | SEA-Forge | not started; `feat/horizon-desk-support` is adjacent | D1 enrolment decision API bound to digest and expiry; D4/D6 `bootstrap_controller` and `recovery_controller` classes |
| 3 Gauntlet binding | Gauntlet | not started | binding record before launch |
| 4 consumer migration | Workbench, Horizon | not started | sessions, experts, agent door on the shared contract |
| 5 qualification | all | not started | real-boundary tests; skipped prerequisite = unqualified |

Application to the SOP: every `SOP §7.5` disposition, every gate set to `not_applicable` on a hard gate, every
`episode.settle`, and every world promotion is a P3 human-reserved or independently authorized operation.
Horizon's open critic item 4 (an agent driving the paired browser can act as the person) is closed for these
operations by A3-UV at the SEA-Forge boundary; ordinary P0/P1 desk interaction stays session-based.

## 9. Projections and Horizon

Horizon adds read adapters only: `field-case.ts` (SFWP `case.field.get`, `case.continuation`), and
`realitytrace-structure.ts` (CLI JSON for `observed` and residual records). Both feed the existing
`DebuggerSources`, so the seven operations answer over real records:

| Question (M§6) | Operation | Source |
|---|---|---|
| why was this difference classified so | `why` | residual record: rule, occurrences, declared element |
| declared v_n versus v_n+1, or two checkpoints | `diff` | DomainForge semantic diff; never rewrites the frame's checkpoint |
| operational history and provenance | `timeline` | case ledger joined with RealityTrace records |
| consequences of a candidate revision | `counterfactual` | regression comparison under the candidate; never executes |
| is a move available, and what blocks it | `reach` | extended from capability rungs to the seven Aim dimensions (`SOP §7.2`) |
| re-derive a settlement | `replay` | recomputes observed representation and comparison from records |

Parity rule: each Horizon answer embeds the record ids and digests it was built from; an acceptance test asserts
byte-equality of the classified result with the CLI output for the same ids (M§7 last bullet).

## 10. Horizon and Workbench boundary (AD-8)

Horizon is where the case is understood and decided. Workbench is where delegated work is executed. The seam:

- Horizon → Workbench: existing Connect RPC `assign`, carrying `assignment_id`, `episode_ref`, the `SOP §7.4`
  brief and budget. Workbench opens a session with `correlationId = case:<id>/assign:<id>`.
- Workbench → Horizon: session deep link `#session/<id>[/<eventId>]` stored on the assignment; Workbench shows a
  read-only case strip (case, world, episode, budget, authority state) fetched from SEA-Forge, and a "back to
  case" link.
- Approvals never happen in Workbench for P3 operations; Workbench shows "awaiting decision" and links to the
  Horizon decision card.
- One operator identity (Cognate identity contract) signs in to both.
- The two design systems stay separate; shared vocabulary for states and the case strip is the unifying element.

## 11. Recovery

| Failure | Behaviour |
|---|---|
| RealityTrace unavailable at ingest | emitters queue in their durable outbox; episode cannot settle; result stays `U` |
| SEA-Forge unavailable or decision unknown | governed operations refuse (existing fail-closed); no promotion |
| Interrupted assignment | Cognate continuation resumes; Casework records interruption; budget and partial evidence retained |
| Crash during enrolment or promotion | saga state plus reconciliation job (D5); uncertain outcomes fail closed |
| Bad promotion | re-pin previous `world_ref` via gated transition; evidence untouched |
| Projection drift in Horizon | projections are rebuildable; `last_event_id` mismatch forces refetch |

## 12. Integrated release gate (AD-9)

`godspeed-readiness` gains profile `fde_field_case`: pinned heads for every repo including DomainForge; per-repo
gates; contract gates (shared schemas identical across repos); the Field Case end-to-end proof (deliverable G);
negative suite; installation from release artifacts on a clean cell. GO requires every gate executed; a skipped
gate is NO-GO.
