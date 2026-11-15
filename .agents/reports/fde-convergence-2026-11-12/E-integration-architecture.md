# E. Integration architecture (draft 2)

Baseline for implementation. `SOP §` cites `sop/GS-FDE-SOP-001-v0.4.1.md`; `M§` cites `Mission.md`; `RD-n` cites
`H-egm-review-record.md`, which governs where this text and H differ. WP ids come from `_revision-ids.md` and
match `F-implementation-plan.md` (draft 2). `AD-1…AD-9` and `I-1…I-3` are stable. Draft 1 is in `draft1/`.

## 1. Ownership (unchanged, made explicit)

| Concern | Sole writer | Readers / projectors | Never |
|---|---|---|---|
| Meaning: `.sea`, `world_ref`, concept identity, policy semantics and evaluation, semantic diff | DomainForge | all | no other repo parses `.sea`, mints a `world_ref`, or evaluates a policy |
| Case, plan, gates, frozen references, approvals, dispositions, world registry and promotion, settlement decision | SEA-Forge / Casework | Horizon, Workbench, Cognate (SFWP / authority socket) | Horizon and Workbench hold no case truth, only rebuildable projections |
| Runs, events, continuations, assignments' execution, artifacts, operator/agent identity contract | Cognate | Workbench, Horizon | Cognate does not decide authority |
| Expectation, observation, evidence, residuals, observed representation, structural comparison, diagnosis, proposals | RealityTrace | SEA-Forge (corroboration), Horizon | never executes, authorizes, edits a model, or evaluates policies |
| Navigation, decision presentation, journal, handoff rendering | Horizon | none | "the cockpit never decides" |
| Agent work sessions, terminal, browser, spend display | Cognate Workbench | Horizon (deep links, assign) | no case or approval truth |
| Retrieval with citations | Context Kernel | Workbench, SWE_SEED, Horizon | not canonical truth |
| Frozen-reference execution verification | Gauntlet, SWE_SEED | RealityTrace (as observations) | no authority |

Policy semantics have one owner (RD-08): where DomainForge cannot evaluate an expectation the result is
`insufficient`; there is no second evaluator.

## 2. Identities (one spelling each)

| Identity | Format | Minted by | Carried in |
|---|---|---|---|
| `world_ref` | `world:<name>@sha256:<64hex>` | DomainForge | every case, episode, event, RealityTrace record, Horizon frame |
| semantic reference | `namespace::name` | model author | events, alongside `world_ref` |
| `declaration_key` | `concept:<uuidv5>` (`ConceptId`) | UUIDv5 of `namespace::name` only (`domainforge-core/src/concept_id.rs`), so stable across world versions; `world_ref` is carried beside it, never mixed in (RD-22) | events; RealityTrace subject refs. Emitters send all three; RealityTrace resolves `namespace::name` against the pinned world's semantic snapshot, rejects an event whose element does not exist there, and never trusts the executor's claim about the key (RD-22) |
| `case_id`, `plan_item_id`, `run_id` | existing Casework values | SEA-Forge | events, frames |
| `n` | integer per `(case_id, plan_item_id)` | case keeper, at freeze | `episode_ref`, I-3 |
| `episode_ref` | `episode:<case_id>:<plan_item_id>:<n>@sha256:<digest>`, the digest computed over the frozen-reference body (`case_id`, `plan_item_id`, `n`, `world_ref`, criteria, evidence sources; it does not contain `episode_ref`) (RD-23) | SEA-Forge at freeze | RealityTrace question/threshold, event context |
| `assignment_id` | `assign:<caseId>:<stepId>:<n>:<k>` (RD-13, RD-23); `n` is the episode counter, `k` the delegation instance within episode `n` | Horizon at Hand off, using the case keeper's `n` and `k`; SEA-Forge validates | session, event context, delegation |
| correlation | structured `{case_id, assignment_id?, episode_ref?, run_id?}` (RD-18) | emitter | every event; never parsed from a string |
| RealityTrace ids | existing ULIDs and `question:`, `threshold:`, `observation:`, `evidence:`, `settlement:`, `residual:` | RealityTrace | Horizon, bundle |
| `decision_id` | SEA-Forge CEP decision, scoped by `operation_id` | SEA-Forge | events, approvals; verified at ingest |
| cross-system record key | `(system, record_id)` (RD-12) | owning system | idempotency key of every cross-system write |
| Principal | Cognate identity contract | Cognate identity | every authenticated request |

**AD-1.** Identifier changes: the `assignment_id` instance suffix and the `episode_ref` composition only. Hashing is
SHA-256 over RFC 8785 canonical JSON, stated in CEP-0008. No frozen specification is edited. Each interface or
schema change gets a spec and ADR update in the same change; status goes through `CURRENT_STATUS.yaml` and `just
status-check`; ask-first items go to the owner (H §4, OD-3).

## 3. The Field Case in Casework (AD-2)

A Field Case is a Casework case with template `fde.field_case@1` plus **one new case-scoped document type**. The
persisted plan schema and plan hash do not change (RD-09).

| SOP concept | Casework construct (existing unless NEW) |
|---|---|
| Field Case (`SOP §1.1`) | `CasePlan` with `template_ref = "fde.field_case@1"` |
| Content groups (`SOP §4.2`) | NEW `cases/<id>/field_case.json`, schema generated from the model, revised by `field_case.revised`; each field has `status ∈ declared, observed, inferred, proposed, unknown` and a source ref |
| Activity (`SOP §6`) | discretionary `PlanItem`; repetition via the existing `repetition` marker (`max_instances > 1` is valid only with it, `case_engine.rs:246-248`). Activity identity lives in records, NOT in `PlanItem.markers` |
| Episode (`SOP §2.1`) | `episode.frozen {episode_ref, canonical reference, digest, kind ∈ evaluated, exploratory, executor_provenance, coverage manifest}`; `HorizonItem.run_ids` models repeats |
| Case Assignment | existing delegation (`agent_run.delegate`, `delegation.preview`); payload extended in WP-20 |
| Evidence gate (`SOP §8`) | `Milestone` item per gate; identity and state in `gate.assessed {gate, state, evidence_refs, rationale, assessor}`; hard-gate `not_applicable` needs A3 (§8) |
| Epistemic result | `episode.settled {episode_ref, result ∈ S,F,U,C, reliability, independence, settlement_ref}` |
| Disposition (`SOP §7.5`) | `disposition.recorded {level ∈ case, episode, disposition, rationale}` behind the existing approval path |
| Strategy (`SOP §4.4`) | `strategy.recorded`, revisable |
| Continuation (`SOP §10.1`) | read verb `case.continuation`, computed from the ledger plus RealityTrace and Cognate references; never stored |

**One write verb:** `case.record.append {kind, payload, idempotency_key}` (contract in WP-11; kinds and read verbs added in WP-20), validated against a per-kind schema
registry. Six kinds: `field_case.revised`, `strategy.recorded`, `episode.frozen`, `episode.settled`, `gate.assessed`,
`disposition.recorded`. Reads: `case.field.get`, `case.continuation`. Typed frames with golden fixtures in
`apps/godspeed-casework-go/internal/adapters/sfwp/testdata/frames.json`, mirrored in the Go client and generated
TypeScript. The template is authored once and a test fails if its operations or gates differ from the model's (no
generator exists; none is built).

**Invariants (RD-10).**
- **I-1 freeze before move.** The case keeper refuses a delegation or governed operation bound to an evaluated
  episode, checked on the execution identity (not a caller tag), unless that episode's freeze record and its
  RealityTrace plan both exist.
- **I-2 no self-settlement.** The settling principal's delegation chain (root operator, delegated agent, execution)
  must not contain the executor's execution or agent identity. A human root settles only via A3. Sibling agents
  under one root qualify only when the verifier's context is recorded as fresh.
- **I-3 frozen means frozen.** Key `(case_id, plan_item_id, n)`. A second freeze for the key is refused whatever its
  digest; changed criteria allocate `n+1`; the case keeper allocates `n`.

**Lifecycle** is a pure function over the ledger with golden tests, no enum change: `closed` only when the case state
is terminal and a case-level close disposition exists; `suspended` when the latest case-level disposition is
`suspend` (new freezes and delegations are refused, NT-22); otherwise `open`. A case cannot reach Completed without
a close disposition (NT-23). Episode-level outcomes never close a case.

## 4. The declared model (AD-3)

- Source `sop/GS-FDE-SOP-001-v0.4.1.md`, promoted to `sea-rs/docs/sop/` and hash-pinned (WP-13). Model
  `sea-rs/.sea/interaction/fde-sop/` per the IDM method (model, `source-translation-map.md`,
  `assumptions-and-unknowns.md`, `validation/`); world `godspeed-fde-sop`.
- Today's grammar, no change: `role_binding` for `SOP §3.1`; entities and resources; one `operation` per navigation
  step and activity (`intent`, `access policy_governed by … at precondition|postcondition`, `evidence
  operation_trace`); Obligation/Prohibition `policy` for invariants; `flow`; `metric`. Operation-bound policies are
  evaluated only by DomainForge `evaluate` (WP-12, §6); `validate` skips them.
- Each translation-map row carries `SOP §` and status `declared | inferred | unknown`.
- Validation: real `domainforge validate` and `envelope --emit cep`, recording source hash, `world_ref`, compiler
  version. `domainforge test` is not used as evidence.
- Check as input `sea-rs/.sea/interaction/validation/limitations.md` (L2 no list fields, L5 arithmetic in `where`
  passes silently, L9). A clause that depends on a recorded limitation is marked in the map and reported
  `insufficient` (`not_evaluable`), never `confirmed`.

## 5. Semantic exhaust (AD-4)

**Envelope.** The event rides the CEP `execution_trace` envelope Cognate's outbox already emits, in
`extensions["godspeed.interaction_event"]`. `agent-event-v1` is not used (a closed telemetry format,
`sxr-cli/src/agent_event.rs`). Profile and hash rule: WP-11.

| Field | Type | Req. | Meaning |
|---|---|---|---|
| `world_ref` | string | yes | model of the element |
| `semantic_ref` | `namespace::name` | yes | declared element addressed |
| `declaration_key` | `concept:<uuid>` | yes | derived and checked (§2) |
| `actor.principal`, `actor.role` | string | yes | authenticated principal; SOP role |
| `operation`, `resource` | string | yes | declared or observed operation; resource acted on |
| `context.case_id`, `context.plan_item_id` | string | yes | case anchor |
| `context.episode_ref` | string | evaluated: yes | frozen reference |
| `context.assignment_id` | string | if delegated | `assign:<caseId>:<stepId>:<n>:<k>` |
| `correlation` | object | yes | structured (§2) |
| `causation_id` | string | if caused | record id of the cause in the owning system |
| `authority.decision_id` | string \| `not_required` \| `unknown` | yes | verified against SEA-Forge |
| `expectation_ref` | string | evaluated: yes | RealityTrace question / threshold ids |
| `outcome.status`, `outcome.effect_refs` | enum `succeeded \| failed \| denied \| interrupted \| unknown` (RD-23); string[] | status yes | observed, never inferred |
| `evidence_refs`, `settlement_ref` | string[]; string | no | evidence ids; settlement events only |
| `source.system`, `source.record_id` | string | yes | pointer to the untouched source record |
| `source.sequence` | integer | yes | owning system's sequence (ordering) |
| `source.recorded_at` | timestamp | yes | emitter clock; informational only |
| `executor_provenance` | `program`, `live`, `injected_fault` | yes | RD-02 |
| `emitter.signature` | `{key_id, alg, sig}` | yes from WP-33 | workload-identity signature over the canonical extension; before WP-33 absent, and authenticity rests on source-record resolution plus the independent-emitter rule (RD-21) |

**Emitter trust (RD-05, RD-21).** From WP-33, RealityTrace verifies the signature, then resolves `source.record_id` against the owning
system's record; only then does the occurrence count. `decision_id` is verified against SEA-Forge. `confirmed` needs
at least one occurrence from an emitter independent of the executor (SEA-Forge authority or ledger record, the
verifier's own observation, Gauntlet / SWE_SEED); executor-only evidence caps at `insufficient` (NT-19, NT-20).
Ordering uses the owning system's `sequence` and causation, never outbox arrival or `recorded_at`. Until WP-33 (emitter signatures), authenticity is source-record resolution against the owning system plus the independent-emitter rule (NT-20, WP-17); S1 and S2 use SEA-Forge's existing identity gate with distinct actor ids and scope their claims accordingly. NT-19 (forged or unsigned event) belongs to WP-33 and S3.

**Emitters:** SEA-Forge ledger and authority decisions (WP-14; closes the missing `EvidenceRecorded` producer);
Cognate runs through the existing outbox, finishing `feat/realitytrace-real-snapshot` (WP-15); Workbench
`ObservationDelivery` (WP-23); Gauntlet / SWE_SEED verification records (WP-43).

**RealityTrace anchoring (RD-07).** The case workspace is an enrolled repo/checkout; each episode is a `run`; the
frozen reference is a DomainForge Semantic Verification Contract imported as `verification.contract_imported`, then
`plan.created`, `question.declared`, `threshold.declared`; observations hang off run and plan; residuals reference
question, snapshot and qualifying evidence. An event that cannot be anchored is `ingest.rejected`; there is no
"unanchored occurrence" record, and a rejected event supports no conclusion (M§7).

**Freeze order (RD-12).** (1) the case keeper writes the RealityTrace plan, question and threshold; (2) the case
record `episode.frozen` referencing them, with the coverage manifest (which emitters must report which event kinds),
frozen with the episode, not self-reported; (3) the external anchor. Delegation is refused unless all three exist.

**External anchor (RD-04).** At model registration, each freeze and pre-registration, the digest is published as a
signed annotated tag `fde-anchor/<kind>/<id>` pushed to the GitHub remote by the case-keeper service identity, never
an executor credential. The verifier checks the server-side push time and that the tag predates the first run
record, and issues its own receipt at freeze. **Limits (RD-26).** On a single-operator host, executors and the case keeper share an OS user, so the anchor proves "not rewritten after the server-recorded push time", not "could not have been written by an executor". A dedicated anchor credential with a protected tag namespace `fde-anchor/*` (no delete, no force) is requested from the owner (OD-6); without it, anchoring rows are PARTIAL with this limit stated. Verification needs network access; offline it verifies the recorded verifier receipt only and reports PARTIAL. NT-16 backdated freeze / regenerated chain; NT-17 reference derived
from the result; NT-18 model registered after execution.

## 6. Observed representation and comparison (AD-5)

New RealityTrace crate `sxr-observed` (ask-first; `sxr-core` stays I/O-free) and CLI group `realitytrace observed`.

- `observed derive --world <world_ref> --scope case:<id> --episode <episode_ref> --kind evaluated|exploratory
  [--until <record>]`: from verified records only; nodes per `declaration_key` plus `undeclared:<sha256 of
  operation+resource kind>`; directly-follows edges by owning-system sequence and causation; per node/edge
  occurrence ids, frequency, ordering statistics, role/context distribution, authority outcomes, coverage against
  the frozen manifest, uncertainty. Deterministic canonical JSON stored as `representation.recorded` with the input
  record-set hash (WP-16).
- `observed compare --declared <snapshot_id> --observed <representation_id> --profile <id> [--mapping <file>]`
  emits `residual.computed` per difference, with `comparison ∈ episode | structural`; the kinds are never mixed in
  one list (WP-17). Without a required key mapping (§7) it refuses to run.
- Classes (Mission fixes five): `confirmed`, `contradicted`, `observed_undeclared`, `declared_unobserved`,
  `insufficient`. `not_evaluable` is a basis within `insufficient`, not a class.
- `declared_unobserved` only for obligations whose trigger was observed inside the frozen coverage window and whose
  required record is absent (RD-03). Untriggered declared elements are reported `not exercised`; that is not a class.

**Prerequisite (WP-12).** `domainforge evaluate --model <file> --facts <file> --format json` returns per policy and
binding `true | false | unknown` and the bound instances, including operation-bound policies. The facts-file schema is owned by WP-12 and published through WP-11. `validate` cannot
serve: it emits violations only, separates unknown from false only by message text, and skips operation-bound
policies. Spike WP-10 first, on a ten-element model against the DomainForge library, covering an empty fact set, one
occurrence, an obligation that evaluates to unknown, and an operation-bound policy.

| Expectation kind | Decided by | Mapping |
|---|---|---|
| policy-evaluable | DomainForge `evaluate` over a generated, hashed, never hand-edited facts file | true→`confirmed`, false→`contradicted`, unknown→`insufficient` |
| structural (declared flows, operation pre/postconditions, triggered obligations) | `sxr-observed` over declared relations and observed occurrences | same classes; undeclared behaviour→`observed_undeclared` |
| neither can decide | none | `insufficient`, basis `not_evaluable` |

**Comparison profile and support.** The profile (WP-11) fixes per comparison kind the quantifier and a minimum
support `k`; each result records `support_count`. No vacuous `confirmed`: a universal rule over zero instances, or
fewer than `k` supporting occurrences, is `insufficient` (basis `below_minimum_support`).

| Cannot be classified by policies | Handled by |
|---|---|
| ordering | edge comparison in `sxr-observed` over sequence and causation |
| frequency | profile threshold on counts; below `k` is `insufficient` |
| absence | `declared_unobserved` rule (trigger observed in the window, record absent) |
| timing | profile deadline over source sequence/causation; emitter clocks never decide |
| context variation | reported in the representation; attributed only by diagnosis (`SOP §9.2`), not classified |

Diagnosis and proposals reuse `diagnostic.hypothesis`, `diagnostic.settled`, `proposal.raised` (which requires a
DomainForge semantic diff).

## 7. Governed structural revision (AD-6)

1. `proposal.raised` references residuals and diagnosis, freezes the held-out rule, lists intended changes.
2. Candidate `.sea` on a branch with a `ConceptChange`; DomainForge validates and produces the semantic diff;
   RealityTrace records `semantic.snapshot` and `semantic.diff`.
3. The regression matrix (below) comes from re-running `observed compare` under the candidate and is part of the
   promotion request.
4. SEA-Forge approval-gated world transition; approver differs from proposer; human-reserved approval is A3 (§8).
5. Pin / promote / re-pin (WP-18 minimal; WP-22 full). 6. A subsequent episode is frozen under the new `world_ref`.

**Transition kinds** follow CEP: `semantic_change`, `source_edit_only`, `compiler_upgrade`, `migration`; only
`semantic_change` is a structural revision (floored to `escalate`). In-flight cases keep their pinned world.
**Key mapping:** `declaration_key` is stable while `namespace::name` is unchanged; a rename needs an explicit mapping
from the semantic diff.

**Write path.** SEA-Forge has the transition but no pin/promote path (the registry is built from host config,
`server/src/cep_authority.rs:40-100`; a transition needs both worlds registered). New: a durable pin record and
`world.promote` / `world.repin`, as a saga keyed `(proposal_id, candidate_world_ref)` (register candidate, approve,
pin, record) finished or rolled back by the reconciler (§11). Every classification is stamped with its world, so a
re-pin leaves no stale `confirmed`. **Held-out set:** occurrences recorded before the proposal, excluded from the
diagnosis by the frozen rule.

| Old → New class under the candidate | Result |
|---|---|
| `confirmed` → `contradicted` | fails unless the element is in the proposal's intended changes |
| `confirmed` → `insufficient` or `observed_undeclared` | fails (lost capability) |
| `contradicted` → `confirmed` by weakening or removing a policy | escalated approval class; listed as a weakening |
| `observed_undeclared` → `confirmed` | intended gain |
| any class → `insufficient` (other than the rows above) | fails (lost evidence) unless coverage is unchanged and the element is listed as intended |
| `insufficient` → any other class | passes and is reported |
| `confirmed` → `declared_unobserved` | fails |
| `observed_undeclared` → `contradicted` | passes only if listed as intended (it declares the behaviour a violation) |
| all other transitions, including same class before and after | pass (RD-25) |

This matrix is total (RD-25); other documents cite it instead of repeating rows.

NT-21: lost capability and unlisted weakening each refuse promotion.

**Capability-delta proof (AT-13).** The same subsequent-episode records are compared under the old and new world,
with a control variant lacking the newly declared step. A falsifier is frozen beforehand; "no gain" is reportable.

## 8. Identity and authority (AD-7)

Adopt the owner-selected design in `Horizon-gaps/.agents/reports/2026-11-08-unified-human-agent-identity-authority.md`
(profile S1, classes P0–P3, assurance A3). Draft 1 was wrong that a passkey prompt on the desk closes the
browser-agent hole: under S1 the page may be agent-controlled, so an A2 ceremony there proves authenticator
participation, not review (RD-06). Human-reserved confirmation happens only on an **A3 surface** that receives the
canonical operation from SEA-Forge by `pending_id` and is not rendered by an agent-controllable page: the paired
phone (existing mobile remote). SEA-Forge, not Horizon, validates the A3 evidence (approval extension in stage 2).
The desk shows "confirm on your phone". The phone surface is class **A3-UV**; the protected-terminal class **A3-T** is
kept distinct, is usable only where policy explicitly accepts it, and is never accepted for a P3 operation (RD-21). With no A3 channel, reserved decisions are refused and the product says
which and how to restore the channel.

| Operation | Class | Required evidence |
|---|---|---|
| read, navigate, draft, note, exploratory observation | P0/P1 | session |
| freeze an episode reference; assign within an existing grant | P1 | authenticated principal; case-keeper preview check code |
| settle an episode (S/F/U/C) | P2 human-reserved, or independent verifier principal | verifier ≠ executor (I-2); A3 when a human settles |
| case-level disposition; set a hard gate not-applicable | P2/P3 human-reserved | A3 + SEA-Forge approval |
| promote a world; bootstrap or recover a controller | P3 | A3 + online SEA-Forge decision bound to operation digest and expiry |

The Chromium virtual authenticator is valid for automated regression only; a human-reserved decision counted as
PROVEN needs a real authenticator (OD-1, OD-5). NT-24: a reserved decision confirmed on the desk page, even with a
passkey assertion, is refused.

**S1 limits.** S1 defends against an agent controlling an authenticated browser or page. It does not resist a
hostile local process running as the operator (that is S2), a compromised issuer or authority host, root
compromise, a cloned authenticator, or a human approving the wrong thing. Horizon's open critic item 4 is closed
only for the reserved operations above; ordinary P0/P1 desk work stays session-based.

| Stage | Owner | State | Remaining | WP |
|---|---|---|---|---|
| 0B containment | Horizon | built on `horizon-desk-gaps`; enrolment proven against a test double | D3 route scope, D5 saga and reconciliation, real P3 authority client, first-controller bootstrap, A3 confirmation on the phone, PR split | WP-32 |
| 1 shared identity | Cognate | `packages/identity` on `horizon/shared-identity`, unconsumed | rename `@godspeedai/cognate-identity`; D2 evidence-verified issuance; scope claims; operator recording | WP-30 |
| 2 authority bridge | SEA-Forge | not started (`feat/horizon-desk-support` adjacent, PR #91) | D1 enrolment decision API; D4/D6 controller classes; reserved-operation check validating A3 | WP-31 |
| emitter identity | Cognate, SEA-Forge, RealityTrace | none | signatures, verification, independent-emitter rule | WP-33 |
| 3 Gauntlet binding, 4 migration | Gauntlet, Workbench | not started | binding record; Workbench consumes the identity package | WP-34 |
| 5 qualification | all | not started | real-boundary tests; skipped prerequisite = unqualified | S3 gate with WP-52 |

**Staged enforcement (RD-21).** The reserved-operation A3 check arrives with WP-31 and is off before it. In S1 and S2,
S1/S2 approvals use SEA-Forge's existing approval path with distinct actor identities and are recorded with assurance
`session`; every row that depends on a human-reserved decision is PARTIAL until S3. Principals are SEA-Forge actor
identities under its existing gate. I-1 and I-2 use SEA-Forge actor and delegation identities in S1-S2 and are
re-proven on the shared identity contract in S3; WP-21 does not depend on WP-30.

## 9. Projections and Horizon

Read adapters (WP-19): `field-case.ts` (`case.field.get`, `case.continuation`) and `realitytrace-structure.ts` (CLI
JSON for `observed` and residuals), feeding the generic `DebuggerSources` (`src/domain/debugger.ts:180-197`).

| Question (M§6) | Operation | Today | Extension to build (WP-40) |
|---|---|---|---|
| why classified so | `why` | lineage walk | source: residual record (rule, occurrences, element) |
| declared v_n vs v_n+1 | `diff` | checkpoint object diff | source: DomainForge semantic diff; never rewrites the frame's checkpoint |
| history, provenance | `timeline` | ordered records | join ledger with RealityTrace records |
| revision consequences | `counterfactual` | edge walk, `settles:false` (`:540-552`) | source: regression-matrix result; never executes |
| move available, what blocks | `reach` | five-rung capability ladder (`REACH_RUNGS`, `:591`) | availability over the seven Aim dimensions: new, a schema revision of `horizon.world-debugger/1` |
| re-derive a settlement | `replay` | equality of one field to one value (`:632-645`) | re-derivation of representation and comparison: new |

Parity: each answer embeds the record ids and digests it was built from; the test compares the ledger result with
the rendered DOM text, not only adapter JSON (RD-17), and with CLI output for the same ids (M§7).

## 10. Horizon and Workbench boundary (AD-8)

- **Hand off is the one door** (RD-13): composer, proposed step, preview, freeze the expectation, assignment; the
  brief is prefilled from the freeze.
- Horizon → Workbench: Connect RPC `assign`, today `{id, requestId, expertId, workspaceId, message, reviews}`
  (`workshop.ts:170-176`); the payload gains `episode_ref`, the `SOP §7.4` brief and budget. `assignment_id` is
  `assign:<caseId>:<stepId>:<n>:<k>`, so repeated instances of a step no longer collide. The session carries the
  structured correlation.
- Workbench → Horizon: deep link `#session/<id>[/<eventId>]` on the assignment; a read-only case strip from
  SEA-Forge; "back to case".
- No P2/P3 confirmation in Workbench or on the desk page: both show "awaiting decision" and point to the A3 surface.
- One operator identity signs in to both; design systems stay separate. Packages WP-23, WP-42.

## 11. Recovery

The case keeper is authoritative for state; a reconciler (WP-22) detects orphan RealityTrace records and incomplete
freezes. Idempotency key for cross-system writes is `(system, record_id)`.

| Failure | Behaviour |
|---|---|
| RealityTrace unavailable | emitters queue in the durable outbox; episode cannot settle; result `U` |
| SEA-Forge unavailable or decision unknown | governed operations refuse (fail closed); no promotion |
| Interrupted assignment | Cognate continuation resumes; interruption recorded; budget and evidence kept |
| Partial freeze | delegation refused (I-1); reconciler completes from the plan if the digest matches, else marks it abandoned and the next freeze allocates `n+1`; nothing deleted |
| Orphan RealityTrace records | reported, never counted; removed only by an authorized abandon record |
| Outbox duplicates | dropped on `(system, record_id)`; a different body for the same key is `ingest.rejected` |
| Chain loss or regeneration | the anchor tag no longer matches; verification fails (NT-16); affected episodes void, re-frozen as `n+1` |
| Crash during enrolment | saga state plus reconciliation (D5); uncertain outcomes fail closed |
| Partial promotion | reconciler completes or rolls back by `(proposal_id, candidate_world_ref)`; registry unchanged until the pin record exists |
| Bad promotion | re-pin the previous `world_ref` through the gated transition; classifications are world-stamped |
| Horizon projection drift | rebuild; `last_event_id` mismatch forces refetch |

## 12. Integrated release gate (AD-9)

FDE gates are added to the existing `godspeed-readiness` `local_operator` profile, which must stay GO; no forked
profile (RD-18, WP-57): pinned heads from origin refs for every repo including DomainForge; per-repo gates;
shared-schema identity gates; the Field Case proof; the negative suite; the operate package (WP-55: topology,
configuration and ports, migration of existing cases and ledgers, outbox depth / rejection rate / projection lag,
retention and backup of the new stores, added to the existing backup/restore and upgrade gates); clean-cell
install from release artifacts. A skipped gate is NO-GO.

## 13. Contracts to be specified in WP-11 and WP-20

Schemas with types, request/response/error tables and fixtures, reviewed before code. Paths are proposed.

| Contract | WP | Owner | Schema path | Minimum fields decided |
|---|---|---|---|---|
| Interaction-event extension | WP-11 | CEP | `cep/schemas/extensions/godspeed.interaction_event.v1.json` | §5 table; `outcome.status ∈ succeeded, failed, denied, interrupted, unknown` |
| CEP-0008 hash rule | WP-11 | CEP | `cep/CEP-0008` amendment | SHA-256 over RFC 8785 JSON |
| Frozen reference (verification contract) | WP-11 | DomainForge schema; SEA-Forge writes | `sea-rs/contracts/fde/frozen-reference.v1.json` | `episode_ref`, intention, expected change, invariants, predicted irrelevancies, failure/success regions, falsifiers, evidence source, decision-maker, coverage manifest, `executor_provenance`, kind |
| Comparison profile | WP-11 | RealityTrace | `sxr/contracts/comparison-profile.v1.json` | per kind: quantifier, `k`, deadline, class mapping |
| Observed representation | WP-11 | RealityTrace | `sxr/contracts/observed-representation.v1.json` | nodes, edges, occurrence ids, frequency, ordering, distributions, authority outcomes, coverage, uncertainty, input-set hash, `world_ref` |
| Residual | WP-11 | RealityTrace | `sxr/contracts/residual.v1.json` | class, basis, `comparison`, keys, occurrence ids, rule, `support_count`, question, snapshot, world stamp |
| Anchor tag | WP-11 | SEA-Forge | `sea-rs/contracts/fde/anchor.v1.json` | tag `fde-anchor/<kind>/<id>`, digest, pusher identity, verifier receipt |
| `evaluate` output | WP-12 | DomainForge | `domainforge/schemas/evaluate-output.v1.json` | per policy and binding: tristate, bound instances, operation-bound flag |
| `evaluate` facts file | WP-12 | DomainForge | `domainforge/schemas/evaluate-facts.v1.json` | schema owned by WP-12, published through WP-11; generated, hashed, never hand-edited |
| `case.record.append` | WP-11 | SEA-Forge | `sea-rs/contracts/fde/case-record-append.v1.json` | `{kind, payload, idempotency_key}`, request/response/error tables; WP-20 adds the kinds and read verbs |
| Field-case document | WP-20 | SEA-Forge | `sea-rs/contracts/fde/field-case.v1.json` | §3 table; per-field status and source ref |
| Record-kind registry | WP-20 | SEA-Forge | `sea-rs/contracts/fde/case-record-kinds.v1.json` | six kinds, payload schemas, idempotency key, refusal codes |
| SFWP frames (`append`, `field.get`, `continuation`) | WP-20 | SEA-Forge | `sfwp/testdata/frames.json` path above | `field.get`, `continuation` read verbs and the six kinds on the WP-11 `append` contract |
| Continuation | WP-20 | SEA-Forge | `sea-rs/contracts/fde/continuation.v1.json` | `SOP §10.1` items |
| Reserved-operation interface | WP-20 | SEA-Forge | `sea-rs/contracts/fde/reserved-operation.v1.json` | operation, class, evidence (§8), digest, expiry, `pending_id`; A3 proof minimum: SEA-Forge challenge, operation digest, authenticator assertion, confirmation-surface id, expiry |
| Assignment payload | WP-20 | SEA-Forge, Horizon | `sea-rs/contracts/fde/assignment.v1.json` | `assignment_id`, `episode_ref`, brief, budget; id `assign:<caseId>:<stepId>:<n>:<k>` |

## 14. Decisions made while revising

- Contract file names and paths in §13 are proposed (extends RD-18).
- `disposition.recorded` carries `level ∈ case, episode` (extends RD-10, RD-15).
- Basis `below_minimum_support` added under `insufficient` (extends RD-08).
- S3 qualification attaches to the S3 gate with WP-52; no WP exists for it (extends RD-01).
- `executor_provenance` and `kind` are carried on events and `episode.frozen` (extends RD-02, RD-03).
- An abandoned partial freeze allocates `n+1`; nothing is deleted (extends RD-10, RD-12).
- `source.sequence` is the ordering field of the extension (extends RD-05).
