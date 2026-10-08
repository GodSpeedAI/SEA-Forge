# SEA Forge Governed Workbench Frontend Completion Eval

**Version:** v0.1
**Purpose:** Independent, evidence-based completion gate for the SEA Forge Governed Workbench frontend
**Canonical product contract:** The embedded `SEA Forge Governed Workbench UX` epic at the end of this document
**Primary verdicts:** `PASS_STOP`, `FAIL_CONTINUE`, `BLOCKED`

---

## Paste-ready invocation

Use this entire document as the evaluator's instructions and source contract.

```text
Evaluate the SEA Forge Governed Workbench frontend against the protocol and
canonical epic embedded in this document.

Repository root: <REPO_ROOT>
Application URL or start command: <APP_URL_OR_START_COMMAND>
Build command: <BUILD_COMMAND_OR_UNKNOWN>
Test command: <TEST_COMMAND_OR_UNKNOWN>
Test identities/fixtures: <IDENTITIES_FIXTURES_OR_UNKNOWN>
Declared exclusions: <NONE_OR_EXPLICIT_STORY_IDS_WITH_REASONS>

Do not modify the repository or application. Inspect, run, interact, test, and
report. Return exactly one verdict: PASS_STOP, FAIL_CONTINUE, or BLOCKED.
```

## Governing decision

The frontend is done only when a fresh, independent evaluator can demonstrate
that the in-scope governed journeys are reachable, truthful, understandable,
and recoverable from observable application behavior. A route, component,
mockup, screenshot, API response, passing unit test, or agent claim is not by
itself proof that a user journey is complete.

This eval judges **frontend completion**, not complete SEA Forge system
correctness. It must distinguish:

- `INTEGRATED`: the frontend is exercised against the intended backend or a
  contract-faithful test environment.
- `MOCKED`: the frontend is exercised against deterministic mocks or fixtures.
- `STATIC_ONLY`: the evaluator can inspect source but cannot run and interact
  with the application.

`STATIC_ONLY` can never receive `PASS_STOP`. A mocked implementation may receive
`PASS_STOP` only with qualification `FRONTEND_COMPLETE_MOCKED`; it must not be
reported as end-to-end system completion.

## Evaluator role

You are an independent product evaluator, not the implementation agent. Your
job is to falsify the claim that the frontend is complete. Do not repair code,
fill gaps, reinterpret missing behavior charitably, or accept narration as
proof. Prefer observable evidence over inference. place your report in `.agents/reports/frontend-eval-<TIMESTAMP>.md` and the JSON result in `.agents/reports/frontend-eval-<TIMESTAMP>.json`.

You must:

1. Read the embedded epic before grading.
2. Build and run the application when possible.
3. Exercise the UI as each relevant actor.
4. Test happy paths, denials, failures, stale states, empty states, retries,
   cancellations, and recovery paths.
5. Reconcile every one of the 128 durable user stories to evidence.
6. Apply all 15 hard gates.
7. Produce a machine-readable result and a concise human report.
8. Stop after the verdict; do not propose optional polish as required work.

## Evidence rules

A claim may be marked `PASS` only when the evaluator records enough evidence
for another reviewer to reproduce it. Each passing claim needs at least:

- the story or gate ID;
- actor and precondition;
- route or entry point;
- actions performed;
- expected and observed result;
- one or more concrete evidence references, such as screenshot name, DOM text,
  browser trace, test output, console output, network call, event/record ID,
  or source location supporting the observed behavior.

Apply these rules strictly:

- Source code proves intent, not runtime behavior.
- A screenshot proves appearance, not interaction or state transition.
- A successful API response proves transport, not user comprehension.
- A disabled control without an explanation and lawful next step is incomplete.
- A button, menu item, or route that is inert, placeholder-only, or backed by a
  generic toast is not implemented.
- Hidden functionality that a representative user cannot discover does not
  satisfy the UX story.
- A generated artifact, command exit code, or agent statement does not prove
  settlement.
- The evaluator may not invent exclusions. Only exclusions explicitly supplied
  by the product owner are excluded from the denominator.
- “Backend not implemented” does not excuse deceptive frontend claims. The UI
  must label unavailable, simulated, stale, unsupported, or unknown states
  honestly.

## Story result vocabulary

Use exactly one result for every in-scope story:

- `PASS`: complete, reachable, truthful, and evidenced.
- `PARTIAL`: meaningful behavior exists, but a required state, explanation,
  transition, actor path, or evidence link is missing.
- `FAIL`: absent, broken, misleading, unreachable, or contradicted by runtime.
- `NOT_TESTED`: required evidence could not be obtained. Counts as zero and
  prevents `PASS_STOP`.
- `OUT_OF_SCOPE`: explicitly excluded by the product owner with a reason. The
  evaluator may challenge an exclusion that would make an epic completion claim
  misleading, but must not silently change it.

## Defect severity

- `P0`: violates a hard governance invariant, exposes secrets, performs or
  represents unauthorized side effects, conflates execution with settlement,
  permits self-certification, destroys history, or materially deceives the user.
- `P1`: breaks a representative end-to-end journey, prevents a required actor
  from acting, hides the reason or next lawful path, or makes evidence/audit
  reconstruction impractical.
- `P2`: incomplete edge state, weak feedback, inconsistent terminology,
  accessibility problem, or recoverable interaction defect.
- `P3`: non-blocking fit-and-finish issue. P3 items do not prevent `PASS_STOP`
  unless they collectively make the interface materially unusable.

---

# Part I — Hard gates

All hard gates are binary. One failed hard gate forces `FAIL_CONTINUE`,
regardless of numerical score.

## G1. Authority is visible before side effects

**Source invariant:** Authority precedes side effects.

Pass only if protected actions visibly preview the relevant actor, resource,
boundary, approval, sandbox/environment, and consequence before execution, and
if unauthorized or unresolved identity states cannot proceed through the UI.

Probe at least one command, agent, projection, transition, approval-sensitive,
and adoption action when those actions are in scope.

Primary stories: `2.2`, `2.5`, `2.6`, `6.8`, `6.9`, `8.2`, `8.4`, `9.1`,
`9.2`, `14.9`, `15.6`.

## G2. Execution and settlement remain distinct

Pass only if progress, termination, outputs, exit codes, generated files, and
agent narration are displayed separately from accepted, rejected, or escalated
settlement. A successful run must not automatically appear complete without its
criteria and settlement basis.

Primary stories: `9.3`, `9.8`, `9.9`, `12.1`, `12.3`, `12.4`, `12.5`.

## G3. Denial and escalation are governed outcomes

Pass only if denial, expiry, escalation, and blocked states use typed reason
codes or equivalent structured explanations, identify the governing reference
or boundary, show downstream effect, and present a lawful next step. A generic
error toast fails this gate.

Primary stories: `1.7`, `2.6`, `3.7`, `7.8`, `8.7`, `10.5`, `11.8`.

## G4. Important status resolves to source records

Pass only if readiness, case, run, approval, evidence, settlement, capability,
artifact, and assurance statuses expose their underlying typed record, event,
digest, evidence, or machine-readable source reference. Decorative status
badges without traceability fail.

Primary stories: `2.7`, `3.8`, `7.4`, `11.1`, `12.1`, `12.7`, `12.9`,
`12.10`, `14.5`, `16.8`.

## G5. Derived views disclose freshness, rebuildability, and assurance

Pass only if self-model, Thoth answers, memory, capability, projections,
indexes, and operational-health views visibly distinguish current, stale,
pending, degraded, unsupported, and unknown states where applicable and expose
rebuild or verification paths.

Primary stories: `1.6`, `3.9`, `13.3`, `13.7`, `16.1`, `16.2`, `16.4`,
`16.8`.

## G6. State vocabularies are not collapsed

Pass only if the UI preserves the epic's important distinctions, including:

- healthy / stale / degraded / unavailable / quarantined /
  integrity-pending / blocked;
- declared / installed / available / validated / demonstrated / degraded /
  unsupported / unknown;
- active / enabled / waiting / blocked / completed / rejected / future;
- attempted / demonstrated / proven / degraded / quarantined / contracted;
- execution termination versus settlement.

Primary stories: `1.6`, `3.2`, `4.4`, `4.5`, `4.7`, `7.3`, `12.4`, `13.4`,
`16.8`.

## G7. Blocked and failed states expose the next lawful path

Pass only if every sampled blocked, failed, stale, incompatible, denied,
quarantined, expired, or drifted state explains what happened, what was and was
not changed, and the permitted retry, replan, approval, revalidation, adoption,
endpoint, environment, remediation, or escalation path.

Primary stories: `1.7`, `4.8`, `5.6`, `7.8`, `8.7`, `11.8`, `14.3`, `14.9`,
`15.7`, `16.7`.

## G8. History is append-only from the user's perspective

Pass only if retries create new run episodes; approval decisions, cancellations,
replans, reopens, terminations, adoptions, and transitions preserve prior state
and attributable reasons; and canonical history is not editable in place.

Primary stories: `7.5`, `7.6`, `7.7`, `8.3`, `11.4`, `12.8`, `14.10`,
`15.6`.

## G9. Disclosure is constrained before retrieval

Pass only if Thoth, audit search, and memory recall visibly operate within actor
and disclosure boundaries before showing protected content, and their answers
state limitations and do not imply execution authority.

Primary stories: `3.7`, `3.8`, `3.9`, `12.9`, `13.1`, `13.2`.

## G10. Secret values never surface

Pass only if secret values are absent from records, forms after submission,
errors, logs, transcripts, evidence views, network-inspection surfaces exposed
by the application, and copied/exported content. Secret identifiers or redacted
references may be shown when useful; raw values may not.

Primary stories: `2.5`, `2.7`, `9.2`, `9.4`, `9.8`, `12.2`, `12.9`.

## G11. Agent work is visibly untrusted until evaluated

Pass only if external plans and agent outputs enter as proposals or evidence,
not authority or proof; the UI displays immutable criteria and evaluator status;
and “agent says done” cannot close a plan item or case.

Primary stories: `6.3`, `9.4`, `9.5`, `9.6`, `9.9`, `10.3`, `10.4`, `12.3`.

## G12. Required separation of duty is enforced and explained

Pass only if the UI excludes ineligible approvers, settlers, and promoters;
prevents self-approval or self-certification in sampled protected paths; and
shows why an actor is ineligible.

Primary stories: `2.3`, `8.4`, `10.6`, `12.5`, `12.6`, `14.8`.

## G13. Failure retains diagnostic evidence

Pass only if failures, timeouts, disconnects, cancellations, sandbox violations,
turn-cap exits, invalid projections, drift, failed probes, and interrupted runs
retain enough trace, scope, side-effect status, evidence, and recovery context to
understand and continue lawfully.

Primary stories: `4.6`, `4.8`, `5.2`, `5.6`, `5.7`, `9.8`, `11.4`, `11.6`,
`11.8`, `12.1`.

## G14. Projections and indexes never masquerade as truth

Pass only if generated projections, indexes, summaries, screenshots, and
rebuildable views are labeled as derived, linked back to authoritative records,
and quarantined or degraded when invalid or stale.

Primary stories: `5.7`, `12.10`, `13.3`, `13.7`, `14.1`, `14.3`, `14.4`,
`16.4`.

## G15. The UX prioritizes spendable affordances

Pass only if users can distinguish what is visible, reachable, permitted, and
settleable now from theoretical or merely installed capability, and if the
current case horizon and operational debt lead directly to available actions.

Primary stories: `3.6`, `4.7`, `4.8`, `7.3`, `7.8`, `10.2`, `16.7`.

---

# Part II — Representative scenario pack

Run all scenarios whose stories are in scope. A scenario passes only when every
required checkpoint passes. Use deterministic fixtures whenever production data
would make the result non-reproducible.

## S1. Fresh cell and readiness diagnosis

**Actors:** Cell Administrator, Operator
**Covers:** journeys 1, 2, 16

1. Open or initialize a cell.
2. Confirm cell identity and prevent accidental overwrite of existing history.
3. Observe startup checks for policy, identity, ledger, model, extensions,
   environments, server, and Genesis Self-Model.
4. Inject or select at least two non-healthy states, such as stale plus blocked
   or quarantined plus integrity-pending.
5. Follow the evidence and next corrective action for each state.

Required checkpoints: explicit cell context; distinguishable readiness states;
no protected work with unresolved identity; source evidence; affected
capabilities; lawful recovery action; honest assurance label.

## S2. Identity conflict, authority preview, denial, and escalation

**Actors:** Operator, Agent Sponsor, Approver
**Covers:** journeys 2, 8

1. Enter with a valid resolved identity and inspect principal, actor type, role,
   identity source, and active context.
2. trigger a missing or conflicting identity state.
3. Preview a protected action's required surfaces and boundaries.
4. Attempt the action without sufficient authority.
5. Inspect denial reason, matched policy or equivalent reference, downstream
   effect, and next action.
6. Resolve through an eligible approver; verify an ineligible/self approver is
   unavailable or rejected with explanation.

Required checkpoints: no anonymous fallback; immutable configuration context;
typed denial; governed approval; separation of duty; durable decision evidence.

## S3. Ask what SEA Forge is and what can actually be done

**Actors:** Operator, Auditor, External Agent
**Covers:** journeys 3, 4

1. Ask about current installation identity and architecture.
2. Ask whether one capability is declared, installed, available, validated,
   demonstrated, degraded, unsupported, or unknown.
3. Ask what an operation requires and what authority it needs.
4. Ask for current affordances.
5. Ask for the evidence behind one claim.
6. Ask about a failed or denied record.

Required checkpoints: one verified snapshot; freshness and assurance; omitted
claim classes/limitations; evidence links; no hidden-fact invention; knowledge
explicitly confers no execution authority; usable assets distinguished from
assets that merely exist.

## S4. Domain validation, model pinning, drift, and projection quarantine

**Actors:** Domain Author, Plan Author
**Covers:** journey 5

1. Select a valid `.sea` source set and inspect the resolved model.
2. Submit separate syntax, import, and semantic failures.
3. Pin exact model and source references to a plan.
4. Introduce source drift after pinning.
5. Attempt reuse, then follow revalidation or replan.
6. Run a valid projection and an invalid projection.

Required checkpoints: error layers remain distinct; references are selectable;
hashes/version/adapter are visible; drift blocks silent reuse; derived output is
hash-linked and limitation-labeled; invalid output is quarantined and cannot
activate downstream work.

## S5. Create, preview, and immutably commit a case

**Actors:** Case Owner, External Agent, Delivery Lead
**Covers:** journey 6 and parts of 7

1. Start cases from at least two sources: known intent or template, plus external
   proposal, ADLC/ODI, multi-agent topology, or spec-to-code.
2. Bind environment, evaluator, sandbox, endpoint, retention, and limits.
3. Preview stages, items, sentries, milestones, criteria, origins, models,
   delegates, boundaries, and likely approvals.
4. Correct an invalid or unaffordable plan.
5. Commit the case.
6. Verify later UI changes cannot rewrite the committed plan and criteria.

Required checkpoints: external proposal remains untrusted; unresolved semantic
refs cannot activate; full preview exists; commitment precedes side effects;
provenance and configuration digests remain inspectable.

## S6. Live case horizon, human work, retry, replan, and parking

**Actors:** Case Owner, Human Assignee, Approver
**Covers:** journeys 7, 8

1. Open a live case containing active, enabled, waiting, blocked, completed,
   rejected, and future work.
2. Inspect why two items changed state.
3. Complete a human task with required artifact/evidence.
4. Retry one item and verify a new run episode.
5. Add discretionary work through the governed path.
6. Replan or reopen with an attributable reason.
7. Put the case into an approval-wait or bounded-exhaustion state.

Required checkpoints: current horizon is legible; sentries/triggers/evidence are
traceable; retries append; human work becomes committed evidence; parked is not
failed or complete; next action is explicit.

## S7. Governed command and agent execution

**Actors:** Operator, Case Owner, Agent Sponsor
**Covers:** journey 9

1. Run one non-agent action under an explicit grant.
2. Configure an agent task with endpoint, instruction packet, context refs,
   expected artifacts, tools, model, caps, timeout, retention, continuation, and
   criteria.
3. Run one HTTP/ACP-style agent or a deterministic equivalent.
4. Observe progress and outputs.
5. Produce an executor success that fails a settlement criterion.
6. Produce one timeout, cancellation, endpoint failure, turn-cap exit, sandbox
   violation, or ACP disconnect.

Required checkpoints: actual boundary matches displayed boundary; credentials
remain secret; provider does not change governance semantics; narration is not
proof; success can settle rejected; every termination preserves evidence and
basis.

## S8. Concurrency, intervention, restart recovery, and lawful continuation

**Actors:** Operator, Case Owner
**Covers:** journey 11 and parts of 7/9

1. Run parallel sibling episodes.
2. Show shared capacity saturation separately from authority/dependency blocks.
3. Cancel one child run and verify siblings continue.
4. Simulate interruption or restart.
5. Identify active, interrupted, orphaned, and unsettled runs.
6. Resume or settle with explicit basis.
7. Reactivate an earlier ADLC stage after rejection or drift.

Required checkpoints: event stream is sourced from committed records; cancel
scope is precise; control action is durable; restart does not hide work; failure
shows side-effect status and lawful retry/replan/environment/endpoint/escalation
path.

## S9. Evidence, settlement, audit, and replay

**Actors:** Auditor, Reviewer, Operator
**Covers:** journey 12

1. Open one linked run record.
2. Compare every immutable criterion with its evidence and result.
3. Verify artifact/transcript identity, lineage, ownership/license where
   applicable, and policy-governed transcript exposure.
4. Inspect declarer standing, independence, reliability, policy snapshot, and
   evidence.
5. Inject invalid provenance or self-declaration.
6. Verify ledger inclusion/consistency and replay ordering.
7. Reach machine-readable source records from the human-readable view.

Required checkpoints: intent, authority, attempt, observation, settlement, and
promotion are reconstructable; weak proof is excluded; termination and
settlement are separate; no screenshot becomes proof substrate.

## S10. Governed memory and capability evolution

**Actors:** Operator, Auditor, Case Owner
**Covers:** journey 13

1. Recall memory under an actor boundary.
2. Inspect which recalled records influenced plan-time or run-time work.
3. Make the memory index stale or unavailable and use authoritative fallback.
4. Inspect attempted, demonstrated, proven, degraded, quarantined, and
   contracted capabilities.
5. Trace promotion and contraction to qualifying or disqualifying evidence.
6. Rebuild memory and capability views.

Required checkpoints: retrieval is constrained before disclosure; index failure
changes speed rather than truth; pre-contraction evidence remains audit-visible
but cannot restore the tier; routing and Thoth use the same capability record.

## S11. Projection, artifact maturity, export, import, and adoption

**Actors:** Domain Owner, Delivery Lead, Artifact Owner, Cell Administrator
**Covers:** journeys 14, 15

1. Run a deterministic governed projection.
2. Navigate representative spec-to-code stages.
3. Demonstrate generated output separately from runtime readiness.
4. Attempt an artifact transition with a missing gate, then satisfy it.
5. Attempt stage teleportation and verify rejection.
6. Export a verifiable bundle.
7. Import valid and tampered/incompatible bundles.
8. Verify imported history remains distinct and assets remain inert until local
   governed adoption.

Required checkpoints: full lineage and dimensional artifact identity; invalid
output quarantined; transition tokens preserve predecessor chain; failed import
is atomic; adoption is explicit and attributable; imported evidence does not
inflate local capability.

## S12. Operational health, change, and honest assurance

**Actors:** Cell Administrator, Auditor, Operator
**Covers:** journey 16

1. Change a release, extension, endpoint, environment, or toolchain input.
2. Validate or rebuild the self-model.
3. Make the rebuild fail and inspect the preserved stale snapshot.
4. Change extension/endpoint lifecycle while work is in flight.
5. Rebuild one derived store independently.
6. Inspect version skew and affected operations.
7. Open unresolved operational debt.

Required checkpoints: historical and in-flight meaning remains pinned; stale but
verifiable data is preserved; derived repair does not rewrite history; debt is
actionable; assurance labels appear only when their evidence requirements hold.

---

# Part III — Frontend quality gates

These do not replace epic semantics. They ensure the implemented journeys are
usable rather than merely present.

## Q1. Build and runtime integrity

- Production build succeeds.
- Test command succeeds, or missing test infrastructure is reported.
- No uncaught exception occurs in tested journeys.
- No repeated console error, hydration failure, broken route, or failed asset
  load materially affects use.
- Refresh and direct navigation preserve or safely reconstruct route state.

## Q2. Interaction completeness

For each tested state, verify loading, empty, populated, blocked/denied, error,
stale/degraded where relevant, and recovery. No in-scope primary control may be
placeholder-only, inert, or permanently disabled without explanation.

## Q3. Accessibility

At minimum:

- all primary journeys are keyboard operable;
- focus order and visible focus are usable;
- dialogs trap and restore focus correctly;
- controls have accessible names;
- status is not conveyed by color alone;
- validation and asynchronous errors are programmatically associated and
  understandable;
- major headings and landmarks form a coherent structure;
- no critical automated accessibility violations remain.

## Q4. Responsive and information-density behavior

Exercise at least a desktop viewport around `1440×900`, a compact laptop around
`1280×720`, and a narrow viewport around `390×844` unless the declared product
scope excludes narrow layouts. Critical actions and evidence links must remain
reachable; tables may adapt through scrolling or alternate layouts but may not
silently hide governance-critical fields.

## Q5. Terminology and visual truthfulness

Canonical terms, state labels, actor roles, evidence language, and capability
vocabulary must be consistent with the epic. Visual prominence must not imply a
stronger assurance, authority, completion, or capability state than the records
support.

---

# Part IV — Scoring and verdict

## Numerical score

Calculate only after hard gates:

- Story completion: 70 points.
  - `PASS = 1.0`, `PARTIAL = 0.5`, `FAIL = 0`, `NOT_TESTED = 0`.
  - Exclude only owner-declared `OUT_OF_SCOPE` stories.
  - Score = earned story points / possible story points × 70.
- Representative scenarios: 15 points.
  - Equal weight across applicable scenarios.
- Accessibility and interaction resilience: 10 points.
- Visual/terminology coherence: 5 points.

Do not use the numerical score to override a hard-gate failure.

## `PASS_STOP`

Return `PASS_STOP` only when all conditions hold:

1. Execution mode is `INTEGRATED` or `MOCKED`, not `STATIC_ONLY`.
2. All 15 hard gates pass.
3. No `P0` or `P1` defects remain.
4. Every in-scope story has a result; none is `NOT_TESTED`.
5. Every applicable representative scenario passes.
6. Total score is at least `92/100`.
7. Production build succeeds.
8. No uncaught runtime error occurs in the scenario pack.
9. No in-scope primary action is a placeholder, no-op, or misleading simulation.
10. Declared exclusions are explicit in the report.

Set qualification to:

- `FRONTEND_COMPLETE_INTEGRATED` for integrated execution.
- `FRONTEND_COMPLETE_MOCKED` for contract-faithful deterministic mocks.

After `PASS_STOP`, do not ask the implementation agent to continue polishing.
Any further work requires a new requirement, defect, or separately approved
quality objective.

## `FAIL_CONTINUE`

Return `FAIL_CONTINUE` when testing is possible but any `PASS_STOP` condition is
not met. Provide the smallest defect set that, if fixed, could change the next
verdict. Order by dependency and severity, not by screen.

## `BLOCKED`

Return `BLOCKED` only when a credible verdict cannot be produced because the
application cannot be built/run, required credentials or fixtures are absent,
the epic/source contract is unavailable, or the environment prevents meaningful
interaction. Do not use `BLOCKED` for ordinary product failures; those are
`FAIL_CONTINUE`.

---

# Part V — Required evaluation procedure

1. **Resolve inputs.** Record repo revision, environment, mode, commands, URL,
   identities, fixtures, and exclusions.
2. **Inventory the UI.** Identify routes, navigation, actor surfaces, major state
   stores, data adapters, and test assets. This inventory is orientation, not
   proof.
3. **Build and smoke test.** Run the production build and existing tests. Start
   the app and check direct navigation, refresh, and basic console/network health.
4. **Create the story matrix.** Copy the 128-story inventory from this document.
   Apply only declared exclusions.
5. **Run all hard-gate probes.** Record pass/fail and evidence for G1–G15.
6. **Run the scenario pack.** Execute S1–S12 as applicable, using multiple actors
   and negative-state fixtures.
7. **Grade every story.** Do not bulk-pass stories merely because a scenario
   worked. Each story needs its own evidence reference.
8. **Run quality gates.** Test accessibility, responsive behavior, error/loading/
   empty states, and terminology.
9. **Reconcile contradictions.** Runtime behavior outranks comments, design docs,
   TODOs, test names, and model narration. Report any misleading claim as a
   defect.
10. **Calculate score and verdict.** Apply the strict verdict rules.
11. **Emit both outputs.** First the concise human report, then the exact JSON
    object defined below.

---

# Part VI — Required output format

## Human report

Use this exact order:

```markdown
# SEA Forge Frontend Eval Result

Verdict: PASS_STOP | FAIL_CONTINUE | BLOCKED
Qualification: FRONTEND_COMPLETE_INTEGRATED | FRONTEND_COMPLETE_MOCKED | NONE
Score: <0-100>
Execution mode: INTEGRATED | MOCKED | STATIC_ONLY
Confidence: HIGH | MEDIUM | LOW
Evaluated revision: <revision or unknown>

## Decision basis
<3-8 sentences explaining why the verdict follows from the gates.>

## Hard gates
| Gate | Result | Evidence | Blocking defect |

## Scenario results
| Scenario | Result | Key evidence | Covered story IDs |

## Blocking defects
| Severity | ID | Story/gate | Reproduction | Expected | Observed | Required correction |

## Story coverage
| Story ID | Result | Evidence | Notes |

## Declared exclusions
| Story ID(s) | Owner-provided reason | Effect on claim |

## Untested or blocked evidence
<None, or exact missing input and why it prevents judgment.>

## Non-blocking observations
<Only P2/P3 issues. Do not turn optional polish into completion work.>
```

## Machine-readable result

Return one valid JSON object after the human report. Do not wrap it in comments.

```json
{
  "eval_protocol": "sea-forge-governed-workbench-frontend-v0.1",
  "verdict": "PASS_STOP | FAIL_CONTINUE | BLOCKED",
  "qualification": "FRONTEND_COMPLETE_INTEGRATED | FRONTEND_COMPLETE_MOCKED | NONE",
  "score": 0,
  "execution_mode": "INTEGRATED | MOCKED | STATIC_ONLY",
  "confidence": "HIGH | MEDIUM | LOW",
  "evaluated_revision": "string-or-unknown",
  "hard_gates": [
    {
      "id": "G1",
      "result": "PASS | FAIL | NOT_TESTED",
      "evidence": ["reproducible evidence reference"],
      "defect_ids": []
    }
  ],
  "scenarios": [
    {
      "id": "S1",
      "result": "PASS | PARTIAL | FAIL | NOT_APPLICABLE | NOT_TESTED",
      "covered_story_ids": ["1.1"],
      "evidence": ["reproducible evidence reference"]
    }
  ],
  "story_coverage": [
    {
      "id": "1.1",
      "result": "PASS | PARTIAL | FAIL | NOT_TESTED | OUT_OF_SCOPE",
      "evidence": ["reproducible evidence reference"],
      "notes": ""
    }
  ],
  "defects": [
    {
      "id": "SF-FE-001",
      "severity": "P0 | P1 | P2 | P3",
      "story_ids": ["9.9"],
      "gate_ids": ["G2", "G11"],
      "title": "concise defect title",
      "reproduction": ["step 1", "step 2"],
      "expected": "expected observable behavior",
      "observed": "observed behavior",
      "required_correction": "minimum outcome needed, not implementation prescription"
    }
  ],
  "declared_exclusions": [
    {
      "story_ids": ["string"],
      "reason": "owner-provided reason",
      "effect_on_claim": "string"
    }
  ],
  "blockers": [],
  "next_fix_set": ["defect IDs in dependency order"],
  "stop_reason": "why implementation must stop or continue"
}
```

---

# Part VII — Few-shot grading examples

These examples are binding interpretations of the protocol.

## Example A — Executor says success, no settlement

**Observed:** An agent run ends with a green “Success” badge and the case item
moves to Completed. The UI shows generated files and the agent's final message,
but no immutable criteria comparison, evaluator result, declaration standing,
or settlement record.

**Grade:** `FAIL`; defects are `P0`; G2 and G11 fail. Relevant stories include
`9.9`, `12.3`, `12.4`, and `12.5`. Exit or narration has been treated as proof.

## Example B — Denial is a generic error

**Observed:** A user without authority clicks “Run.” The request is blocked, but
the only feedback is “Something went wrong.” No reason code, policy reference,
affected boundary, downstream effect, or approval/escalation path is available.

**Grade:** `FAIL`; defect is at least `P1` and may be `P0` if the interface is
misleading. G3 and G7 fail. A technically blocked request does not satisfy the
UX requirement for a governed denial.

## Example C — Imported asset is inert until adoption

**Observed:** A valid bundle imports atomically. Imported cases and records are
visibly identified as external. Its template is viewable but disabled for new
cases. An eligible administrator can inspect provenance and compatibility,
request local adoption, approve it through the governed path, and then use the
adopted local version. A tampered bundle leaves the cell unchanged.

**Grade:** `PASS` for the covered `15.x` stories, assuming reproducible evidence
exists. The UI preserves the transport/adoption boundary.

## Example D — Route and components exist, controls do nothing

**Observed:** Source inspection finds pages for approvals, capabilities, and
artifacts. The running app renders them, but primary controls are disabled with
no explanation or produce placeholder toasts. Tests only assert that headings
exist.

**Grade:** `FAIL`, not `PARTIAL`, for the affected primary actions. Presence is
not a completed journey. Do not infer future wiring from component names or
TODOs.

## Example E — Complete deterministic mocked frontend

**Observed:** The evaluator can run all applicable scenarios with deterministic
fixtures, including denial, stale, failure, cancellation, retry, settlement,
contraction, quarantine, and adoption. Network calls are mocked but conform to a
versioned contract. Every status is explicitly labeled simulated/test data. All
hard gates pass and score is 96.

**Grade:** `PASS_STOP` with qualification `FRONTEND_COMPLETE_MOCKED`. Do not call
it integrated or claim the backend's governance semantics are proven.

---

# Part VIII — Implementer/evaluator stop loop

Use this protocol outside the evaluator when an implementation agent is doing
the work:

1. Give the implementation agent the epic and current defect set.
2. Let it implement only until the defect set and in-scope stories are addressed.
3. Start a fresh evaluator session with this document, the repo, and the running
   application. Do not include the implementer's self-assessment or chain of
   thought.
4. On `FAIL_CONTINUE`, send the evaluator's `next_fix_set` and reproducible
   defects back to the implementer. Do not send vague requests to “make it more
   polished.”
5. On `BLOCKED`, resolve only the missing environment/input and rerun.
6. On `PASS_STOP`, terminate the implementation loop. Further work requires a
   new requirement or separately approved quality target.

This separation is essential. A self-evaluating coding agent is biased toward
recognizing its own intended structure and under-testing missing behavior.

---

# Part IX — Canonical 128-story inventory

The evaluator must include one result row for every story below.

## Journey 1 — Open, initialize, and establish cell readiness

- `1.1` — Open or create a cell
- `1.2` — Recognize existing history
- `1.3` — Migrate compatible legacy history
- `1.4` — Validate startup requirements
- `1.5` — Validate and realize the Genesis Self-Model
- `1.6` — See readiness and assurance
- `1.7` — See the next corrective action

## Journey 2 — Establish actor identity, sponsorship, and authority context

- `2.1` — Confirm the acting identity
- `2.2` — Resolve identity failures safely
- `2.3` — Sponsor automated actors
- `2.4` — Inspect the active policy state
- `2.5` — Preview authority requirements
- `2.6` — Understand denial or escalation
- `2.7` — Preserve exact authorized configuration

## Journey 3 — Ask SEA Forge what it is and what is currently possible

- `3.1` — Ask about SEA Forge identity and architecture
- `3.2` — Ask about a capability
- `3.3` — Ask what an operation requires
- `3.4` — Ask what authority an operation requires
- `3.5` — Ask about projection and environment support
- `3.6` — Ask for current affordances
- `3.7` — Ask why something failed or was denied
- `3.8` — Ask for the evidence behind a claim
- `3.9` — Receive a bounded and reproducible answer

## Journey 4 — Discover and prepare usable operating assets

- `4.1` — Browse plan templates
- `4.2` — Browse ADLC, ODI, and orchestration templates
- `4.3` — Browse environment contracts
- `4.4` — Browse extensions and projection adapters
- `4.5` — Browse agent endpoints
- `4.6` — Probe an endpoint
- `4.7` — Browse demonstrated capabilities
- `4.8` — See why an asset is not usable

## Journey 5 — Supply and validate executable domain meaning

- `5.1` — Select or supply a `.sea` source set
- `5.2` — Validate syntax, imports, and semantics
- `5.3` — Inspect the validated domain model
- `5.4` — Pin the model to a plan
- `5.5` — Validate every semantic reference
- `5.6` — Detect and respond to model drift
- `5.7` — Project and verify the model

## Journey 6 — Create a case from the appropriate work source

- `6.1` — Start from a known intent
- `6.2` — Start from a versioned template
- `6.3` — Start from an external plan proposal
- `6.4` — Start an ADLC or ODI-grounded development case
- `6.5` — Start a multi-agent topology
- `6.6` — Start a governed spec-to-code pipeline
- `6.7` — Bind environments, evaluators, and execution modes
- `6.8` — Preview the complete case before commitment
- `6.9` — Commit the case immutably

## Journey 7 — Understand and navigate the live case

- `7.1` — See case purpose and governing outcome
- `7.2` — See stages, items, milestones, and sentries
- `7.3` — See the current case horizon
- `7.4` — Understand every state transition
- `7.5` — Follow distinct run episodes
- `7.6` — Add newly discovered work
- `7.7` — Reopen, replan, or terminate deliberately
- `7.8` — Distinguish parked from failed

## Journey 8 — Resolve approvals and human work

- `8.1` — View one approval and human-task inbox
- `8.2` — Inspect the full decision context
- `8.3` — Approve or reject with evidence
- `8.4` — Enforce approval authority and separation of duty
- `8.5` — Complete a human plan item
- `8.6` — Resolve ACP permission requests
- `8.7` — Handle denial and expiry predictably

## Journey 9 — Execute governed work and delegate bounded agent labor

- `9.1` — Execute only after authority
- `9.2` — Use the granted sandbox and environment
- `9.3` — Follow non-agent execution
- `9.4` — Configure an agent task
- `9.5` — Run HTTP or ACP agents under the same fabric
- `9.6` — Run SWE_SEED-harnessed coding agents
- `9.7` — Monitor and intervene in agent dialogue
- `9.8` — Preserve evidence from every termination
- `9.9` — Evaluate outcomes rather than claims

## Journey 10 — Let Thoth propose bounded next work

- `10.1` — Invoke a sponsored manager loop
- `10.2` — See the deterministic case judgment
- `10.3` — Review the bounded proposal
- `10.4` — Admit the proposal through normal governance
- `10.5` — Stop or escalate predictably
- `10.6` — Prevent Thoth self-certification

## Journey 11 — Monitor, control, resume, and recover operations

- `11.1` — Watch the live event stream
- `11.2` — See parallel work and capacity
- `11.3` — Receive actionable notifications
- `11.4` — Cancel scoped work
- `11.5` — Resume parked cases
- `11.6` — Recover after interruption
- `11.7` — Reactivate earlier developmental work
- `11.8` — Create a new lawful path after failure

## Journey 12 — Inspect evidence, settlement, and audit truth

- `12.1` — Inspect one linked run record
- `12.2` — Verify artifact and transcript identity
- `12.3` — Compare criteria with evidence
- `12.4` — Distinguish execution termination from settlement
- `12.5` — Inspect declaration standing and reliability
- `12.6` — Detect invalid provenance or self-declaration
- `12.7` — Verify ledger inclusion and consistency
- `12.8` — Replay ordered case execution
- `12.9` — Inspect authority and disclosure history
- `12.10` — Access machine-readable source records

## Journey 13 — Recall developmental memory and use demonstrated capability

- `13.1` — Recall relevant memory under authority
- `13.2` — See which memory influenced work
- `13.3` — Preserve truth when the index is stale
- `13.4` — Inspect a capability record
- `13.5` — Understand variation, recovery, and burden
- `13.6` — Understand promotion or contraction
- `13.7` — Rebuild memory and capability views
- `13.8` — Use capability in routing and Thoth answers

## Journey 14 — Project specifications and mature governed artifacts

- `14.1` — Run a governed projection
- `14.2` — Navigate the spec-to-code pipeline
- `14.3` — Quarantine invalid generated output
- `14.4` — Distinguish generated output from runtime readiness
- `14.5` — Browse artifact identity and lineage
- `14.6` — Synthesize cognitive material
- `14.7` — Productize intellectual property
- `14.8` — Capitalize a product
- `14.9` — Preview and satisfy stage gates
- `14.10` — Verify TransitionTokens and prevent teleportation

## Journey 15 — Share and adopt governed assets across cells

- `15.1` — Export a verifiable cell bundle
- `15.2` — Import a bundle atomically
- `15.3` — Keep imported history distinct
- `15.4` — Keep imported assets inert
- `15.5` — Review provenance and compatibility
- `15.6` — Adopt an imported asset through governance
- `15.7` — Reject unsafe or incompatible imports

## Journey 16 — Maintain integrity, freshness, and operational health

- `16.1` — Validate or rebuild the self-model
- `16.2` — Preserve stale but verifiable snapshots
- `16.3` — Manage extension and endpoint lifecycle
- `16.4` — Rebuild derived stores independently
- `16.5` — Inspect version skew and compatibility
- `16.6` — Protect in-flight work during change
- `16.7` — Find unresolved operational debt
- `16.8` — See honest assurance labels

---

# Part X — Canonical source epic

The following epic is the authoritative semantic source. If an evaluator rule
appears to conflict with a product requirement, preserve the epic requirement
and report the apparent conflict rather than weakening the product contract.

# Epic: SEA Forge Governed Workbench UX

**Status:** Draft v0.1
**Epic type:** Umbrella product epic / end-to-end story map
**Source scope:** SEA Forge minimum lifecycle, full governed substrate, ADLC/ODI self-model and Thoth, and governed agent orchestration.

## Epic outcome

A user can move from a fresh or existing SEA Forge cell to a settled, evidence-linked, reusable capability without guessing:

```text
open the cell
→ establish trust and readiness
→ understand current affordances
→ define the domain and desired outcome
→ create and commit a governed case
→ authorize and execute human, command, projection, or agent work
→ monitor, intervene, and recover
→ inspect evidence and settlement
→ reuse what was proven as memory, capability, projection, or governed artifact
```

The UX must preserve the system's central distinctions:

- authority before side effects;
- execution before settlement, but execution never equal to settlement;
- agent narration as input, never proof;
- declared, observed, and demonstrated state as separate views;
- denial as a governed outcome, not a generic error;
- ledger records as truth and every UI state as a rebuildable view;
- a capability as repeated settlement under variation, not a single success;
- an affordance as a reachable and governable path, not merely a visible feature.

## Primary actors

- **Operator / Case Owner**
- **Approver / Human Task Assignee**
- **Domain or Plan Author**
- **Agent Sponsor**
- **Auditor / Reviewer**
- **Cell Administrator**
- **External Agent or Integration**

## Numbering contract

- `X` = durable journey area.
- `X.Y` = durable user story.
- Later scenarios, acceptance criteria, UI states, and edge cases use `X.Y.1`, `X.Y.2`, and so on.
- New stories are appended within their journey area; existing story IDs should not be renumbered merely because the implementation changes.

---

## 1. Open, initialize, and establish cell readiness

**Settlement target:** The user enters a known workspace whose identity, configuration, semantic self-model, and integrity state are explicit before work begins.

### 1.1 Open or create a cell

As a cell administrator, I want to select an existing SEA Forge root or initialize a new one so that all governed records belong to an explicit cell.

### 1.2 Recognize existing history

As a returning user, I want SEA Forge to detect the existing version, cell identity, cases, and ledgers so that initialization cannot overwrite prior governed work.

### 1.3 Migrate compatible legacy history

As a cell administrator, I want to migrate older records into the current ledger without re-keying them so that historical evidence remains attributable and verifiable.

### 1.4 Validate startup requirements

As an operator, I want policy, identity, ledger, model, extension, environment, and server configuration checked before work begins so that invalid configuration cannot produce partial side effects.

### 1.5 Validate and realize the Genesis Self-Model

As an operator, I want SEA Forge to validate its bundled system and ADLC/ODI models and create a release-and-cell snapshot so that self-knowledge refers to a precise installation state.

### 1.6 See readiness and assurance

As an operator, I want one readiness view that distinguishes healthy, stale, degraded, unavailable, quarantined, integrity-pending, and blocked components so that I know what this cell can safely do now.

### 1.7 See the next corrective action

As an operator, I want every readiness problem to identify its evidence, affected capabilities, and lawful next step so that a limitation becomes a navigable recovery path.

---

## 2. Establish actor identity, sponsorship, and authority context

**Settlement target:** Every protected action is attributable to a resolved actor operating under visible boundaries.

### 2.1 Confirm the acting identity

As a user, I want to see the resolved principal, actor type, role, identity source, and active cell or case context so that I know who SEA Forge believes is acting.

### 2.2 Resolve identity failures safely

As an operator, I want missing or conflicting identity to halt protected work with a typed explanation so that SEA Forge never silently falls back to anonymous authority.

### 2.3 Sponsor automated actors

As an agent sponsor, I want to bind an automated actor to an eligible human or service sponsor so that privileged automated work remains accountable.

### 2.4 Inspect the active policy state

As an operator, I want to see the active policy bundle, version, digest, assurance, and reload state so that I know which governance contract controls the work.

### 2.5 Preview authority requirements

As a case owner, I want to see the action surfaces, resources, sandbox, network, secret, approval, and separation-of-duty requirements a proposed case may trigger so that blockers are visible before launch.

### 2.6 Understand denial or escalation

As an operator, I want denied and escalated decisions to show reason codes, matched policy references, boundaries, and required next steps so that governance outcomes are actionable.

### 2.7 Preserve exact authorized configuration

As an auditor, I want each action bound to immutable policy, identity, endpoint, environment, model, and parameter digests so that later configuration changes cannot alter what was authorized.

---

## 3. Ask SEA Forge what it is and what is currently possible

**Settlement target:** The user receives grounded, disclosure-controlled answers from one verified self-model snapshot.

### 3.1 Ask about SEA Forge identity and architecture

As an operator or external agent, I want to ask what SEA Forge is and how the current installation is structured so that I do not have to infer the system from code or documentation.

### 3.2 Ask about a capability

As an operator, I want to ask whether a capability is declared, installed, available, validated, demonstrated, degraded, unsupported, or unknown so that design claims are not confused with operational proof.

### 3.3 Ask what an operation requires

As an operator, I want to ask for the inputs, toolchains, environments, authority, sandbox, evidence, and settlement requirements of an operation so that I can form a spendable path.

### 3.4 Ask what authority an operation requires

As an operator, I want to ask which authority surfaces, grants, approvals, and sponsors govern an operation so that permission requirements are visible before an attempt.

### 3.5 Ask about projection and environment support

As a domain or plan author, I want to ask whether a target projection, environment, evaluator, toolchain, or sandbox is supported and available in this cell so that I do not plan against missing substrate.

### 3.6 Ask for current affordances

As an operator or agent, I want to ask which actions are currently visible, reachable, permitted, and settleable in my context so that SEA Forge surfaces paths rather than an undifferentiated feature catalog.

### 3.7 Ask why something failed or was denied

As an operator, I want a recorded failure or denial explained from its own authority, trace, evidence, settlement, and disclosure records so that explanation does not invent hidden facts.

### 3.8 Ask for the evidence behind a claim

As an auditor, I want a claim linked to its snapshot, model, projection, capability record, settlements, and evidence so that I can verify it independently.

### 3.9 Receive a bounded and reproducible answer

As a user of Thoth, I want every answer to disclose freshness, assurance, limitations, omitted claim classes, and that knowledge confers no execution authority so that I know exactly what the answer does and does not prove.

---

## 4. Discover and prepare usable operating assets

**Settlement target:** The user can distinguish assets that merely exist from those that are locally usable and demonstrated.

### 4.1 Browse plan templates

As a case owner, I want to inspect built-in, local, imported, and adopted templates with versions, parameters, stages, sentries, and criteria so that I can select a durable process asset.

### 4.2 Browse ADLC, ODI, and orchestration templates

As a case owner, I want to compare ADLC, ODI-grounded ADLC, sequential-agent, and concurrent-agent templates so that I can choose the control shape that matches the work.

### 4.3 Browse environment contracts

As an operator, I want to inspect environments, supplied tools, evaluators, versions, and compatibility separately from sandbox isolation so that I understand both execution content and security posture.

### 4.4 Browse extensions and projection adapters

As a cell administrator, I want to inspect installed, active, disabled, incompatible, deprecated, and quarantined extensions and adapters so that extension state is explicit.

### 4.5 Browse agent endpoints

As an operator, I want to inspect HTTP and ACP agent endpoints with provider, model, connection mode, configuration digest, and evidence-derived status so that I can select a governed delegate.

### 4.6 Probe an endpoint

As an operator, I want to run an authority-gated endpoint probe that records compatibility, latency, destination, and typed failure evidence so that availability is demonstrated rather than asserted.

### 4.7 Browse demonstrated capabilities

As an operator, I want to inspect capability records with their variation, recovery, reliability, regression, and supporting settlements so that routing decisions can use proven history.

### 4.8 See why an asset is not usable

As an operator, I want missing dependencies, disabled policies, stale snapshots, incompatible versions, failed probes, or absent evidence displayed with the affected operations so that unusable assets do not look ready.

---

## 5. Supply and validate executable domain meaning

**Settlement target:** Plans and actions bind to a validated, hash-pinned semantic world before they can activate.

### 5.1 Select or supply a `.sea` source set

As a domain or plan author, I want to choose an existing validated model or supply a new `.sea` model and imports so that work is grounded in explicit domain meaning.

### 5.2 Validate syntax, imports, and semantics

As a domain author, I want parsing, import resolution, concept identity, policy, unit, relation, and semantic failures reported separately so that I can correct the actual layer that failed.

### 5.3 Inspect the validated domain model

As a plan author, I want to browse the model namespace, version, source hashes, entities, resources, flows, policies, metrics, relations, mappings, and projections so that references can be selected without guessing.

### 5.4 Pin the model to a plan

As a plan author, I want the plan to record the exact DomainModelRef, concept refs, source hashes, and adapter version so that the plan's meaning is reproducible.

### 5.5 Validate every semantic reference

As a case owner, I want every plan item, criterion, desired outcome, environment, and projection reference resolved before activation so that unresolved meaning cannot enter authority or execution.

### 5.6 Detect and respond to model drift

As a case owner, I want source drift to block silent reuse and offer an authority-checked revalidation or replan path so that changed meaning creates a new explicit settlement path.

### 5.7 Project and verify the model

As a domain author, I want to create supported projections with declared limitations, deterministic parameters, validation, hashes, and quarantine for invalid output so that projections remain rebuildable rather than new truth.

---

## 6. Create a case from the appropriate work source

**Settlement target:** The user creates an ordinary governed CasePlan regardless of whether the starting source was an intent, template, external proposal, or development methodology.

### 6.1 Start from a known intent

As an operator, I want to submit a supported intent and receive the deterministic plan derived from it so that simple work has a low-burden entry path.

### 6.2 Start from a versioned template

As a case owner, I want to instantiate a named template with typed parameters so that repeatable work begins from a durable process asset.

### 6.3 Start from an external plan proposal

As an operator or external agent, I want to submit a plan as untrusted input so that generated plans can enter SEA Forge without receiving implicit authority.

### 6.4 Start an ADLC or ODI-grounded development case

As an agent-development or product lead, I want to create a sentry-driven ADLC case and optionally bind its criteria to ODI desired outcomes so that development follows a replayable control loop with outcome provenance.

### 6.5 Start a multi-agent topology

As a case owner, I want to create sequential or concurrent agent work from built-in topology templates so that delegation order and rollup behavior are explicit.

### 6.6 Start a governed spec-to-code pipeline

As a delivery lead, I want to create a case spanning specification, `.sea`, semantic graph, manifest, generated contracts, last-mile runtime, and acceptance so that each transformation settles independently.

### 6.7 Bind environments, evaluators, and execution modes

As a case owner, I want each relevant plan item bound to its environment, evaluator, sandbox class, endpoint, retention, and execution limits so that dependencies and payment boundaries are declared up front.

### 6.8 Preview the complete case before commitment

As a case owner, I want to preview stages, items, sentries, milestones, criteria, origins, models, environments, delegates, boundaries, and likely approvals so that invalid or unaffordable plans are caught early.

### 6.9 Commit the case immutably

As a case owner, I want the accepted plan, parameters, criteria, provenance, configuration digests, and model refs committed before side effects so that later evidence proves what was actually intended.

---

## 7. Understand and navigate the live case

**Settlement target:** At any moment the user can see what is active, what is blocked, why the state changed, and which next actions are currently spendable.

### 7.1 See case purpose and governing outcome

As a case owner, I want the case to show its intent, job or requirement origin, desired outcome, owner, model, and completion rule so that its direction remains explicit.

### 7.2 See stages, items, milestones, and sentries

As a case owner, I want the case structure to show required, optional, discretionary, human, command, projection, transition, and agent work plus the events that activate it so that non-linear flow is understandable.

### 7.3 See the current case horizon

As an operator, I want active, enabled, waiting, blocked, completed, rejected, and future work separated so that I know what can happen now.

### 7.4 Understand every state transition

As an operator, I want an item or case state to show the triggering event, satisfied or unmet sentry, evidence, authority decision, and settlement so that state never appears arbitrary.

### 7.5 Follow distinct run episodes

As an operator, I want every activation or retry represented as a new immutable run linked to its plan item and prior attempts so that repetition never overwrites history.

### 7.6 Add newly discovered work

As an authorized operator, I want to add a discretionary item through the same validation, criteria-provenance, and authority path as planned work so that knowledge work can adapt safely.

### 7.7 Reopen, replan, or terminate deliberately

As an authorized operator, I want to reopen, replan, or terminate a case with an attributable reason and preserved history so that lifecycle changes remain governed events.

### 7.8 Distinguish parked from failed

As a case owner, I want a case with no executable work, unresolved approval, or bounded manager exhaustion shown as parked or awaiting action rather than falsely failed or complete.

---

## 8. Resolve approvals and human work

**Settlement target:** Human judgment enters the case as authority-checked evidence rather than an off-system conversation.

### 8.1 View one approval and human-task inbox

As an approver or assignee, I want pending approvals and active human tasks across open cases in one queue so that required human action is discoverable.

### 8.2 Inspect the full decision context

As an approver, I want to see the actor, action, resource, purpose, evidence, requested boundaries, expiry, policy reason, and downstream effect so that I can make an informed decision.

### 8.3 Approve or reject with evidence

As an authorized approver, I want to approve or reject with an optional note so that the decision and rationale become durable evidence.

### 8.4 Enforce approval authority and separation of duty

As an auditor, I want unauthorized approvers and actors excluded by authorship, proposal, execution, settlement, or promotion rules prevented from resolving the request so that approval cannot become self-certification.

### 8.5 Complete a human plan item

As an assigned human actor, I want to complete a human task with the required artifact, evidence, or note so that case sentries can reevaluate from a committed event.

### 8.6 Resolve ACP permission requests

As an approver, I want agent permission requests mapped into the same authority and approval model so that CLI agents have no parallel access channel.

### 8.7 Handle denial and expiry predictably

As a case owner, I want denied or expired approvals to show their effect on the live session, run, plan item, and case so that the next path is explicit.

---

## 9. Execute governed work and delegate bounded agent labor

**Settlement target:** Every side effect occurs inside the granted boundary, produces evidence, and proceeds to settlement regardless of executor type.

### 9.1 Execute only after authority

As an operator, I want commands, files, APIs, projections, artifact transitions, recalls, and agent calls to begin only after their exact authority decision exists so that no UI path races ahead of governance.

### 9.2 Use the granted sandbox and environment

As an operator, I want execution to use exactly the granted isolation class, network posture, workspace, environment, tools, limits, and secrets so that the displayed boundary is the real boundary.

### 9.3 Follow non-agent execution

As an operator, I want bounded progress, outputs, evaluator status, termination reason, and captured artifacts visible without presenting logs or exit code as settlement so that execution is observable but not overstated.

### 9.4 Configure an agent task

As a case owner, I want to choose an eligible endpoint and inspect the instruction packet, context refs, expected artifacts, tools, model, turn and token caps, timeout, transcript retention, continuation behavior, and criteria so that delegation has a complete job contract.

### 9.5 Run HTTP or ACP agents under the same fabric

As an operator, I want OpenAI-compatible, Anthropic-compatible, and ACP-resident agents executed as ordinary governed run episodes so that provider choice does not change authority, evidence, or settlement semantics.

### 9.6 Run SWE_SEED-harnessed coding agents

As an operator, I want route and proof artifacts harvested from SWE_SEED-projected hosts and linked to the delegation so that harness evidence has stronger standing than chat narration.

### 9.7 Monitor and intervene in agent dialogue

As an operator, I want to see bounded turn, token, streaming, permission, and continuation state and be able to cancel one delegation without cancelling siblings so that cost and risk remain controllable.

### 9.8 Preserve evidence from every termination

As an auditor, I want success, denial, timeout, sandbox violation, turn-cap exit, cancellation, endpoint failure, and ACP disconnect to preserve trace, artifacts, transcript summary or permitted transcript, and settlement basis so that failure cannot erase the record.

### 9.9 Evaluate outcomes rather than claims

As a case owner, I want command and agent outputs evaluated against the immutable criteria and required declarations so that “the executor said done” never closes the work.

---

## 10. Let Thoth propose bounded next work

**Settlement target:** Thoth can help move a case from stalled to progressing without acquiring unbounded planning, authority, or settlement power.

### 10.1 Invoke a sponsored manager loop

As an operator or agent sponsor, I want to invoke Thoth for a specific case and bounded iteration count so that management is explicit rather than a hidden daemon.

### 10.2 See the deterministic case judgment

As a case owner, I want each manager iteration to classify the case as satisfied, progressing, stalled, or blocked from committed records so that its response is explainable.

### 10.3 Review the bounded proposal

As a case owner, I want a proposed agent task linked to a versioned catalog entry or case-declared discretionary template with its criteria, limits, endpoint, and `proposed_by` identity so that Thoth cannot invent unconstrained work.

### 10.4 Admit the proposal through normal governance

As an approver or case owner, I want Thoth proposals validated, authorized, and added through the same plan-mutation path as human proposals so that management has no private control channel.

### 10.5 Stop or escalate predictably

As a case owner, I want Thoth to stop when the case is satisfied and park plus escalate when its iteration grant is exhausted so that bounded management cannot loop indefinitely.

### 10.6 Prevent Thoth self-certification

As an auditor, I want immutable provenance to prevent Thoth from approving, settling, or promoting its own claims and proposals so that the manager remains separate from judgment.

---

## 11. Monitor, control, resume, and recover operations

**Settlement target:** Work remains visible and controllable through concurrency, interruption, cancellation, drift, and failure.

### 11.1 Watch the live event stream

As an operator, I want an ordered, filterable stream of case, run, authority, approval, evidence, and settlement events sourced from committed records so that live monitoring is not a second truth channel.

### 11.2 See parallel work and capacity

As an operator, I want concurrent episodes, sibling independence, rollup relationships, and shared concurrency saturation displayed separately from authority or dependency blocks so that scheduling is comprehensible.

### 11.3 Receive actionable notifications

As an operator or approver, I want notifications for approvals, failures, cancellations, and completed runs linked to the exact governed record so that alerts lead to the correct action.

### 11.4 Cancel scoped work

As an operator, I want cancellation to name the affected run, preserve sibling work, and create a durable control and settlement record so that intervention has predictable scope.

### 11.5 Resume parked cases

As an operator, I want cases parked for approval or other resumable conditions to continue from their committed state after resolution so that work does not restart from scratch.

### 11.6 Recover after interruption

As an operator, I want restart recovery to identify active, interrupted, orphaned, and unsettled runs and either resume or settle them with explicit basis so that process death cannot hide work.

### 11.7 Reactivate earlier developmental work

As an ADLC case owner, I want rejected simulation, drift, failed activation, or remediation evidence to reactivate the appropriate earlier stage so that development behaves as a cybernetic loop rather than a waterfall.

### 11.8 Create a new lawful path after failure

As a case owner, I want a typed failure to identify the failed boundary, side-effect status, missing condition, and permitted retry, replan, environment, endpoint, or escalation path so that failure closes diagnostically.

---

## 12. Inspect evidence, settlement, and audit truth

**Settlement target:** The user can prove what was intended, authorized, attempted, observed, accepted, and promoted.

### 12.1 Inspect one linked run record

As an operator or auditor, I want a single view linking the plan item, origin, criteria, authority, trace, evidence, artifacts, transcript, settlement, declarations, and semantic envelope so that I do not manually reconstruct the episode.

### 12.2 Verify artifact and transcript identity

As an auditor, I want captured bytes, hashes, descriptors, ownership, license, lineage, transcript summary, sealed or full transcript, and evidence refs verified according to policy so that evidence identity is reproducible.

### 12.3 Compare criteria with evidence

As a reviewer, I want each immutable settlement criterion paired with the evidence that passed, failed, or remained unavailable so that the settlement basis is inspectable.

### 12.4 Distinguish execution termination from settlement

As an operator, I want termination reason and accepted, rejected, or escalated settlement shown separately so that cancellation, turn-cap exit, or exit code cannot define outcome by itself.

### 12.5 Inspect declaration standing and reliability

As an auditor, I want each settlement declaration to show declarer identity, standing, independence, reliability weight, policy snapshot, criteria, and evidence so that strong and weak settlement are not conflated.

### 12.6 Detect invalid provenance or self-declaration

As an auditor, I want mismatched criteria, broken origin chains, missing evidence, unauthorized declarations, and self-certification excluded from qualifying capability weight with a clear reason so that weak proof cannot silently promote claims.

### 12.7 Verify ledger inclusion and consistency

As an auditor, I want to verify ledger chains, checkpoints, witness receipts, record inclusion, and consistency between checkpoints without mutating history so that integrity is independently testable.

### 12.8 Replay ordered case execution

As an auditor, I want to replay ledgered case and parallel dispatch ordering so that the sequence behind a result can be reproduced.

### 12.9 Inspect authority and disclosure history

As an auditor, I want to search authority decisions, approvals, ACP permissions, Thoth questions, disclosure plans, answers, denials, and exact configuration digests so that governance has no invisible channel.

### 12.10 Access machine-readable source records

As an auditor or integrator, I want every human-readable status and report linked to its typed records and rebuild state so that screenshots never become the proof substrate.

---

## 13. Recall developmental memory and use demonstrated capability

**Settlement target:** Settled history improves future work without allowing indexes or summaries to replace source evidence.

### 13.1 Recall relevant memory under authority

As an operator, I want to search memory by content, kind, entity, process, session, result, and case while access is constrained before retrieval so that prior experience can inform current work safely.

### 13.2 See which memory influenced work

As an auditor, I want plan-time and run-time recalls linked to the resulting plan, decision, and settlement so that hidden context cannot shape governed execution.

### 13.3 Preserve truth when the index is stale

As an operator, I want recall to fall back to authoritative records when its rebuildable index is missing or stale so that indexing failure changes speed rather than truth.

### 13.4 Inspect a capability record

As an operator, I want to see attempted, demonstrated, proven, degraded, quarantined, or contracted capability status with its qualifying settlements so that capability is not inferred from output volume.

> **Canonical capability-state vocabulary (§7.3):** `attempted →
> demonstrated → proven`, plus `degraded`, `quarantined`, and `contracted`.
> `contracted` is a first-class state: a capability that was promoted to a
> higher tier and then narrowed by a recorded contraction (lost qualifying
> evidence, regression, policy exclusion, or scope reduction). Semantics:
> the prior proven/demonstrated evidence is retained for audit; the current
> effective tier is the contracted one; transitions out of `contracted`
> require fresh qualifying settlements (not reuse of the pre-contraction
> evidence). Evidence requirements mirror promotion: a contraction MUST cite
> the contraction reason, the triggering settlement/regression record, and
> the new effective scope. All capability-status references in this epic
> MUST use this vocabulary; earlier drafts that used `contracted` without
> defining it are resolved by this definition.

### 13.5 Understand variation, recovery, and burden

As an operator, I want capability views to show relevant variation, recovery, reliability, regression, and orchestration burden so that narrow or brittle success is visible.

### 13.6 Understand promotion or contraction

As an operator, I want capability change linked to the exact promotion policy, qualifying declarations, thresholds, exclusions, and regressions so that capability updates are explainable.

### 13.7 Rebuild memory and capability views

As a cell administrator or auditor, I want derived memory and capability stores rebuilt deterministically from source records so that projections can be repaired and verified.

### 13.8 Use capability in routing and Thoth answers

As a case owner, I want plan gating and Thoth self-description to consult the same evidence-backed capability records so that routing and explanation cannot disagree silently.

---

## 14. Project specifications and mature governed artifacts

**Settlement target:** Semantic and work artifacts progress through deterministic transformation and governed maturity without teleporting across stages.

### 14.1 Run a governed projection

As a domain or case owner, I want to select a validated source, target, adapter, version, parameters, and limitations and receive validated, hash-linked output so that projections remain deterministic.

### 14.2 Navigate the spec-to-code pipeline

As a delivery lead, I want to inspect each ADR, PRD, SDS, `.sea`, semantic graph, manifest, generated-contract, last-mile, and acceptance stage with its inputs, outputs, authority, evidence, and settlement so that progress is not opaque.

### 14.3 Quarantine invalid generated output

As a delivery lead, I want malformed, semantically invalid, drifted, or unverifiable outputs blocked from downstream activation and quarantined with typed reasons so that generation failure cannot propagate.

### 14.4 Distinguish generated output from runtime readiness

As a delivery lead, I want generated contracts and handwritten bindings or runtime adapters represented as separate settled work so that file creation is not mistaken for a working system.

### 14.5 Browse artifact identity and lineage

As an artifact owner or auditor, I want to inspect content hashes, versions, type, maturity, lifecycle status, identity assurance, owner, license, review, sources, derivations, and producing evidence so that each artifact dimension remains explicit.

### 14.6 Synthesize cognitive material

As an authorized creator, I want to move a cognitive artifact to the intellectual stage through a governed derive or promote transition so that structured knowledge earns its maturity claim.

### 14.7 Productize intellectual property

As an authorized creator, I want to move an intellectual artifact to product through a governed derive or promote transition so that packaging changes remain distinguishable from maturity recognition.

### 14.8 Capitalize a product

As an authorized owner and approver, I want to recognize a product as capital only through content-preserving promotion with qualifying reuse or value evidence and separation of duty so that quality alone does not imply capital.

### 14.9 Preview and satisfy stage gates

As an artifact owner, I want to see all required semantic anchors, evidence, reviews, ownership, license, attestation, reuse, and approval conditions before requesting a transition so that missing gates, missing evidence, or missing requirements are visible.

### 14.10 Verify TransitionTokens and prevent teleportation

As an auditor, I want every transition linked through valid predecessor stages, mode, gate version, authority, evidence, and hashes so that skipped stages or rewritten lineage are rejected.

---

## 15. Share and adopt governed assets across cells

**Settlement target:** Transported records remain verifiable and isolated until a local governed adoption grants use.

### 15.1 Export a verifiable cell bundle

As a cell administrator, I want to select and preview records, runs, templates, environments, and descriptors for a hash-verifiable export so that the transport boundary is explicit.

### 15.2 Import a bundle atomically

As a cell administrator, I want the entire bundle verified before any content is admitted so that tampered or incomplete imports leave no partial state.

### 15.3 Keep imported history distinct

As an operator, I want imported records and runs visibly separated from local history so that external observations do not inflate local capability.

### 15.4 Keep imported assets inert

As a cell administrator, I want imported templates, environments, and extensions disabled or non-instantiable by default so that import does not confer authority.

### 15.5 Review provenance and compatibility

As a cell administrator, I want to inspect source cell, versions, hashes, dependencies, required authority surfaces, and local compatibility before adoption so that reuse is deliberate.

### 15.6 Adopt an imported asset through governance

As an authorized administrator, I want to adopt a verified template or environment through a governed action so that it becomes locally usable with explicit accountability.

### 15.7 Reject unsafe or incompatible imports

As a cell administrator, I want tampered, incompatible, oversized, or semantically invalid bundles rejected with the existing cell unchanged so that federation failure is bounded.

---

## 16. Maintain integrity, freshness, and operational health

**Settlement target:** The installation can evolve without silently changing the meaning, authority, or evidence of active and historical work.

### 16.1 Validate or rebuild the self-model

As a cell administrator, I want to validate source hashes independently or rebuild and reprobe the self-model after release, extension, endpoint, environment, or toolchain changes so that Thoth reflects current reality.

### 16.2 Preserve stale but verifiable snapshots

As an auditor, I want a failed rebuild to preserve the prior snapshot as explicitly stale rather than replace or corrupt it so that historical answers remain reproducible.

### 16.3 Manage extension and endpoint lifecycle

As a cell administrator, I want to register, validate, activate, disable, upgrade, deprecate, and quarantine extensions and endpoints while in-flight runs retain their launch snapshots so that change is deterministic.

### 16.4 Rebuild derived stores independently

As a cell administrator, I want memory, capability, projection, self-model, and other rebuildable views repaired without altering authoritative history so that operational recovery does not rewrite truth.

### 16.5 Inspect version skew and compatibility

As a cell administrator, I want incompatible record, model, template, environment, adapter, and extension versions tied to the operations they affect so that upgrade risk is localized.

### 16.6 Protect in-flight work during change

As an operator, I want committed plans, criteria, policy hashes, model refs, endpoint snapshots, environments, and run history preserved across reloads and upgrades so that active work cannot change meaning mid-execution.

### 16.7 Find unresolved operational debt

As a cell administrator, I want one view of stale snapshots, failed probes, degraded extensions, unsettled runs, expired approvals, quarantined records, blocked cases, and pending integrity so that hidden debt becomes actionable.

### 16.8 See honest assurance labels

As an operator or auditor, I want local digest, local tamper-evident, witnessed, degraded, pending, stale, and unsupported assurance labels shown only when their evidence requirements hold so that trust is never implied beyond proof.

---

## Cross-cutting UX invariants

These invariants apply to every story and should become the default acceptance layer when the stories are expanded.

1. **Authority precedes side effects.**
2. **Settlement determines completion.**
3. **Denial and escalation are governed outcomes, not generic errors.**
4. **Every important status resolves to committed source records.**
5. **Every derived view exposes freshness, rebuildability, and assurance.**
6. **Declared, installed, available, validated, demonstrated, degraded, unsupported, and unknown remain distinct.**
7. **Blocked and failed states expose the next lawful path.**
8. **Retries, replans, approvals, cancellations, adoptions, and transitions append history rather than edit it.**
9. **Disclosure is constrained before retrieval.**
10. **Secret values never appear in records, errors, transcripts, or views.**
11. **Agents produce untrusted work; criteria and settlement judge it.**
12. **Authors, proposers, executors, approvers, settlers, and promoters remain separated where required.**
13. **Failure retains enough trace and evidence to diagnose and recover.**
14. **A projection or index may improve usability but never become truth.**
15. **The UX prioritizes currently spendable affordances over theoretical capability inventory.**

## Explicit UX non-goals inherited from the specifications

- A raw self-model or knowledge-graph query surface for external actors.
- A conversational transcript as a source of truth.
- An unbounded autonomous Thoth loop.
- A parallel permission system inside ACP or any agent provider.
- A group-chat agent topology.
- A cron or background scheduler disguised as case management.
- A UI that edits canonical history or treats projections as authoritative.
- A completion state based solely on exit code, generated files, or agent narration.

## Specification traceability

| Specification capability | UX journey coverage |
| --- | --- |
| Minimum governed lifecycle | 1, 2, 6, 7, 9, 11, 12 |
| E1 hardened sandboxes | 2, 6, 9, 11, 12 |
| E2 case engine | 6, 7, 8, 11 |
| E3 operator and approval loop | 7, 8, 11 |
| E4 settlement integrity and capability memory | 12, 13 |
| E5 spec-to-code pipeline | 6, 14 |
| E6 SeaCell federation readiness | 15 |
| E7 governed semantic memory | 13 |
| E8 plan templates | 4, 6 |
| E9 environments and evaluators | 4, 6, 9, 12 |
| E10 artifact-to-IP | 14 |
| E11 Genesis Self-Model | 1, 3, 16 |
| E12 ADLC/ODI templates | 4, 6, 7, 11 |
| E13 Thoth governed epistemic interface | 3, 12, 16 |
| E14 agent provider and endpoint registry | 4, 9, 16 |
| E15 governed delegation | 8, 9, 11, 12 |
| E16 topology templates and Thoth manager | 4, 6, 10 |
| E17 ACP and SWE_SEED integration | 8, 9, 12 |

## Epic completion signal

A representative user can complete this full journey without reading raw implementation documentation or manually reconciling unrelated files:

```text
open or initialize a cell
→ confirm identity, policy, self-model, and integrity readiness
→ ask what the installation can actually do
→ choose a validated domain, template, environment, and executor
→ preview and commit a case with immutable criteria and provenance
→ resolve authority, approvals, and human tasks
→ execute command, projection, transition, and/or agent work
→ monitor, cancel, resume, replan, or reactivate safely
→ inspect evidence, transcript, artifacts, declarations, and settlement
→ use settled history as memory and capability
→ project, productize, capitalize, export, or adopt governed artifacts
→ verify every displayed claim back to ledgered source records
```
