# C. Functional convergence matrix and fidelity definition (draft 2)

Sources: `Mission.md` (M§n; evidence contract ME1–ME5 and ME-levels; execution rules MX), SOP v0.4.1 (`S§n.m`,
identical numbering to v0.4), and the operator capabilities in the mission brief (U). Work packages `WP-nn` are in
F draft 2, tests `AT-nn` / `NT-nn` in G draft 2, journeys `J1`–`J10` in B, decisions `AD-n` and invariants `I-n` in
E, revision decisions `RD-n` in H. Draft 1 is in `draft1/`. Today column corrected against
`egm-review/03-codebase-feasibility.md`.

## 1. Fidelity definition (fixed before implementation, RD-16)

- **Unit:** one row = one individually testable requirement.
- **Status:** `PROVEN` (working behaviour, reproducible evidence, test executed in the integrated gate, executor
  provenance `program` or `live`), `PARTIAL`, `PLANNED` (not yet built or run, scheduled), `BLOCKED` (cannot proceed;
  cause stated), `NOT TESTED` (a prerequisite was skipped, or the behaviour did not occur through real execution).
  Only `PROVEN` counts. Fixture or `injected_fault` evidence never makes a row PROVEN for an execution claim.
- **Fidelity** = PROVEN rows / applicable rows, reported as **absolute counts and a percentage together**
  (e.g. "151 of 153, 98.7 %"). Target: 100 % of critical rows and every applicable row accounted for; the percentage
  alone is never the verdict.
- **Not applicable** requires the owner's written approval; it is not decided by an EGM record or by the
  implementing agent. Without approval the row stays in the denominator.
- **Critical** rows (column `C` = ●) must all be PROVEN. Every critical row has at least one negative test that
  fails with its named code when the protected property is broken (mutation re-applied by the verifier, G §8). §4
  proves the rule row by row. **One unproven critical row makes the verdict NOT MET** whatever the percentage.
- **Evidence levels stay distinct (ME-levels):** bounded interaction (R-87), repetition (R-88), variation (R-89),
  recovery (R-90), transfer (R-79) each have their own rows and ledger claims. After one case the expected status is:
  R-87 PROVEN; R-88 PARTIAL (the `program` path re-executed once); R-89 PLANNED; R-90 PARTIAL; R-79 PARTIAL.
- Rows that depend on an A3 confirmation on a separate surface, or on any human-reserved decision, are PARTIAL until S3
  exists and a real authenticator is provided (staged enforcement, RD-21; G §3). Anchoring rows are PARTIAL without
  OD-6 (RD-26).

Today column: `✓` exists and is evidenced, `◐` exists partly or is unverified, `✗` absent.

## 2. Matrix

### Scenario and declared model

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-01 | Source evidence for declared intent (the SOP), implemented behaviour (code, schemas), actual executions | M§1 | | sea-rs | ◐ no FDE model or runs | WP-13, WP-51 | J2 | AT-01 |
| R-02 | The four behaviour kinds arise from real execution, none instructed; kinds that do not arise are NOT TESTED | M§1, RD-03 | ● | sea-rs | ✗ | WP-09, WP-50, WP-51, WP-19 | J6 | AT-07, NT-26 |
| R-82 | Executor provenance (`program`, `live`, `injected_fault`) recorded on every occurrence; synthetic evidence never presented as actual execution | M§1, RD-02 | ● | Cognate / sea-rs | ✗ | WP-11, WP-15, WP-19 | — | AT-05, AT-18, NT-26 |
| R-129 | Execution inputs are deterministic and reproducible | M§1 | | sea-rs | ✗ | WP-15, WP-50 | — | AT-18 |
| R-03 | Contextually different meanings of one term preserved (two "settlement" concepts stay distinct) | M§1, S§2.2 | | DomainForge | ◐ semantic packs exist, no FDE packs | WP-13 | J2 | AT-01 |
| R-04 | Valid `.sea` IDM derived by the existing IDM method | M§2, S§12 | | DomainForge / sea-rs | ◐ method exists | WP-13 | J2 | AT-01 |
| R-05 | IDM covers purpose, actors, entities, states, operations, preconditions, affordances, policies, authority, expected outcomes, evidence and settlement obligations | M§2 | | sea-rs | ✗ | WP-13 | J2 | AT-02 |
| R-06 | Every substantive assertion classed declared / observed / inferred / proposed / unknown with provenance | M§2, S§4.2 | | sea-rs | ◐ pattern in Horizon IDM | WP-13 | J1, J2 | AT-02 |
| R-07 | Validated by the real compiler; source hash, `world_ref`, compiler version and validation output recorded | M§2 | ● | DomainForge | ✓ CLI | WP-13 | J2 | AT-01, NT-27 |
| R-08 | Declared model established, and externally anchored, before the evaluated executions | M§2 | ● | SEA-Forge | ◐ ordering rules exist, no external anchor | WP-13, WP-14 | J3 | AT-04, NT-18 |
| R-83 | Baseline repositories resolved by Git remote; dirty trees, frozen specs and worktrees preserved; baseline commits recorded | M orientation | | sea-rs | ◐ fetch done, serial gates not run | WP-00a, WP-00b, WP-00c | — | AT-18 |
| R-84 | Proof does not depend on unrelated Horizon product features | M§6 | | Horizon | ◐ | WP-19 | — | AT-14 |
| R-85 | Existing owners are used for model change, authority, execution, evidence and settlement; no second owner of any | M§5, RD-08 | | all | ◐ | WP-18, WP-14 | — | AT-12 |
| R-136 | Gap-to-evidence reconnaissance map and verified list of which debugger operations run on actual sources recorded | M orientation, MX1 | | sea-rs / Horizon | ◐ A-current-state | WP-00a, WP-40 | — | AT-18 |

### Field Case, navigation and control

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-10 | Field Case is an authoritative, versioned case with the S§4.2 content groups; record owners per S§12 | S§1.1, §4.2, §12 | | SEA-Forge | ◐ cases exist, no field-case document | WP-20, WP-21 | J1 | AT-03 |
| R-11 | Case opened or retrieved in Horizon and confirmed against the authoritative record | S§4.1 | | Horizon | ◐ case keeper | WP-41, WP-21 | J1 | AT-03, AT-14 |
| R-12 | Reopened case is continuable without the previous worker; journal never substitutes for records | S§4.3, §10.1 | ● | SEA-Forge / Horizon | ◐ handoff exists, no continuation read model | WP-21, WP-41 | J1, J9 | AT-15, NT-39 |
| R-100 | Resume selects a real or blocked affordance without reconstructing hidden context | S§11.2 | | Horizon | ◐ | WP-21, WP-41 | J1 | AT-15 |
| R-13 | Strategy decision recorded before substantive work and revisable | S§4.4 | | SEA-Forge | ✗ | WP-21 | J1 | AT-03 |
| R-94 | Stated versus revealed problem and intake heuristics recorded | S§4.5, §4.2 | | SEA-Forge | ✗ | WP-21 | J1 | AT-03 |
| R-14 | Activities are repeatable, omittable, unordered; repetition through the existing `repetition` marker | S§6 | | SEA-Forge | ◐ `repetition` marker exists; no activity tag in `PlanItem.markers` (tag lives in case records) | WP-21 | J1 | AT-03 |
| R-95 | Episode sized to one team, problem, loop, authority surface and evidence trail | S§6.2 | | SEA-Forge | ✗ | WP-21 | J3 | AT-03 |
| R-96 | Residue captured per settled episode (traces, examples, corrected outputs, evaluation cases, rules, decisions, policies, lessons) | S§6.3 | | SEA-Forge | ✗ | WP-21 | J10 | AT-10 |
| R-134 | Listen: direct observation, interpretation, disputed claim and missing evidence kept separate | S§7.1 | | Horizon | ◐ | WP-41 | J1 | AT-10 |
| R-15 | Candidate moves compared on seven dimensions; blocked and unknown shown as such | S§1.2, §7.2, U | | Horizon | ◐ `reach` is a capability ladder; move availability is new | WP-40, WP-41 | J3 | AT-10, AT-19 |
| R-16a | Evaluated episode's reference carries the eight S§7.2 fields (intention, expected change, invariants, predicted irrelevancies, failure and success regions, falsifiers, evidence source, decision-maker) | S§7.2 | | SEA-Forge | ◐ question and threshold exist | WP-11, WP-14 | J3 | AT-04 |
| R-16b | Reference fixed, hashed and externally anchored before execution | S§7.2, M§3 | ● | SEA-Forge + RealityTrace | ✗ | WP-14 | J3 | AT-04, NT-16 |
| R-16c | Reference not derived from the result | M§3 | ● | SEA-Forge | ✗ | WP-14 | J3 | NT-17 |
| R-114 | Freeze is two-phase; delegation refused unless the RealityTrace records, the case record and the anchor all exist (I-1; NT-33c: delegation refused when the anchor is missing) | RD-12, RD-10 | ● | SEA-Forge | ✗ | WP-14 | J3 | AT-04, NT-32, NT-33 |
| R-17 | Exploratory observation labelled at the time, never relabelled | S§7.2 | ● | SEA-Forge | ✗ | WP-14 | J3 | NT-13 |
| R-18 | Assignment bound to case, world, criterion, authority, budget, with the S§7.4 brief | S§2.1, §7.3-7.4 | | SEA-Forge / Cognate | ◐ delegation exists; Horizon assignment carries no episode reference, brief or budget | WP-20, WP-23 | J4 | AT-17 |
| R-126 | Assignment id `assign:<caseId>:<stepId>:<n>:<k>` (`n` episode counter, `k` delegation instance within the episode) so repeated activities do not collide | RD-13 | | Horizon | ✗ `assign:<case>:<step>` | WP-20, WP-23 | J4 | AT-17 |
| R-125 | Hand-off is the single door: composer, proposed step, preview, freeze, assignment prefilled from the freeze | RD-13 | | Horizon | ◐ `/desk/handoff` makes a case step only | WP-41 | J3, J4 | AT-19 |
| R-19 | SEA-Forge authorization at the action boundary for consequential actions | S§7.3 | ● | SEA-Forge | ✓ CEP loop; I-1 enforcement on execution identity is new | WP-14, WP-31 | J4 | AT-05, NT-34 |
| R-20 | Execution, cost, external effects, denials, interruptions captured | S§7.3, §5.3 | | Cognate / Workbench | ◐ no cost ledger in Cognate | WP-15, WP-23 | J4 | AT-17 |
| R-21 | Changing criteria, meaning or policy starts a new authorized evaluation version `n+1` | S§7.3 | ● | SEA-Forge | ✗ | WP-14, WP-16 | J3 | NT-01, NT-25 |
| R-97 | Within-reference executor corrections do not start a new evaluation version | S§7.3 | | SEA-Forge | ✗ | WP-14 | J4 | AT-04 |
| R-22a | Result classed S / F / U / C; no other value | S§7.5 | ● | RealityTrace / SEA-Forge | ◐ settlement kinds differ | WP-14, WP-21 | J5 | AT-09, NT-42 |
| R-22b | Settlement reliability and source independence recorded separately from the result | S§7.5 | ● | RealityTrace / SEA-Forge | ✗ | WP-14, WP-21 | J5 | AT-09, NT-42, NT-58 |
| R-23 | Executor cannot rewrite frozen criteria or attest its own acceptance (I-2 on the delegation chain) | S§3.2 | ● | SEA-Forge | ◐ single-actor `proposed_by` compare only; no chain, no execution identity | WP-21 | J5 | NT-10 |
| R-99 | Model provider does not determine authority or evidence | S§3.5 | | SEA-Forge | ◐ | WP-21 | J5 | AT-09 |
| R-24 | Six case-level dispositions, chosen by the authorized actor | S§7.5 | ● | SEA-Forge | ✗ | WP-21 | J5 | AT-10, NT-40 |
| R-25 | Four status axes distinct and never collapsed | S§7.6 | | SEA-Forge / Horizon | ✗ | WP-21, WP-41 | J1, J5 | AT-10 |
| R-26 | Tool completion is not settlement; settlement is not closure or capability | S§7.5 | ● | all | ◐ SEA-Forge principle only | WP-21 | J5 | AT-10, NT-41 |
| R-118 | Lifecycle is a pure function: `closed` only with terminal state and a case-level close disposition; `suspended` refuses freeze and delegation | RD-10, S§7.6 | ● | SEA-Forge | ✗ `CaseState` has no suspended | WP-21 | J1, J9 | AT-10, NT-22 |
| R-27 | Nine gates (met / blocked / unassessed / not applicable); no automatic advance; reassess on evidence change; evidenced decision not to deploy possible | S§8.1-8.2 | | SEA-Forge | ◐ milestones | WP-21 | J8 | AT-11 |
| R-28 | Not-applicable never waives a hard governance gate | S§8.1 | ● | SEA-Forge | ✗ | WP-21 | J8 | NT-11 |
| R-29 | Payment priced and measured in separate currencies; unmeasured is `unknown`, never zero | S§5.1-5.3 | | Workbench / Horizon | ◐ spend ceilings | WP-23, WP-41 | J3 | AT-17 |
| R-30 | Reliability dimensions T/A/I/K/D/X/N applied only where thresholds are set | S§7.7 | | RealityTrace | ◐ | WP-17 | J5 | AT-09 |
| R-92 | Four field tests (authority, evidence, residue, better next problem) applied at case review | S§1.5 | | SEA-Forge | ✗ | WP-21 | J9 | AT-10 |
| R-93 | FDE-owned work separated from DE/ADE implementation responsibility in the model | S§3.3 | | sea-rs | ✗ | WP-13 | J1 | AT-02 |
| R-135 | Horizon and GodSpeed-Agent present and propose; they never authorize | S§3.4 | | Horizon | ◐ | WP-41 | J5 | AT-19 |

### Semantic exhaust and anchoring

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-31 | Workflow executed through the real runtime path | M§3 | ● | Cognate / SEA-Forge | ◐ live loop skips without prerequisites | WP-15, WP-51, WP-16 | J4 | AT-05, NT-35 |
| R-32 | Events carry model and element identity, actor, operation, resource, context; an unregistered world, an element absent from the pinned world, or an undeclared actor/operation/resource is refused (NT-15a, NT-15b) | M§3 | ● | all emitters | ◐ outbox emits CEP `execution_trace`, no semantic fields | WP-11, WP-14, WP-15, WP-16 | J6 | AT-05, NT-02, NT-15 |
| R-33 | Events carry correlation, causation and a verifiable authority decision | M§3 | ● | all emitters | ◐ | WP-11, WP-14, WP-15, WP-16 | J6 | AT-05, NT-43 |
| R-34 | Events link expected outcome (pre-action reference) and actual outcome, evidence, settlement | M§3 | ● | RealityTrace | ◐ `evidence.recorded` only | WP-11, WP-16 | J5 | AT-05, NT-36 |
| R-35 | Original source records preserved; canonical contracts and adapters used | M§3 | ● | all | ✓ CAS | WP-11, WP-16 | J6 | AT-05, NT-44 |
| R-36 | Independent verifier can establish the expectation pre-existed the execution | M§3 | ● | RealityTrace / verifier | ✗ chain is internal only | WP-14, WP-54 | J3 | AT-04, NT-16, NT-17 |
| R-37 | Instrumentation begins when an episode needs evidence; coverage gaps reported | S§6.1 | | RealityTrace | ✗ coverage | WP-17 | J6 | AT-06 |
| R-102 | Model digest, each freeze digest and each pre-registration digest anchored externally before first run (`fde-anchor/*` tags) | RD-04 | ● | SEA-Forge | ✗ | WP-09, WP-13, WP-14 | — | AT-04, AT-18, NT-16, NT-18 |
| R-103 | Anchor tags are published by the case-keeper identity, never an executor credential | RD-04 | ● | SEA-Forge | ✗ | WP-14 | — | NT-48 |
| R-104 | Verifier issues its own receipt at freeze and checks server-side push time | RD-04 | | independent | ✗ | WP-54 | — | AT-18 |
| R-105 | Every interaction event signed by its emitter's workload identity from WP-33 (S3); unsigned or forged events cannot count | RD-05 | ● | Cognate / SEA-Forge / RealityTrace | ✗ | WP-33 | — | NT-19 |
| R-106 | `source.record_id` resolved against the owning system before an occurrence counts; `ingest.rejected` otherwise | RD-05, RD-07 | ● | RealityTrace | ✗ | WP-16 | — | AT-05, NT-35 |
| R-107 | `confirmed` requires at least one occurrence from an emitter independent of the executor (before WP-33, authenticity is source-record resolution plus this rule, RD-21) | RD-05 | ● | RealityTrace | ✗ | WP-17 | — | NT-20 |
| R-108 | Ordering from the owning system's recorded sequence and causation, never outbox arrival | RD-05 | ● | RealityTrace | ✗ | WP-16 | — | NT-49 |
| R-137 | Anchor independence limit stated: on a single-operator host the anchor proves "not rewritten after the server-recorded push time", not "could not have been written by an executor"; PARTIAL without OD-6 (dedicated anchor credential, protected `fde-anchor/*` tags); offline verification checks the receipt only and is PARTIAL | RD-26 | | SEA-Forge | ✗ | WP-09, WP-14 | — | NT-48 |
| R-117 | Reconciler detects orphan RealityTrace records and incomplete freezes; cross-system idempotency key `(system, record_id)` | RD-12 | | SEA-Forge | ✗ | WP-14, WP-22 | — | AT-04, AT-05 |
| R-123 | Contracts (schemas, error tables, fixtures) reviewed before code; vendored copies gated by digest | RD-18 | | CEP / consumers | ✗ | WP-11, WP-20 | — | AT-05 |

### Observed behaviour and comparison

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-40 | Observed representation computed from records, not hand-authored | M§4 | ● | RealityTrace | ✗ | WP-16, WP-17 | J6 | AT-06, NT-37 |
| R-41a | Preserves occurrence identity and frequency | M§4, S§9.1 | | RealityTrace | ✗ | WP-16 | J6 | AT-06 |
| R-41b | Preserves ordering | M§4, S§9.1 | | RealityTrace | ✗ | WP-16 | J6 | AT-06 |
| R-41c | Preserves context and provenance | M§4, S§9.1 | | RealityTrace | ✗ | WP-16 | J6 | AT-06 |
| R-41d | Preserves uncertainty and missing coverage | M§4, S§9.1 | | RealityTrace | ✗ | WP-16 | J6 | AT-06 |
| R-41e | Every aggregate traceable to its supporting occurrences | M§4 | ● | RealityTrace | ✗ | WP-16, WP-17 | J6 | AT-06, NT-47 |
| R-42 | Compared with the versioned declared model through semantic identities and explicit mappings | M§4 | | RealityTrace + DomainForge | ✗ | WP-12, WP-17 | J6 | AT-07 |
| R-119 | Key mapping from the semantic diff is mandatory when a concept is renamed; comparison refuses without it | RD-11 | ● | RealityTrace | ✗ | WP-18, WP-22 | J7 | NT-54 |
| R-112 | Policy semantics stay in DomainForge (`evaluate`); no second evaluator in RealityTrace; inexpressible clauses are `insufficient` with basis `not_evaluable` | RD-08 | ● | DomainForge / RealityTrace | ✗ `validate` emits violations only and skips operation-bound policies | WP-10, WP-12, WP-17 | J6 | AT-07, NT-52 |
| R-113 | Comparison profile (minimum support, coverage window, expected emitters, scope) frozen with the episode; no `confirmed` on vacuous truth or one occurrence unless the profile allows | RD-08 | ● | RealityTrace | ✗ | WP-11, WP-17 | J6 | AT-07, NT-28 |
| R-115 | Coverage manifest (emitters, event kinds, window) frozen with the episode; absence needs manifest coverage and a closed window | RD-12 | ● | RealityTrace / SEA-Forge | ✗ | WP-11, WP-14, WP-17 | J6 | AT-07, NT-30 |
| R-116 | `observed derive` takes `--episode` and `--kind`; exploratory occurrences never feed `confirmed` | RD-12 | ● | RealityTrace | ✗ | WP-16 | J6 | AT-06, NT-53 |
| R-43a | Class `confirmed` distinguished | M§4 | ● | RealityTrace | ✗ | WP-17 | J6 | AT-07, NT-28 |
| R-43b | Class `contradicted` distinguished | M§4 | ● | RealityTrace | ✗ | WP-17, WP-16 | J6 | AT-07, NT-03 |
| R-43c | Class `observed_undeclared` distinguished | M§4 | ● | RealityTrace | ✗ | WP-17 | J6 | AT-07, NT-04 |
| R-43d | Class `declared_unobserved` distinguished (triggered obligation inside the coverage window); untriggered elements reported `not exercised` | M§4, RD-03 | ● | RealityTrace | ✗ | WP-17 | J6 | AT-07, NT-29 |
| R-43e | Class `insufficient` / ambiguous distinguished, with basis | M§4 | ● | RealityTrace | ✗ | WP-17 | J6 | AT-07, NT-06 |
| R-44 | Each difference names evidence, context, affected elements and classification basis | M§4 | ● | RealityTrace | ◐ residual kind only | WP-17 | J6 | AT-07, NT-45 |
| R-45 | Changing inputs changes the observed model and comparison with no manual edit | M§4 | ● | RealityTrace | ✗ | WP-16, WP-17 | J6 | AT-08, NT-03 |
| R-46 | Episode comparison and structural comparison never merged (separate field and lists) | S§9.1, RD-12 | | RealityTrace | ✗ | WP-17 | J5, J6 | AT-07 |
| R-47 | Divergence not treated as noncompliance; six diagnostic causes kept open until evidence attributes | S§1.3, §9.2 | | RealityTrace | ◐ diagnostic kinds | WP-18 | J6 | AT-12 |

### Governed revision and development

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-50 | Investigation concludes: local correction, maintain difference, observe further, or structural candidate | M§5 | | RealityTrace | ◐ | WP-18 | J6 | AT-12 |
| R-98 | Recurrence trigger for structural review recorded in the proposal | S§9.3 | | RealityTrace | ✗ | WP-18 | J7 | AT-12 |
| R-51 | Original divergence and diagnosis recorded | M§5.1 | | RealityTrace | ✓ kinds | WP-18 | J6 | AT-12 |
| R-52 | Versioned candidate `.sea` with stated changed distinctions | M§5.2-5.3 | | DomainForge | ◐ `ConceptChange` | WP-18 | J7 | AT-12 |
| R-53 | Retained invariants and affected capabilities identified; candidate and semantic diff validated | M§5.4-5.5 | | DomainForge | ✓ diff | WP-18 | J7 | AT-12 |
| R-54a | Held-out executions excluded from diagnosis by a rule frozen in the proposal | M§5.6, RD-11 | ● | RealityTrace | ✗ | WP-18 | J7 | AT-12, NT-55 |
| R-54b | Failing retained-behaviour regression blocks success | M§5.6 | ● | RealityTrace | ✗ | WP-18 | J7 | AT-12, NT-08 |
| R-54c | Regression transition matrix: lost capability and unlisted weakening refuse promotion; listed intended changes pass | RD-11 | ● | RealityTrace | ✗ | WP-18 | J7 | AT-12, NT-21 |
| R-55 | Independent authorization before promotion; denied or unknown cannot promote; proposer ≠ approver | M§5.7 | ● | SEA-Forge | ◐ approval-gated `semantic_change` transition exists; no promote write path | WP-18 | J7 | AT-12, NT-05 |
| R-56 | Previous version, migration and recovery path preserved; old evidence stays attached to its version | M§5.8, S§9.4 | ● | SEA-Forge / RealityTrace | ◐ registry built from host config; no pin write path | WP-18, WP-22 | J7 | AT-12, NT-07 |
| R-120 | Every classification stamped with its world; a re-pin leaves no stale `confirmed` | RD-11 | ● | RealityTrace | ✗ | WP-17, WP-22 | J7 | NT-56 |
| R-121 | Promotion saga keyed `(proposal_id, candidate_world_ref)`; idempotent; uncertain outcomes fail closed | RD-11 | | SEA-Forge | ✗ | WP-22 | J7 | AT-12 |
| R-122 | Only `semantic_change` is a structural revision; in-flight cases keep their pinned world | RD-11 | | SEA-Forge | ◐ kinds exist in CEP | WP-18, WP-22 | J7 | AT-12 |
| R-57 | Subsequent bounded episode under the approved representation | M§5.9 | | all | ✗ | WP-18, WP-51 | J7 | AT-13 |
| R-58 | New representation makes a distinction or action available that was absent before, shown by old-vs-new world on the same records and a control variant; "no gain" reportable | M§5 | ● | all | ✗ | WP-18, WP-51 | J7 | AT-13, NT-38 |
| R-59 | Unsuccessful and inconclusive experiments preserved | M§5 | | RealityTrace | ✓ append-only | WP-18, WP-51 | J7 | AT-12 |
| R-101 | Where promotion is not justified, divergence and rationale for non-promotion retained | S§11.6 | | RealityTrace | ◐ | WP-18 | J7 | AT-12 |
| R-60 | Undeclared occurrence stays distinguishable from an authorized model change | M§7 | ● | RealityTrace | ✗ | WP-17 | J6 | NT-04 |
| R-61 | Developmental memory states what the next observer can now do; asset is a candidate until variation and transfer | S§9.5, §1.4 | | SEA-Forge / Horizon | ◐ lessons | WP-21, WP-41 | J10 | AT-13 |

### Horizon inspection and operator experience

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-62a | Operator can inspect the purposeful workflow and the pinned world | M§6 | | Horizon | ◐ debugger ops are generic object and link walks | WP-19, WP-40 | J2 | AT-14 |
| R-62b | Operator can inspect the observed execution and its evidence | M§6 | | Horizon | ✗ | WP-19, WP-40 | J5 | AT-14 |
| R-62c | Operator can inspect the declared-observed difference | M§6 | | Horizon | ✗ | WP-19, WP-40 | J6 | AT-14 |
| R-62d | Operator can see why a difference was classified as it was | M§6 | | Horizon | ✗ | WP-40 | J6 | AT-14 |
| R-62e | Operator can inspect operational history and source provenance | M§6 | | Horizon | ◐ | WP-19, WP-40 | J6 | AT-14 |
| R-62f | Operator can inspect the candidate change and hypothetical consequences | M§6 | | Horizon | ✗ `counterfactual` is an edge walk; `diff` is an object diff | WP-40 | J7 | AT-14 |
| R-62g | Operator can inspect authorization, settlement and resulting version | M§6 | | Horizon | ✗ | WP-19, WP-40 | J5, J7 | AT-14 |
| R-62h | Operator can see what the subsequent episode can now distinguish or do | M§6 | | Horizon | ✗ | WP-40 | J7 | AT-14 |
| R-63 | Horizon shows the same result as the programmatic path (rendered text equals ledger value) | M§7, S§11.7 | ● | Horizon | ✗ | WP-19 | J2, J5, J6, J7 | AT-14, NT-09 |
| R-64 | Separation of world versions, checkpoints, hypotheticals, authority decisions, observed consequences and developmental claims, by source mark | M§6 | ● | Horizon | ◐ value labels exist | WP-19, WP-41 | J2, J7 | AT-14, NT-46 |
| R-133 | Source-grounded labels and the existing Horizon interaction contracts used | M§6 | | Horizon | ✓ v2 contracts | WP-19 | J2 | AT-14 |
| R-132 | Horizon debugger operations extended where they cannot answer M§6 (move availability, regression consequences, re-derivation, semantic diff) | M§6, RD-13 | | Horizon | ◐ `reach`, `counterfactual`, `replay`, `diff` have narrower meanings | WP-40 | J3, J6, J7 | AT-14 |
| R-65 | Explore, discover affordances and blocked possibilities, investigate evidence, compose, delegate, monitor, interrupt, resume, inspect plans and costs, decide at boundaries, compare candidates, preserve work | U | | Horizon + Workbench | ◐ | WP-41, WP-42 | J1 to J10 | AT-19 |
| R-66 | One coherent environment across Horizon and Workbench | U | | Horizon + Workbench | ◐ connect on a branch | WP-42 | J4 | AT-19 |
| R-67 | Human-reserved decisions cannot be made by an agent, including one driving the operator's browser | S§3.2, RD-06 | ● | SEA-Forge / Cognate / Horizon | ✗ | WP-30, WP-31, WP-32 | J5, J7 | NT-12, NT-24 |
| R-110 | Reserved decisions (settle by a human, case disposition, hard-gate waiver, promote, controller bootstrap) are confirmed only on the A3 surface (paired phone); the desk shows "confirm on your phone" | RD-06 | ● | Horizon / SEA-Forge | ✗ | WP-31, WP-32 | J5, J7 | NT-24, NT-12 |
| R-111 | SEA-Forge, not Horizon, validates A3 evidence bound to the operation digest and expiry | RD-06 | ● | SEA-Forge | ✗ | WP-31 | J7 | NT-51 |
| R-109 | A virtual-authenticator assertion never makes a human-reserved row PROVEN | RD-06 | ● | sea-rs | ✗ | WP-53 | — | NT-50 |
| R-68 | Case continuation and receiver-specific handoff (delivery engineering, agent engineering) from source-owned records; reusable handoff usable by a different FDE | S§10.1-10.4 | | SEA-Forge / Horizon | ◐ | WP-21, WP-41 | J9 | AT-15 |
| R-69 | Closure only by authorized disposition with economic account, liabilities, recoverability | S§10.3 | ● | SEA-Forge | ✗ | WP-21 | J9 | AT-10, NT-23 |
| R-70 | Retrieved context is cited reference material, never canonical | S§3.4 | | Context Kernel | ◐ exists, not linked to the brief | WP-44 | J4 | AT-17 |
| R-91 | One vocabulary: framing, projection, case horizon, residual, residue, source mark, blocked move, difference, carry-over, episode/run/assignment/session, settle/dispose/close; glossary crosswalk | S§2.2, RD-14 | | sea-rs / Horizon | ✗ | WP-13, WP-41 | J1 to J10 | AT-01, AT-19 |

### Verification, evidence and transfer

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-71a | Frozen expectation cannot be rewritten by a later observation | M§7 | ● | SEA-Forge | ✗ | WP-14, WP-16 | — | NT-01, NT-25 |
| R-71b | A trace lacking model identity or required evidence cannot produce a verified conclusion | M§7 | ● | RealityTrace | ✗ | WP-16 | — | NT-02 |
| R-71c | A changed or missing record affects the representation and comparison appropriately | M§7 | ● | RealityTrace | ✗ | WP-16 | — | NT-03, NT-03b |
| R-71d | Undeclared occurrence remains distinguishable from an authorized change | M§7 | ● | RealityTrace | ✗ | WP-17 | — | NT-04 |
| R-71e | Denied or unknown authority cannot promote | M§7 | ● | SEA-Forge | ✗ | WP-18 | — | NT-05 |
| R-71f | Missing observation remains unresolved | M§7 | ● | RealityTrace | ✗ | WP-17 | — | NT-06 |
| R-71g | Settled evidence stays attached to its original model version | M§7 | ● | RealityTrace | ✗ | WP-18, WP-22 | — | NT-07 |
| R-71h | Candidate cannot claim success when regression fails | M§7 | ● | RealityTrace | ✗ | WP-18 | — | NT-08 |
| R-71i | Horizon displays the same source-grounded result as the programmatic path | M§7 | ● | Horizon | ✗ | WP-19 | — | NT-09 |
| R-72a | Clean conforming control runs first, is non-vacuous and yields zero `contradicted` (NT-31a empty ledger, NT-31b zero `contradicted` asserted) | M§7 | ● | all | ✗ | WP-17, WP-52 | — | AT-18, NT-31 |
| R-72b | Deliberate fault injections exist per protected property, labelled `injected_fault`, never supporting an execution claim | M§7 | ● | all | ✗ | WP-52, WP-19 | — | NT-26, NT-59 |
| R-73a | Final confirmation by a fresh-context verifier on a different model CLI; otherwise PARTIAL | M§7, S§3.5, RD-17 | ● | independent | ✗ | WP-54 | — | AT-18, NT-57 |
| R-73b | Verifier makes its own observation of each settled outcome | RD-17 | ● | independent | ✗ | WP-21, WP-54 | J5 | AT-09, NT-58 |
| R-73c | Verifier re-applies hash-pinned mutation patches from a clean checkout and chooses two extra negative tests | RD-17 | ● | independent | ✗ | WP-52, WP-54 | — | NT-59 |
| R-74a | Bundle: run manifest (commits, local changes, versions, environment, commands, run ids) | ME1 | | sea-rs | ◐ conventions | WP-53 | — | AT-18 |
| R-74b | Bundle: semantic evidence (sources, model and hash, pack relationships, assumptions, candidate, diff, approval) | ME2 | | sea-rs | ◐ | WP-53 | — | AT-18 |
| R-74c | Bundle: operational evidence (frozen references with times, events, authority decisions, RealityTrace ids, derived model, residuals, subsequent episode) | ME3 | | sea-rs | ◐ | WP-53 | — | AT-18 |
| R-74d | Bundle: verification evidence (AT/NT results, exit codes, mutation results, verifier report, skipped checks and effect) | ME4 | | sea-rs | ◐ | WP-53 | — | AT-18 |
| R-86 | Horizon artifacts (rendered captures, parity output) are in the bundle | ME, RD-16 | | Horizon / sea-rs | ✗ | WP-19, WP-53 | — | AT-14, AT-18 |
| R-75 | Claim ledger over the eight claim areas with exact claim, location, command, evidence, result PROVEN / PARTIAL / PLANNED / NOT TESTED, scope | ME5 | | sea-rs | ✗ | WP-19, WP-53 | — | AT-18 |
| R-127 | A failed or skipped test never supports a PROVEN claim; unproven claims are stated, not hidden | MX | ● | sea-rs | ✗ | WP-19, WP-53 | — | AT-18, NT-61 |
| R-76a | One command (or short sequence) regenerates the proof | ME | | sea-rs | ◐ `e2e-world-loop.sh` first run unrecorded | WP-19, WP-53 | — | AT-18 |
| R-76b | Verifier can verify and replay immutable raw records to recompute representation and comparison | RD-03 | | sea-rs | ✗ | WP-19, WP-53 | — | AT-18 |
| R-76c | `program` episodes re-executed on a clean cell with class-level comparison | RD-03 | | sea-rs | ✗ | WP-15, WP-53 | — | AT-18 |
| R-77 | Owner repositories' proof ledgers updated | ME | | each repo | ✗ | WP-53 | — | AT-18 |
| R-78 | Acceptance tests and falsifiers registered, hash-pinned and anchored before results exist | MX2 | ● | sea-rs | ◐ preregistration convention, no anchor | WP-09, WP-50 | — | AT-18, NT-60 |
| R-87 | Evidence level: bounded interaction, one case, stated as such | ME-levels | | sea-rs | ✗ | WP-53 | — | AT-18 |
| R-88 | Evidence level: repetition (re-execution of `program` episodes) recorded separately | ME-levels | | sea-rs | ✗ | WP-53 | — | AT-18 |
| R-89 | Evidence level: variation under meaningfully different inputs or contexts recorded separately | ME-levels, S§8 | | sea-rs | ✗ | WP-53 | — | AT-18 |
| R-90 | Evidence level: recovery under fault recorded separately | ME-levels, S§8 | | sea-rs | ✗ | WP-22, WP-53 | — | AT-18 |
| R-79 | Evidence level: transfer; templates and instrumentation move to a second bounded situation without rewriting the evidence mechanism, plus a same-gap variant (PARTIAL, fixture-based) | S§11.8 | | all | ✗ | WP-56 | — | AT-16 |
| R-80 | Gauntlet / SWE_SEED verification against frozen references feeds evidence | S§3.4, §7.3 | | Gauntlet / SWE_SEED | ◐ E7 path; Gauntlet unlinked | WP-43 | J5 | AT-17 |
| R-81a | Installable from release artifacts on a clean cell; FDE gates added to `local_operator`, which stays GO | completion mandate | | godspeed-readiness | ◐ `local_operator` exists | WP-57 | — | AT-20 |
| R-81b | A skipped gate is NO-GO | completion mandate | ● | godspeed-readiness | ◐ | WP-57 | — | AT-20, NT-14 |
| R-124 | Operate package: topology and ports, configuration, migration of existing cases and ledgers, integration observability, retention and backup of new stores | RD-18 | | all | ✗ | WP-55 | — | AT-20 |
| R-128 | Final delivery reports verdict, changes, run commands, evidence location, claim ledger, remaining blockers | M final delivery | | sea-rs | ✗ | WP-53 | — | AT-18 |
| R-130 | Existing skills and verification conventions used; durable status for resumption; changes focused and reversible | MX | | sea-rs | ◐ | WP-00a, WP-53 | — | AT-18 |
| R-131 | Irreversible, shared-environment and frozen-authority changes obtain owner approval; a wrong contract follows its owner's revision process | MX | | sea-rs | ◐ | WP-00b | — | AT-18 |

## 3. Coverage check

Independent list of normative requirements drawn from `Mission.md` and SOP v0.2 before this table was filled; the
table is the check. SOP clause numbers follow v0.4.1 (identical to v0.4); v0.2's own numbering is not used.

| Source item | Rows |
|---|---|
| M orientation: baseline by Git remote, commits, dirty state, versions, existing owners, debugger operations verified | R-83, R-85, R-136 |
| M§1 provenance of scenario; four kinds; contextual terms; deterministic inputs; no synthetic-as-actual | R-01, R-02, R-82, R-03, R-129 |
| M§2 valid IDM, coverage, assertion classes, compiler validation, model before executions | R-04, R-05, R-06, R-07, R-08, R-102 |
| M§3 runtime path, event identity, authority, correlation, expected/actual links, preserved records, "before" provable | R-31, R-32, R-33, R-34, R-35, R-36, R-16b, R-16c, R-105 to R-108 |
| M§4 derived representation, occurrence preservation, mapping, five classes, difference basis, input sensitivity | R-40, R-41a to R-41e, R-42, R-43a to R-43e, R-44, R-45, R-112, R-113, R-115, R-116, R-119 |
| M§5.1–5.9 revision steps, preserved experiments, existing owners | R-50 to R-59, R-54a to R-54c, R-120 to R-122, R-85 |
| M§6 inspection items, labels, separation, no unrelated feature dependence | R-62a to R-62h, R-63, R-64, R-84, R-132, R-133 |
| M§7 nine proofs; clean control; fault injections; fresh verifier | R-71a to R-71i, R-72a, R-72b, R-73a to R-73c |
| ME1–ME4 bundle | R-74a to R-74d, R-86 |
| ME5 claim ledger; ME-levels; single command; owner ledgers | R-75, R-127, R-87 to R-90, R-79, R-76a to R-76c, R-77 |
| MX1–MX6 phases, pre-registration, approvals, status, unproven stays unproven | R-136, R-78, R-130, R-131, R-127, R-128 |
| M final delivery six items | R-128 |
| S§1.1–1.5 purpose, next move, difference as information, outcome, four field tests | R-10, R-15, R-47, R-61, R-92 |
| S§2, 2.1, 2.2 layers, Field Case/episode/assignment, vocabulary | R-10, R-18, R-91 |
| S§3.1–3.5 roles, separation, FDE ownership, components, provider neutrality | R-23, R-93, R-135, R-70, R-80, R-99, R-73a |
| S§4.1–4.5 open/resume, content, continuation, strategy, intake | R-11, R-10, R-12, R-13, R-94 |
| S§5.1–5.3 payment | R-29, R-20 |
| S§6, 6.1–6.3 activities, instrumentation start, sizing, residue | R-14, R-37, R-95, R-96 |
| S§7.1–7.7 Listen, Aim, Move, assignment brief, Claim, status axes, reliability | R-134, R-15, R-16a, R-16b, R-17, R-18, R-19, R-21, R-97, R-22a, R-22b, R-24, R-25, R-26, R-118, R-30 |
| S§8.1–8.2 gates | R-27, R-28 |
| S§9.1–9.5 comparisons, causes, recurrence, revision record, memory | R-46, R-41a to R-41e, R-47, R-98, R-52 to R-56, R-61 |
| S§10.1–10.4 continuation, handoff, disposition, reusable handoff | R-12, R-68, R-69, R-24 |
| S§11 items 1–8 and outcome | R-04, R-100, R-16b, R-43a to R-43e, R-22a, R-24, R-101, R-63, R-79, R-76a |
| S§12 records and owners | R-10, R-35, R-16a |
| S§13 conflicts (non-normative) | none required; vocabulary conflicts resolved in R-91 |
| RD-04 to RD-12, RD-18, RD-26 revision decisions | R-102 to R-108, R-109 to R-111, R-112 to R-117, R-118 to R-123, R-124, R-125, R-126, R-137 |

Totals (counted mechanically from the matrix lines starting `| R-`):

169 rows; 76 critical rows; 102 rows with no implementation today (✗); 60 partial today (◐); 7 existing and evidenced (✓). Every critical row has at least one negative test (§4).

## 4. Critical row → negative test(s)

| Critical row | Negative test(s) |
|---|---|
| R-02 | NT-26 |
| R-82 | NT-26 |
| R-07 | NT-27 |
| R-08 | NT-18 |
| R-12 | NT-39 |
| R-16b | NT-16 |
| R-16c | NT-17 |
| R-114 | NT-32, NT-33 (a, b, c anchor missing) |
| R-17 | NT-13 |
| R-19 | NT-34 |
| R-21 | NT-01, NT-25 |
| R-22a | NT-42 |
| R-22b | NT-42, NT-58 |
| R-23 | NT-10 |
| R-24 | NT-40 |
| R-26 | NT-41 |
| R-118 | NT-22 |
| R-28 | NT-11 |
| R-31 | NT-35 |
| R-32 | NT-02, NT-15 (a world; b actor/operation/resource) |
| R-33 | NT-43 |
| R-34 | NT-36 |
| R-35 | NT-44 |
| R-36 | NT-16, NT-17 |
| R-102 | NT-16, NT-18 |
| R-103 | NT-48 |
| R-105 | NT-19 |
| R-106 | NT-35 |
| R-107 | NT-20 |
| R-108 | NT-49 |
| R-40 | NT-37 |
| R-41e | NT-47 |
| R-119 | NT-54 |
| R-112 | NT-52 |
| R-113 | NT-28 |
| R-115 | NT-30 |
| R-116 | NT-53 |
| R-43a | NT-28 |
| R-43b | NT-03 |
| R-43c | NT-04 |
| R-43d | NT-29 |
| R-43e | NT-06 |
| R-44 | NT-45 |
| R-45 | NT-03 |
| R-54a | NT-55 |
| R-54b | NT-08 |
| R-54c | NT-21 |
| R-55 | NT-05 |
| R-56 | NT-07 |
| R-120 | NT-56 |
| R-58 | NT-38 |
| R-60 | NT-04 |
| R-63 | NT-09 |
| R-64 | NT-46 |
| R-67 | NT-12, NT-24 |
| R-110 | NT-24, NT-12 |
| R-111 | NT-51 |
| R-109 | NT-50 |
| R-69 | NT-23 |
| R-71a | NT-01, NT-25 |
| R-71b | NT-02 |
| R-71c | NT-03, NT-03b |
| R-71d | NT-04 |
| R-71e | NT-05 |
| R-71f | NT-06 |
| R-71g | NT-07 |
| R-71h | NT-08 |
| R-71i | NT-09 |
| R-72a | NT-31 (a, b) |
| R-72b | NT-26, NT-59 |
| R-73a | NT-57 |
| R-73b | NT-58 |
| R-73c | NT-59 |
| R-127 | NT-61 |
| R-78 | NT-60 |
| R-81b | NT-14 |
