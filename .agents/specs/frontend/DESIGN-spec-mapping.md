# DESIGN.md ↔ SEA Forge UX and System Contract Mapping

**Status:** Mockup traceability v0.2  
**Purpose:** Keep OpenDesign-generated surfaces aligned with SEA Forge’s governed lifecycle without turning `DESIGN.md` into a backend specification.  
**Companion files:** `DESIGN.md`, `MOCKUP-BRIEF.md`

---

## 1. Ground rule

The Workbench is a **governed client over source-backed views and protected commands**.

It is not merely read-only, but it also does not own canonical lifecycle truth.

The GUI may:

- render source-backed and rebuildable views;
- keep reversible local drafts;
- issue typed protected commands;
- display server decisions;
- subscribe to events;
- open evidence, provenance, and machine records;
- present lawful recovery paths.

The GUI may not independently:

- evaluate authority;
- activate sentries;
- reduce canonical case state;
- accept settlement;
- determine declaration standing;
- promote capability;
- verify ledger integrity;
- grant agent permissions;
- mutate canonical records;
- infer local authority from imported data.

Preserve:

```text
navigation ≠ mutation
draft ≠ proposal
proposal ≠ committed plan
approval ≠ successful execution
execution success ≠ accepted settlement
agent termination ≠ settlement
accepted settlement ≠ proven capability
derived view ≠ source truth
import ≠ local authority
```

The actual repository inspection will determine concrete command, query, event, and file ownership. Mockups must not invent those contracts.

---

## 2. Visual object mapping

| Design object | UX meaning | Primary source concept | Required display states |
|---|---|---|---|
| `GovernedFocusHeader` | Current object and dominant decision | Current route view model | current, stale, blocked, degraded |
| `GovernedStatusPill` | Typed state label | Domain-specific state | exact domain + exact value |
| `DualStateIndicator` | Prevent false completion | Execution state + settlement state | all meaningful combinations |
| `AgentStateHeader` | Separate agent concerns | Dialogue + termination + settlement | independent dimensions |
| `SourceFreshnessBadge` | Show whether view is current | Source/projection metadata | current, stale, unknown |
| `IntegrityIndicator` | Show assurance and halt state | Ledger/integrity result | partial, verified, pending, failed |
| `WhyStatePanel` | Explain cause and next path | Rules, sentries, criteria, decisions | satisfied, failed, unavailable |
| `EvidenceDrawer` | Inspect proof and provenance | Evidence and source refs | verified, missing, stale, quarantined |
| `ProtectedActionButton` | Request governed transition | Protected command | enabled, disabled, submitting, committed, denied, escalated, unknown |
| `ReadinessConsole` | Assess operation-sensitive readiness | Readiness checks and current snapshot | ready, degraded, stale, blocked, halted |
| `CaseCreationWorkbench` | Create reversible case draft | Entry options, templates, models | draft, incomplete, unavailable |
| `PlanPreflightPanel` | Review exact governed contract | Server preflight result | ready, warning, blocked, escalation, stale |
| `CaseHorizonBoard` | Show currently reachable case paths | Plan, sentries, case events | spendable, active, awaiting, blocked, future, settled |
| `ExecutionConsole` | Observe command execution | Run, trace, evidence | execution and settlement separately |
| `AgentTaskConsole` | Supervise bounded delegation | Endpoint, dialogue, permission, transcript | dialogue, termination, settlement |
| `ApprovalDecisionPanel` | Insert human judgment | Approval request and authority context | pending, approved, rejected, expired |
| `CriterionSettlementMatrix` | Explain whether outcome counts | Criteria, evidence, declarations | pass, fail, unknown, excluded, quarantined |
| `CapabilityPromotionPanel` | Explain demonstrated capacity | Qualifying settlements + policy | attempted, demonstrated, proven, degraded, quarantined |
| `ArtifactMaturityPanel` | Show artifact stage and gates | Artifact descriptor and transitions | cognitive, intellectual, product, capital |
| `IntegrityInspector` | Verify history and evidence | Ledger/checkpoints/proofs | valid, invalid, pending, forked |

---

## 3. Primary page mapping

### P2 — Readiness Overview

| Dimension | Mapping |
|---|---|
| User job | Know whether the cell is safe and capable enough for intended work |
| Primary visual object | `ReadinessConsole` |
| Inputs | Cell context, intended operation, check scope |
| Outputs | Readiness state, blockers, limitations, next lawful actions |
| Protected actions | Check, validate, rebuild, probe where supported |
| Required distinctions | ready vs degraded vs stale vs blocked vs integrity halted |
| Mockup emphasis | Hard gates, intended-operation selector, current affordances, recent invalidations |

### P11 — New Case

| Dimension | Mapping |
|---|---|
| User job | Choose the correct starting representation |
| Primary visual object | Entry-source selector inside `CaseCreationWorkbench` |
| Inputs | Purpose, desired outcome, entry mode |
| Outputs | Local `CaseDraft` skeleton |
| Protected actions | None by default |
| Required distinctions | recommendation vs selection; draft vs committed |
| Mockup emphasis | Outcome-oriented entry cards, unavailable-path reasons, local draft label |

### P12 — Case Configuration

| Dimension | Mapping |
|---|---|
| User job | Bind meaning, criteria, resources, delegates, and limits |
| Primary visual object | Three-region configuration workbench |
| Inputs | Domain model, parameters, criteria, environments, endpoints, limits |
| Outputs | Complete `CasePlanDraft` |
| Protected actions | None until preflight/commit |
| Required distinctions | error, incomplete, unavailable, warning, notice |
| Mockup emphasis | Configuration navigation, active editor, draft-health rail |

### P13 — Case Preflight

| Dimension | Mapping |
|---|---|
| User job | Understand the exact contract before it becomes real |
| Primary visual object | `PlanPreflightPanel` |
| Inputs | Complete draft and governing digests |
| Outputs | Preflight result and commit eligibility |
| Protected actions | Commit case, request approval where supported |
| Required distinctions | nothing executed; ready, warning, blocked, escalated, stale |
| Mockup emphasis | Work burden, authority burden, settlement burden, immutable commit impact |

### P15 — Case Overview

| Dimension | Mapping |
|---|---|
| User job | Know where the case is and what needs attention |
| Primary visual object | Case summary and current-horizon focus |
| Inputs | Case ID |
| Outputs | Current consequence, attention, milestone, capability effect |
| Protected actions | Explicit case actions only |
| Required distinctions | active, parked, completed, terminated, inconsistent |
| Mockup emphasis | Spendable count, attention, latest settlement, milestone trajectory |

### P16 — Case Horizon

| Dimension | Mapping |
|---|---|
| User job | Choose or resolve the next reachable path |
| Primary visual object | `CaseHorizonBoard` |
| Inputs | Case state, plan, sentries, resources |
| Outputs | Selected item and lawful action |
| Protected actions | Start, cancel, add discretionary item, replan where supported |
| Required distinctions | spendable, active, awaiting, blocked, future, settled |
| Mockup emphasis | Why spendable, why blocked, exact next action |
| Constraint | Dragging never changes canonical state |

### P22 — Run Detail

| Dimension | Mapping |
|---|---|
| User job | Observe and control command execution |
| Primary visual object | `ExecutionConsole` |
| Inputs | Run ID |
| Outputs | Execution evidence and settlement handoff |
| Protected actions | Cancel, retry as new episode |
| Required distinctions | execution state vs settlement state |
| Mockup emphasis | Sticky dual-state header, boundary, sandbox, output, evidence, settlement preview |

### P23 — Agent Task Detail

| Dimension | Mapping |
|---|---|
| User job | Supervise bounded delegated work |
| Primary visual object | `AgentTaskConsole` |
| Inputs | Agent run ID |
| Outputs | Dialogue, proof, transcript evidence, settlement handoff |
| Protected actions | Cancel, permission decision route, transcript access request |
| Required distinctions | dialogue vs termination vs settlement |
| Mockup emphasis | Budgets, endpoint metadata, permission request, transcript digest, SWE_SEED proof |
| Constraint | No consumer-chat-first layout and no silent provider fallback |

### P27 — Settlement Detail

| Dimension | Mapping |
|---|---|
| User job | Determine what happened, whether it counts, and why |
| Primary visual object | `CriterionSettlementMatrix` |
| Inputs | Settlement ID |
| Outputs | Basis, consequence, recovery, capability contribution |
| Protected actions | Eligible scoped declaration only |
| Required distinctions | pending, accepted, rejected, escalated, quarantined |
| Mockup emphasis | Expected vs observed, criteria/evidence, declaration standing, next lawful recovery |
| Constraint | No generic “Accept Settlement” button |

### P32 — Capability Detail

| Dimension | Mapping |
|---|---|
| User job | Understand what has actually been demonstrated |
| Primary visual object | `CapabilityPromotionPanel` |
| Inputs | Capability ID |
| Outputs | State, variation, recovery, regression, promotion gaps, proof path |
| Protected actions | Start proof case through P11–P13 |
| Required distinctions | attempted, demonstrated, proven, degraded, quarantined |
| Mockup emphasis | Conservative claim, source settlements, next possible vs currently spendable proof path |

---

## 4. Supporting page families

| Family | Pages and purpose | Principal UI objects |
|---|---|---|
| Cell and context | Cell Gateway, Readiness, Identity/Authority | readiness, actor context, policy, integrity |
| Thoth | Ask Thoth, Answer Detail | typed question composer, grounded answer, disclosure state |
| Assets | Catalog and Detail | availability ladder, version, compatibility, evidence |
| Domain | Models, Model Workbench, Projection Detail | `.sea` editor, validation, concept refs, projection proof |
| Cases | Catalog, Overview, Horizon, Timeline, Plan | case reducer views, sentry explanation, events |
| Inbox | Approvals and Human Tasks | exact decision context, TTL, SoD |
| Operations | Monitor, Run, Agent Task, Thoth Manager | event cursor, concurrency, scoped controls |
| Evidence and audit | Evidence, Settlement, Audit, Integrity | evidence refs, provenance, ledger proof |
| Memory and capability | Recall, Capability Catalog/Detail | protected retrieval, influence, promotion |
| Artifacts and federation | Pipelines, Artifact Catalog/Detail/Transition, Bundles | lineage, stage gates, isolation, adoption |
| Administration | Admin, Maintenance and Debt | versions, stale snapshots, quarantine, repair |

---

## 5. State vocabularies required in mockups

### Availability

```text
declared
installed
available
validated
demonstrated
proven
degraded
unsupported
unknown
stale
quarantined
deprecated
```

### Governance

```text
allow
deny
escalate
degraded
pending
expired
```

### Execution

```text
not_started
queued
waiting_for_capacity
awaiting_authority
awaiting_approval
preparing_environment
running
streaming
cancelling
cancelled
interrupted
succeeded
failed
timed_out
```

### Settlement

The canonical settlement vocabulary is the union of every state referenced
by P27 and the governed-workbench UX epic. Use exactly this set everywhere
(`CaseHorizonBoard`, case-plan state definitions, mapping tables, epic
references); do not alias or subset:

```text
pending         # not yet activated/evaluated
evaluating      # criteria under assessment
accepted        # criteria met, settled positive
rejected        # criteria failed or authority denied
escalated       # punted to human/approval flow
quarantined     # integrity gate held the item
unsettled       # active run, no settlement record yet
future          # not yet reachable in the case plan
settled         # terminal (accepted or rejected); horizon-board rollup
```

> `unsettled` and `pending` are distinct: `unsettled` means a run/case is
> active and no settlement has been declared; `pending` means the item is
> known but not yet under evaluation. `settled` is a rollup over terminal
> `accepted`/`rejected` for horizon-board grouping; it is not a new
> outcome. Older drafts that omitted `pending`, `future`, `rejected`, or
> `settled` are superseded by this canonical set.

### Case and plan items

```text
draft
committed
enabled
active
waiting
blocked
parked
completed
terminated
reactivated
```

### Integrity

```text
unverified
verifying
verified_local
tamper_evident
externally_witnessed
pending_integrity
invalid
fork_detected
```

Mockups may use friendlier display labels, but must preserve the distinctions.

---

## 6. Protected action mapping

The GUI may visually initiate protected actions such as:

- create or migrate cell;
- run readiness checks;
- validate or rebuild a self-model projection;
- probe an endpoint;
- commit a case;
- activate a plan item;
- decide an approval;
- complete a human task;
- cancel a scoped run;
- retry as a new episode;
- request transcript access;
- submit an eligible settlement declaration;
- transition an artifact;
- import or adopt a federation bundle;
- administer extension, endpoint, model, or environment state.

Each action must show:

- requested operation;
- affected resource;
- exact boundary;
- authority state;
- expected consequence;
- whether evidence is preserved;
- whether the action can be reversed;
- what successful settlement means.

The mockup must not imply that button press itself changes canonical state.

---

## 7. Source, projection, and freshness mapping

Every substantial page should be capable of showing:

```text
source record references
source cell
record or view version
projection descriptor/version
current or stale state
last verified time
invalidation cause
integrity level
```

Use the right-side drawer to avoid clutter.

A stale view displays the last verified result plus the reason it is no longer current.

An event stream is delivery, not truth. A disconnected live view must show:

```text
Last confirmed event: EVT-…
State may be stale.
Reconnecting…
```

---

## 8. Corrections from the previous design mapping

The prior mapping is superseded in these areas:

1. The GUI is not limited to a read-only projection plus a small fixed write set. It is a governed client capable of issuing the full set of protected commands exposed by the completed architecture.
2. Settlement is not the center of every screen.
3. “Mission,” “Situation,” and “Affordance” are useful UX concepts but must not be treated as invented canonical SEA Forge record types.
4. The three-choice Recommended/Safe/Experimental pattern is conditional, not universal.
5. Agent provider/model identity is visible as metadata even though agents are not branded personas.
6. Board, list, graph, console, and audit are valid alternate projections when they share source truth.
7. CLI equivalence is optional and shown only where a real command exists.
8. Presentational components do not leave evidence; governed operations do.
9. Color is not the primary or sole state carrier.
10. Capability includes variation, recovery, regressions, burden, and promotion policy—not merely a confidence score.
11. The GUI must represent source/freshness/integrity in addition to operational status.
12. Execution, agent termination, settlement, and capability remain separate state domains.

---

## 9. Mockup acceptance criteria

An OpenDesign output is acceptable only if:

- one application shell connects all screens;
- every screen has one dominant governed focus;
- visual density is operational and avoids giant cards;
- current state has an exact domain label;
- execution and settlement are visibly separate;
- agent dialogue, termination, and settlement are visibly separate;
- source/freshness/evidence are reachable in one action;
- blocked actions explain why;
- no invented choice, cost, confidence, command, or backend state appears;
- Readiness, Case Horizon, Run/Agent Task, Settlement, and Capability form a coherent end-to-end path;
- failure and recovery paths are represented, not only a happy path;
- mobile adaptations preserve inspection and judgment rather than compressing desktop editors beyond usability.
