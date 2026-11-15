# H. EGM review and decision record

Review round 1, 2026-11-12/13. Six Goldfish reviewers (Sonnet, fresh contexts, no access to the design
discussion) reviewed draft 1 of deliverables A–G and SOP v0.4. Reports: `egm-review/01`–`06`. Lenses used:
fidelity and evidence, design and governance, codebase feasibility, implementation readiness, zoom-out,
comprehension and operator experience. Not run: structural and style lenses (low yield for this decision).

Totals: 10 BLOCKER, 48 MAJOR, 11 MINOR. All BLOCKER and MAJOR findings are resolved below as revision
decisions (`RD-n`). Draft 2 of A–G and SOP v0.4.1 implement these decisions; where a draft-1 statement
conflicts with an `RD`, the `RD` governs.

Finding IDs: `F*` = 01 fidelity, `G-*` = 02 design, `C-*` = 03 codebase, `IR-*` = 04 readiness, `ZO-*` = 05
zoom-out, `UX-*` = 06 comprehension (its `F-01…F-10` renamed `UX-01…UX-10` here to avoid collision).

## 1. Revision decisions

### RD-01 Proof strategy: thin slices, not one big run
*Resolves ZO-01, ZO-02, ZO-09, IR-09, IR-10.* The plan is restructured into five slices, each ending in a
runnable gate:
- **S1 walking skeleton.** Smallest path that exercises Mission §1–7 once: model, events, derive, compare,
  existing approval-gated world transition, subsequent episode, Horizon read parity, one script, run on a
  clean cell. Deterministic program executor. No new UI beyond read adapters.
- **S2 Field Case records.** Full SOP model, case records, gates, dispositions, continuation, promotion write
  path and saga.
- **S3 identity and authority.** D1–D6, A3 confirmation, emitter authentication hardening.
- **S4 operator experience.** Journeys, Workbench seam, verification bridges.
- **S5 proof, transfer, operate, release.**
The first Field Case's work items are **not** on the plan's own critical path (RD-02).

### RD-02 First Field Case content and executors
*Resolves IR-02, IR-10, ZO-01 (substance), F2.* The owner requires the first case to run the SOP on real
GodSpeed work, so Refund Desk is not substituted. FC-001 is "Close recorded SEA-Forge and DomainForge debt
items": small, real, verifiable by a deterministic command, and off the critical path. Candidate episodes
(final selection is pre-registered in WP-50 after confirming each item still exists on the integration branch):
derive `ReadinessView.overall = stale` (SEA-Forge observed debt); `events.get_range` tail read; an inquiry
episode with frozen hypothesis "DomainForge KG round trip preserves Policy and EntityInstance triples"
(DomainForge debt D-doc-egm-3 suggests it will be falsified); Horizon request ids for route/reassign/withdraw
(gap A).
Executors are of three provenances, recorded on every occurrence: `program` (a deterministic executor that
applies a hash-pinned patch and runs the declared command through the real SEA-Forge / Cognate / RealityTrace
path; allowed by SOP §3.1 "execution actors may be humans, agents or deterministic programs"), `live` (an agent
or person doing the work undirected), `injected_fault` (negative tests only).

### RD-03 Behaviour kinds come from execution, not directives
*Resolves F2, F7, F8, IR-02.* No episode is instructed to misbehave. Pre-registration records a *prediction* of
the class each episode will yield; a wrong prediction is reported, never edited.
- Conforming: a `program` episode that follows the procedure.
- Contradicted expectation: the falsifiable inquiry episode (episode result **F** on a frozen hypothesis), and at
  structural level any procedure obligation whose trigger occurred and whose required record contradicts it.
- Legitimate undeclared: fresh-context review before claim, which is this programme's standing practice and not
  declared in the SOP as an operation.
- Missing observation: a real instrumentation gap named in the frozen reference as an evidence source (for
  example token cost for `program` executors). The episode's *result* depends on what the frozen reference says
  that evidence was for; payment that is merely unmeasured is `unknown` and does not by itself make the result U.
- Declared-unobserved: only for obligations whose trigger was observed inside the frozen coverage window and
  whose required record is absent. Untriggered declared elements are reported as `not exercised` and are not
  a divergence class.
- `injected_fault` records exist only in the negative suite and never support an execution claim. If a kind
  does not occur through `program` or `live` execution, its row is NOT TESTED.
- AT-18 "regenerate" means: verify and replay the immutable raw records to recompute representation and
  comparison, **and** re-execute the `program` episodes on a clean cell, comparing class-level results.

### RD-04 External anchoring of "before"
*Resolves F1.* At model registration, at each freeze, and at pre-registration, the digest is published as a
signed annotated tag `fde-anchor/<kind>/<id>` pushed to the GitHub remote by the case-keeper service identity,
not by any executor credential. The verifier checks the server-side push time and that the tag object predates
the first run record, and also issues its own receipt at freeze. New negative tests: backdated freeze
(regenerated chain), reference derived from the result, model registered after execution.

### RD-05 Emitter authentication and independent corroboration
*Resolves G-03, C-12 (ordering).* Every interaction event is signed by its emitter's workload identity.
RealityTrace verifies the signature, then resolves `source.record_id` against the owning system's record before
the occurrence may count. A `confirmed` classification requires at least one occurrence from an emitter
independent of the executor (SEA-Forge authority or ledger record, the verifier's own observation, or
Gauntlet/SWE_SEED). `authority.decision_id` is verified against SEA-Forge. Ordering uses the owning system's
recorded sequence and causation links, never outbox arrival order. Until S3 lands, S1 uses SEA-Forge's existing
identity gate and distinct actor ids, and its claims are scoped accordingly.

### RD-06 Human-reserved decisions: A3 on a separate surface
*Resolves G-01, G-02, IR-01 (authenticator), ZO-09.* Draft 1 was wrong to say a passkey prompt on the desk
closes the browser-agent hole. Per the owner-selected identity design (profile S1): confirmation happens only
on an A3 surface that receives the canonical operation from SEA-Forge and is not rendered by an
agent-controllable page: the paired phone (existing mobile remote). SEA-Forge, not Horizon, validates the A3
evidence (approval extension added to stage 2). The desk shows "confirm on your phone".

| Operation | Class | Required evidence |
|---|---|---|
| read, navigate, draft, note, exploratory observation | P0/P1 | session |
| freeze an episode reference; assign within an existing grant | P1 | authenticated principal; case-keeper preview check code |
| settle an episode (S/F/U/C) | P2 human-reserved, or independent verifier principal | verifier ≠ executor (I-2); A3 when a human settles |
| case-level disposition; set a hard gate not-applicable | P2/P3 human-reserved | A3 + SEA-Forge approval |
| promote a world; bootstrap or recover a controller | P3 | A3 + online SEA-Forge decision bound to operation digest and expiry |

The Chromium virtual authenticator is valid for automated regression only; a human-reserved decision counted as
PROVEN in the proof needs a real authenticator. See OD-1.

### RD-07 Ingest path and RealityTrace anchoring
*Resolves C-01, C-03, IR-05 (question/threshold writer).* The interaction event rides the **CEP
`execution_trace` envelope** already emitted by Cognate's outbox, with `extensions["godspeed.interaction_event"]`.
`agent-event-v1` is not used. Events carry both `world_ref` and the semantic reference `namespace::name`; the
`declaration_key` is derived by DomainForge's `ConceptId` rule and checked. RealityTrace's record rules are
honoured: the case workspace is an enrolled repo/checkout; each episode is a `run`; the frozen reference is a
DomainForge Semantic Verification Contract imported as `verification.contract_imported`, then `plan.created`,
`question.declared`, `threshold.declared`; observations hang off the run and plan; residuals reference question,
snapshot and qualifying evidence. An event that cannot be anchored is `ingest.rejected`; there is no "unanchored
occurrence" record. The case keeper performs the freeze writes (RD-12).

### RD-08 Classification needs a DomainForge change; no second evaluator
*Resolves C-02, G-04, G-12, IR-08, ZO-07.* Verified: `domainforge validate` emits violations only, cannot
distinguish unknown from false except by message text, and skips operation-bound policies. Decision: DomainForge
gains `domainforge evaluate --model <file> --facts <file> --format json` returning, per policy and binding,
`true | false | unknown`, the bound instances, and including operation-bound policies. This is a prerequisite
work package owned by DomainForge. The fallback of evaluating policies inside RealityTrace is **withdrawn**
(it would create a second owner of policy semantics). The spike runs first, against a ten-element model and the
DomainForge library directly, and must cover: empty fact set, one occurrence, an obligation that evaluates to
unknown, and an operation-bound policy. Classification is split three ways:
- policy-evaluable expectations: DomainForge result maps true→`confirmed`, false→`contradicted`, unknown→`insufficient`;
- structural expectations (declared flows, operation pre/postconditions, triggered obligations): evaluated by
  `sxr-observed` over the declared graph's relations and the observed occurrences;
- anything neither can decide: `insufficient` with basis `not_evaluable`.
A **comparison profile** (minimum support count, coverage window, emitters expected, which elements are in
scope) is frozen with the episode; `confirmed` on a single occurrence or by vacuous truth is not allowed unless
the profile says so.

### RD-09 Casework changes: one append verb, no plan-schema change
*Resolves ZO-03, C-04, C-05, G-09, IR-03 (partly), C-10.* Instead of eight verbs: `case.record.append
{kind, payload, idempotency_key}` validated against a per-kind schema registry, plus read verbs `case.field.get`
and `case.continuation`. Kinds: `field_case.revised`, `strategy.recorded`, `episode.frozen`, `episode.settled`,
`gate.assessed`, `disposition.recorded`. Activity tags and gate identity live in these records, not in
`PlanItem.markers`, so the persisted plan schema and plan hash do not change. Repetition uses the existing
`repetition` marker. The Field Case document is honestly described as **one new case-scoped document type**,
its schema generated from the model. The template is authored once and checked against the model by a test
(no model-to-template generator exists; none is built). Repository rules are followed: each interface or schema
change gets a spec and ADR update, and status goes through `CURRENT_STATUS.yaml` and `just status-check`. The
changes this plan makes to protected interfaces are listed in §4 for the owner (OD-3).

### RD-10 Invariants and lifecycle, restated so they are enforceable
*Resolves G-10, G-09, UX-05, F9 (mapping).*
- **I-1 freeze before move:** enforced by the case keeper on the *execution identity*: a delegation or governed
  operation bound to an evaluated episode is refused unless the episode's freeze record and its RealityTrace
  plan both exist. Not a caller-supplied tag.
- **I-2 no self-settlement:** the settling principal's delegation chain (root operator → delegated agent →
  execution) must not contain the executor's execution or agent identity, and a human root may settle only via
  A3. Sibling agents under the same root are allowed only when the verifier's context is recorded as fresh.
- **I-3 frozen means frozen:** keyed on `(case_id, plan_item_id, n)`. A second freeze for the same key is
  refused whatever its digest; changed criteria allocate `n+1`. The case keeper allocates `n`.
- **Lifecycle** is a pure function with golden tests: `closed` only when the case state is terminal **and** a
  case-level close disposition is recorded; `suspended` when the latest case-level disposition is `suspend`
  (while suspended the case keeper refuses new freezes and delegations); otherwise `open`. A field case cannot
  be advanced to Completed without a close disposition. No enum change. Episode-level outcomes never close a case.

### RD-11 World revision made precise
*Resolves G-06, G-07, G-08, C-06, F5.*
- Transition kinds follow CEP: `semantic_change`, `source_edit_only`, `compiler_upgrade`, `migration`. Only
  `semantic_change` is a structural revision. In-flight cases keep their pinned world.
- `declaration_key` is stable across versions while `namespace::name` is unchanged; renames require an explicit
  key mapping taken from the semantic diff, and the comparison refuses to run without it.
- Verified: SEA-Forge has the approval-gated transition but no promote/pin write path (the registry is built
  from host config). New: a durable pin record and promote / re-pin operations, with a saga keyed
  `(proposal_id, candidate_world_ref)` and reconciliation. Every classification is stamped with the world it was
  computed under, so a re-pin never leaves stale `confirmed` results.
- **Held-out set:** occurrences recorded before the proposal and excluded from the diagnosis by a rule frozen in
  the proposal.
- **Regression transition matrix** (old class → new class under the candidate): `confirmed→contradicted` fails
  unless that element is listed in the proposal's intended changes; `confirmed→insufficient` and
  `confirmed→observed_undeclared` fail (lost capability); `contradicted→confirmed` by weakening or removing a
  policy requires the escalated approval class and is listed as a weakening; `observed_undeclared→confirmed` is
  the intended gain. The matrix result is part of the promotion request.
- **Capability-delta proof (AT-13):** the same subsequent-episode records are compared under the old and the new
  world, with a control variant in which the newly declared step is absent. A falsifier is frozen beforehand,
  and "no gain" is a reportable outcome.

### RD-12 Split-brain and failure semantics
*Resolves G-11, G-05, UX-05.* Freeze is two-phase: RealityTrace plan/question/threshold first, then the case
record referencing them, then the external anchor; delegation is refused unless all exist. The case keeper is
authoritative for state; a reconciler detects orphan RealityTrace records and incomplete freezes. Idempotency
key for every cross-system write is `(system, record_id)`. Coverage is a manifest frozen with the episode
(which emitters must report which event kinds), not a self-report. `observed derive` takes `--episode` and
`--kind evaluated|exploratory` selectors; episode-comparison residuals and structural-comparison residuals are
distinguished by an explicit field and never mixed in one list.

### RD-13 Horizon: build on the approved v2 desk
*Resolves UX-01, UX-02, UX-06, UX-07, UX-08, UX-09, C-07, C-08, C-11.* Draft 1 misdescribed the desk. Draft 2:
- Keeps v2's layout: views Today, Explore, Decide, Work (cost inside Work), lists Days / Cases / Tags, status
  bar, inspector, composer, keys, attention rules, decision card, preview with check code, comparison view,
  notebook, agent panels. A kept / extended / unchanged table is included.
- Hand-off stays the one door: composer Hand off → proposed step → preview → freeze the expectation → assignment.
  The assignment brief is prefilled from the freeze.
- The status bar is extended (not replaced) with the open episode and its execution state.
- The decision card keeps v2's form; the four status axes appear as one line of words; detail lives in the
  inspector. Candidate moves are a list with the blocking dimension in words; the seven-dimension grid is an
  inspector detail.
- Adds a case timeline in Today, an agent-question card type, and an action in the compound journey.
- Debugger operations are described as they are today, and the work to extend them is stated as work: `reach`
  is a capability ladder (a move-availability answer is new), `counterfactual` is an edge walk (regression
  consequences are a new source), `replay` is an equality check (re-derivation is new), `diff` is an object diff
  (semantic diff is a new source).
- Assignment id becomes `assign:<caseId>:<stepId>:<n>` and the payload gains episode reference, brief and budget.

### RD-14 One vocabulary
*Resolves UX-03, UX-04.* A single glossary and crosswalk (operator word ↔ SOP term ↔ system term) is added to B
and referenced from the SOP. Fixed choices: the value-provenance tag is a **source mark** (v2 keeps "label" for
the operator's pass/fail judgement of a run); **blocked move** versus **gate not met**; **difference** is the
operator word, `residual` the record; **residue** is renamed **carry-over** in operator text; **episode** (case
keeper) / **run** (runtime) / **assignment** (delegation) / **session** (workshop) are defined once with their
1:n relations; **settle** (epistemic result) / **dispose** (authorized decision) / **close** (lifecycle);
"Rules edition" is the operator word for `world_ref`; **checkpoint** is defined; verifier, approver and
disposition authority are separate roles that one person may hold subject to RD-10.

### RD-15 SOP v0.4.1
*Resolves F9.* Restore from v0.2 verbatim: "a settled episode is not automatically a closed case or an earned
reusable capability"; the dogfood / not-a-customer-case sentence and the "separate claims" sentence in §11; the
RuVector / derived-indexes row; "real" and "actual" in the gate evidence; "meaningful variation" in Validate.
The disposition-to-lifecycle mapping applies to case-level dispositions only. Status line: "synthesis proposed
for owner ratification; frozen by hash as declared practice for FC-001" (OD-2).

### RD-16 Matrix and fidelity definition
*Resolves F3, F10, F11.* Split merged rows (the nine Mission §7 proofs become nine rows; R-16, R-41, R-43, R-62,
R-74 are split). Add rows for: baseline by Git remote and dirty-tree preservation; use of existing owners; no
dependence on unrelated Horizon features; Horizon artifacts in the bundle; repetition, variation and recovery
as separate evidence levels; SOP §1.5, §2.2, §3.3, §4.5, §6.2–6.3, §9.3. Every critical row has a negative
test. Additional rows marked critical: result classes, correlation/causation/authority on events, source-record
preservation, occurrence traceability, difference basis, separation of claim kinds, clean control. Status set
gains PLANNED. Fidelity is reported as absolute counts as well as a percentage; a not-applicable ruling needs
the owner's approval, not this record's.

### RD-17 Independence that means something
*Resolves F6, F4.* The independent verifier makes its own observation of each settled outcome (runs the
declared command itself) rather than re-reading the executor's evidence. Final confirmation uses a different
model CLI (the repository's existing independent-review route); if unavailable the claim is PARTIAL. Mutation
patches are hash-pinned files; the verifier re-applies them from a clean checkout. Every negative test asserts a
named refusal or rejection code, has no disjunctive pass condition, and separates tamper mode (chain verification
must fail) from input-variation mode (result must change). The Horizon parity test compares the ledger result
with the rendered DOM text, not only the adapter's JSON.

### RD-18 Plan mechanics
*Resolves IR-03, IR-04, IR-05, IR-06, IR-07, IR-11, IR-12, C-09, ZO-04, ZO-05, ZO-08.*
- **Contracts first.** A contracts package (schemas with types, request/response/error tables, fixtures) for
  every item IR-03 lists is the first deliverable of S1 and S2, reviewed before code. Emitters and validators
  are generated from the CEP schema; the correlation is a structured field, not a parsed string.
- **Branch safety.** Before touching any repo: backup tag on every branch head; push unpushed owner branches to
  origin as backups (Horizon `horizon-desk-gaps`, Workbench `feat/horizon-connect`); integration branches are
  built in new worktrees by merge or cherry-pick of *copies*; no rebase or rewrite of an owner branch; a branch
  whose commits are already on main by patch-id is recorded as landed and left alone; ties go to the owner. The
  37 Workbench PNG re-captures are committed to an evidence branch. Corrected facts: Horizon has nine branches
  with patch-unique commits (superset is unproven and must be checked per branch); SEA-Forge `t1/*` and the
  settlement-claim `-main` branch are already on main; `fix/server-ambiguous-declare-keeps-lease` is the
  pre-squash of merged #84; `feat/horizon-desk-support` is a clean fast-forward (PR #91); the DomainForge
  `println!` item is stale and dropped; `frames.json` lives under
  `apps/godspeed-casework-go/internal/adapters/sfwp/testdata/`.
- **One serial lane per repository;** parallelism only across repositories. The reserved-operation interface is
  defined in the S2 contracts package so authority work and case work do not collide.
- Every work package states a runnable acceptance command, an evidence path under one root, and a recovery
  procedure. Each S1–S3 package gets a file-level brief when it is dispatched.
- **Operate package:** launch topology, configuration and ports, migration of existing cases and ledgers,
  integration observability (outbox depth, rejection rate, projection lag), retention and backup of the new
  stores, added to the existing backup/restore and upgrade gates.
- **Readiness:** FDE gates are added to the existing `local_operator` profile, which must stay GO; no forked
  profile.
- An early clean-cell walking skeleton (S1) probes installation and the authenticator question before S3.

### RD-19 Scope kept, deferred, declined
*Resolves ZO-06, ZO-10, F12, UX-10.* Kept in the baseline but moved after the first proof: Gauntlet/SWE_SEED
evidence bridge, Context Kernel bundle on the brief, tax transfer (reported PARTIAL and fixture-based, with a
same-gap variant to show the learned distinction transfers, not only the mechanism). All ten journeys stay
specified; five are proof-critical (open/resume, aim and freeze, settle and dispose, investigate difference,
structural revision) and all five get the fresh-operator walk-through. The SEA-Forge consoles are documented as
administration-only with a "where do I go" table. Declined: substituting Refund Desk for the SOP case (owner
requirement); ZO-03's suggestion to drop gates and dispositions from the baseline (they are binding SOP rules).

## 2. First Field Case principals

| Role (SOP §3.1) | Principal in FC-001 | Identity |
|---|---|---|
| Sponsor, disposition authority, world-promotion approver | the owner | human operator; A3 on the paired phone |
| FDE / case lead | lead agent session | agent principal under the owner's delegation |
| Criterion author | lead agent, before execution; frozen and externally anchored | same, recorded at freeze |
| Executors | `program` executors; Sonnet implementer agents | distinct execution identities |
| Independent verifier | fresh-context agent on a different model CLI, making its own observation | separate agent principal, no shared context |
| Domain and policy custodian | the owner | human operator |

Before S3 ships, principals are SEA-Forge actor identities under its existing identity gate; rows that depend on
A3 stay PARTIAL until then.

## 3. Findings not adopted, with reasons

| Finding | Decision |
|---|---|
| ZO-01 use Refund Desk as the gating scenario | declined: the owner made the SOP case binding; its concerns are met by RD-01, RD-02, RD-03 |
| ZO-06 cut journeys to five | partly: five are proof-critical; the others stay because the operator capabilities in the brief need them |
| G-04 add a sixth class `not_evaluable` | adapted: Mission fixes five classes; `not_evaluable` is a basis within `insufficient` |
| IR-09 merge the negative suite into owner packages | adapted: each negative test is written with its owning package and collected into one suite run |

## 4. Protected-interface changes this plan makes (for the owner)

SEA-Forge: one SFWP verb family (`case.record.append`, `case.field.get`, `case.continuation`), six record kinds,
world pin/promote/re-pin operations, enrolment decision API and controller classes, reserved-operation check.
DomainForge: `evaluate` command. CEP: interaction-event extension profile, hash rule in CEP-0008.
RealityTrace: `sxr-observed` crate and `observed` commands, emitter signature check. Cognate: identity package
rename and evidence-verified issuance, event extension fields. Horizon: assignment id gains an instance number.
No identifier grammar is changed other than that suffix and the new `episode_ref` composition; no frozen
specification is edited.

## 5. Open owner decisions (work proceeds on the recommended option)

| ID | Decision | Recommendation | Effect if reversed |
|---|---|---|---|
| OD-1 | Does the S1 / A3-on-phone model supersede the open A/B/C choice in `2026-11-08-agent-authority-decision.md`? | Yes: it is the later, owner-selected design and is strictly stronger than option B without the terminal step of option A | S3 authority packages change shape; S1–S2 unaffected |
| OD-2 | Ratify SOP v0.4.1 as declared practice | Ratify | a text change creates v0.4.2, a new world and a re-run of affected episodes |
| OD-3 | Acknowledge the interface changes in §4 | Acknowledge | affected package pauses |
| OD-4 | Push unpushed owner branches to origin as backups before integration | Push | integration proceeds from local copies only |
| OD-5 | A real passkey on the owner's phone is needed for the human-reserved rows | Provide when S3 is ready | those rows stay PARTIAL |

## 6. Remaining consequential trade-offs

- Classification quality is bounded by what DomainForge policies can express; inexpressible SOP clauses are
  reported `insufficient`, which is honest but lowers apparent coverage.
- `program` executors make the proof reproducible but exercise less of the agent runtime than `live` work; both
  are run and reported separately.
- Two operator surfaces remain (desk and workshop). The seam is specified, the cognitive cost is real, and the
  walk-through measures it.
- One passing case proves the bounded mechanism only.

## 7. Proposed baseline (superseded by §8 for round-2 items)

Draft 2 of A–G plus SOP v0.4.1, as revised by this record, is the baseline for implementation. A second,
narrower Goldfish pass checks that each BLOCKER is closed in the revised text before S1 code starts.

## 8. Review round 2 (2026-11-13)

One fresh Goldfish reviewer (Sonnet) checked closure of the ten round-1 BLOCKERs in draft 2 and looked for
regressions: `egm-review/07-round2-closure.md`. Result by its closure table: 5 CLOSED (F1, C-01, C-02, IR-01,
IR-02), 5 PARTLY (F2, G-01, G-03, IR-03, ZO-01), 0 OPEN, plus 8 new MAJOR findings (R2-01…R2-08). (The
reviewer's summary line says 4 and 6; its table shows 5 and 5.)
The decisions below close them; draft 3 edits apply them. Where they conflict with RD-01…RD-19, these govern.

### RD-20 S1 is thin again and its claim is honest
*Resolves R2-02, R2-03, F2 (remainder), ZO-01 (remainder).* S1 runs three `program` episodes in one world: a
conforming control; a falsifiable inquiry whose frozen hypothesis may fail when run (its outcome is not known
at freeze but is deterministic on replay); and, after one revision, a subsequent episode. The revision candidate
in S1 is the executor harness's own fresh-context verification step, which really happens and is not declared
in model v1. S1's claim is: *the Mission §1–7 path exists and runs once on program execution.* It is not
evidence about organisational behaviour, and it says so in the claim ledger. In S1, AT-07 requires `confirmed`
and one of `contradicted` / `observed_undeclared` to arise; the other classes may be NOT TESTED without failing
the gate. The five behaviour kinds on `live` execution are an S5 requirement. S1 gate tests: AT-01, 04, 05, 06,
07, 08, 12, 13, 14, 18 and NT-01…NT-09, NT-16; every other negative test runs in the slice of its owning
package. Horizon gap A is removed from the FC-001 candidates because WP-32 builds it.

### RD-21 Staged enforcement before S3
*Resolves G-01 (remainder), R2-04.* The reserved-operation A3 check arrives with WP-31 and is off before it.
In S1 and S2, approvals use SEA-Forge's existing approval path with distinct actor identities, are recorded
with assurance `session`, and every row that depends on a human-reserved decision is PARTIAL until S3. The
phone surface is the A3-UV class; the protected-terminal class A3-T is kept distinct, is usable only where
policy explicitly accepts it, and is never accepted for a P3 operation. Emitter signatures arrive with WP-33;
before that, authenticity rests on source-record resolution against the owning system and the
independent-emitter rule (NT-20, owned by WP-17). NT-19 (forged or unsigned event) belongs to WP-33 and S3.
I-1 and I-2 use SEA-Forge actor and delegation identities in S1 and S2 and are re-proven on the shared identity
contract in S3; WP-21 does not depend on WP-30.

### RD-22 `declaration_key`
*Resolves R2-06, G-03 fix (d).* Verified in `domainforge-core/src/concept_id.rs`: the key is UUIDv5 of
`namespace::name` only, so it is stable across world versions (which RD-11 relies on). `world_ref` is carried
beside it, never mixed into it. RealityTrace resolves the event's `namespace::name` against the pinned world's
semantic snapshot and rejects an event whose element does not exist there; the executor's own claim about the
key is never trusted.

### RD-23 Contract fixes
*Resolves IR-03 (remainder), R2-05.* The `episode_ref` digest is computed over the frozen-reference body, which
contains `case_id`, `plan_item_id`, the episode counter `n`, `world_ref`, criteria and evidence sources, and
does not contain `episode_ref`. The `case.record.append` contract is specified in WP-11; WP-20 adds kinds and
read verbs. `outcome.status ∈ succeeded | failed | denied | interrupted | unknown`. The A3 proof (WP-20)
carries at least: SEA-Forge challenge, operation digest, authenticator assertion, confirmation-surface id,
expiry. The DomainForge `evaluate` facts-file schema is owned by WP-12 and published through WP-11. Assignment
ids carry both counters: `assign:<caseId>:<stepId>:<n>:<k>`, where `k` is the delegation instance within
episode `n`.

### RD-24 Lanes and bootstrap scripts
*Resolves R2-01, R2-07.* WP-09 creates `scripts/fde-anchor.sh` and the `scripts/fde-proof.sh` skeleton; WP-19
completes the latter. Lane order per repository is explicit in F: SEA-Forge 00c → 09 → 13 → 14 → 18 → 19 → 20 →
21 → 22 → 31 → 33; Cognate 15 → 30 → 23 → 33; Horizon 19 → 32 → 40 → 41; Workbench 23 → 34 → 42 → 44;
RealityTrace 10 → 16 → 17 → 18 → 22 → 33 → 43; DomainForge 10 → 12. "S3 starts after S0" means in a lane that
is free.

### RD-25 Total regression matrix
*Resolves the non-total matrix.* Listed rows stand. For every other transition: same class before and after
passes; any class → `insufficient` fails (lost evidence) unless coverage is unchanged and the element is listed
as intended; `insufficient` → any other class passes and is reported; `confirmed → declared_unobserved` fails;
`observed_undeclared → contradicted` passes only if listed as intended (it declares the behaviour a violation).
E §7 holds the matrix; other documents cite it instead of repeating rows.

### RD-26 Limits of anchor independence
*Resolves R2-08.* On a single-operator host, executors and the case keeper share an OS user, so the anchor
proves "not rewritten after the server-recorded push time", not "could not have been written by an executor".
A dedicated anchor credential with a protected tag namespace (`fde-anchor/*`: no delete, no force) is requested
from the owner (OD-6). Without it, anchoring rows are PARTIAL with this limit stated. Anchor verification needs
network access; offline it verifies the recorded verifier receipt only and reports PARTIAL.

### RD-27 Small corrections
SOP §7.6: lifecycle `closed` needs a recorded case-level close disposition and a terminal case record. B's
crosswalk gives the SOP terms for run and session. `disposition.recorded` carries `level ∈ case, episode`
everywhere it is described. Weak negative tests are strengthened: R-32 gains an actor/operation/resource prong;
R-72a asserts zero `contradicted` on the clean control; R-114 gains "delegation refused when the anchor is
missing". Each critical row's Change column lists the package that owns its negative test.

### Added owner decision
| ID | Decision | Recommendation | Effect if declined |
|---|---|---|---|
| OD-6 | Provide a dedicated anchor credential and protect the `fde-anchor/*` tag namespace on the SEA-Forge remote | Provide | anchoring rows stay PARTIAL (RD-26) |

A third Goldfish pass is not run: the remaining edits are mechanical applications of RD-20…RD-27, checked by
script and by the lead. The baseline for implementation is draft 3 of A–G, SOP v0.4.1 and this record.
