# SEA Forge Workbench Mockup Brief

**Target tool:** OpenDesign  
**Design system:** `DESIGN.md`  
**Semantic mapping:** `DESIGN-spec-mapping.md`  
**Artifact goal:** A coherent, editable high-fidelity desktop mockup of SEA Forge’s primary governed-work path, with key failure states.

---

## 1. Product

SEA Forge is a governed-agent workbench.

It turns domain meaning and intended outcomes into:

```text
plan
→ authority
→ execution
→ evidence
→ settlement
→ memory
→ capability
```

The mockup must make this lifecycle visible without looking like a wizard or a linear project tracker.

The central product promise is:

```text
Know what is true.
Know what may act.
Know what actually happened.
Know what the result proves.
```

---

## 2. Deliverable

Create one cohesive desktop application mockup with:

- a persistent application shell;
- a left navigation rail;
- a top context bar;
- a main content region;
- a contextual right-side Evidence/Provenance drawer;
- route-level screens for the primary path;
- reusable components shared across screens;
- clickable or clearly linked transitions;
- realistic mock data;
- no backend implementation.

Preferred artifact:

- multi-screen HTML/JSX prototype or equivalent OpenDesign artifact;
- 1440×900 primary desktop frame;
- include a compact 1180px adaptation for at least Case Horizon and Settlement;
- include one phone operational frame for approval or run monitoring.

---

## 3. Shared shell

### Top context bar

Show:

- Cell: `Foundry-01`;
- Actor: `Sam`;
- Role: `Founder / Operator`;
- Policy: `PB-14`;
- Integrity: `Externally witnessed`;
- Global search;
- Inbox count;
- Active work count;
- Density selector: Guided / Operational / Audit.

### Left navigation

```text
Readiness
Thoth
Assets
Domain Models
Cases
Inbox
Operations
Evidence
Memory
Capabilities
Artifacts
Federation
Administration
```

Current route uses the blue focus token.

### Right drawer

Tabs:

```text
Why
Evidence
Provenance
Record
```

The drawer should open from any major state or record.

---

## 4. Primary-path screens

Create all screens below using one visual system and shared components.

### Screen A — Readiness Overview

**Purpose:** Determine whether the cell can perform intended work.

Header:

```text
Foundry-01
READY WITH 2 LIMITATIONS
Last verified 4 min ago
Snapshot SM-019
```

Include:

- Intended work selector set to `Create governed case`;
- Critical Foundations list:
  - Cell identity — ready;
  - Actor resolved — ready;
  - Policy loaded — ready;
  - Ledger integrity — externally witnessed;
  - Genesis model — ready;
  - Self-model snapshot — current;
- Operational Capabilities:
  - Case engine — ready;
  - Jail sandbox — ready;
  - Agent delegation — ready;
  - TLA+ projection — unavailable;
  - Anthropic endpoint — validation stale;
- Current lawful actions:
  - `Create case` dominant;
  - `Ask Thoth`;
  - `Browse templates`;
  - `Repair 2 limitations`;
- Recent invalidation row.

Do not make the whole page green. The two limitations remain visible.

### Screen B — New Case

**Purpose:** Choose the starting representation.

Purpose text:

```text
Build and validate a repair agent for recurring service incidents.
```

Entry cards:

- ADLC Case — recommended;
- From Template;
- External Plan Proposal;
- Sequential Agents;
- Concurrent Agents;
- Specification Pipeline;
- Known Intent;
- ODI-grounded ADLC.

Each card shows:

- what it creates;
- relative complexity;
- current availability;
- missing requirement when unavailable.

Label the page:

```text
DRAFT ONLY
No authority is conferred at this step.
```

### Screen C — Case Configuration

Three-column layout:

1. Configuration sections;
2. Active editor;
3. Draft health.

Sections:

```text
Purpose & Owner
Domain Model
Stages & Work
Settlement Criteria
Environment & Sandbox
Delegates
Limits
Review
```

Active section: `Settlement Criteria`.

Show one criterion editor:

```text
Criterion: Repair recovery test passes
Expected: all pass
Evaluator: repair.tests
Required evidence: JUnit report
Origin: Desired Outcome — Reduce Repair Time
Reliability: qualifying declaration required
```

Draft health:

```text
12 valid
3 incomplete
1 unavailable
```

Unavailable item:

```text
Anthropic endpoint configuration is stale.
[Open endpoint]
```

Footer:

```text
Saved locally 8 sec ago
Not committed to SEA Forge
```

### Screen D — Case Preflight

Header:

```text
READY TO COMMIT
Nothing has executed.
```

Contract summary:

- Desired outcome;
- Domain model and hash;
- Template and version;
- Owner/sponsor;
- Completion semantics.

Three burden panels:

- Work shape:
  - 7 stages;
  - 18 plan items;
  - 4 agent tasks;
  - 2 human tasks.
- Authority and coordination:
  - 4 protected surfaces;
  - 2 approvals possible;
  - Jail sandbox;
  - allowlisted network.
- Settlement:
  - 12 immutable criteria;
  - 3 independent declarations;
  - origin coverage 100%.

Preflight checks:

- model references resolve;
- sentries satisfiable;
- evaluators available;
- endpoint validated but not demonstrated;
- creation authority allowed.

Dominant action:

```text
Commit governed case
```

Include a commit confirmation overlay that lists what will and will not happen.

### Screen E — Case Overview

Case:

```text
Build Repair Agent
ACTIVE
3 of 7 milestones settled
```

Include:

- Desired outcome;
- owner;
- plan version;
- model version;
- current horizon counts:
  - 2 spendable now;
  - 1 active;
  - 1 approval;
  - 2 blocked;
  - 8 future;
- Needs Attention:
  - approval expires in 2h;
  - endpoint probe stale;
- Latest Settlement:
  - Simulation rejected;
  - recovery test failed;
- Capability Effect:
  - Repair validation +1 qualifying observation;
  - agent delegation attempted, not demonstrated;
- Milestone trajectory with one active stage and future stages;
- one dominant `Open case horizon` action.

### Screen F — Case Horizon

Support Board and List switches. Show Board by default.

Columns:

```text
SPENDABLE NOW
ACTIVE
AWAITING / BLOCKED
FUTURE / SETTLED
```

Cards:

1. Spendable:
   - `Run repair simulation`;
   - command task;
   - 3 criteria;
   - Jail sandbox;
   - `Start`.

2. Active:
   - `Agent writes adapter`;
   - Dialogue streaming;
   - Turn 4/12;
   - Tokens 31%;
   - `Open`.

3. Awaiting:
   - `Deploy test environment`;
   - approval pending;
   - TTL 1h 58m;
   - `Resolve`.

4. Future:
   - `Activate staging`;
   - Sentry: Build settlement accepted.

5. Settled:
   - Domain Model validated.

Selected-item drawer explains:

- why spendable;
- authority boundaries;
- environment;
- criteria;
- evidence needed;
- next action.

No drag-and-drop state mutation.

### Screen G1 — Command Run Detail

Run:

```text
RUN-07
Execute Repair Simulation
Execution: SUCCEEDED
Settlement: EVALUATING
```

Use a sticky dual-state header.

Show:

- lifecycle tracker;
- governed boundary:
  - Jail sandbox;
  - workspace filesystem;
  - network denied;
  - 10m timeout;
- output tabs:
  - stdout;
  - stderr;
  - events;
- evidence:
  - stdout.txt verified;
  - junit.xml verified;
  - repair-model.json artifact;
- settlement preview:
  - 2 criteria pass;
  - 1 criterion fails;
  - independent declaration pending.

Copy:

```text
Output is execution evidence, not settlement.
```

### Screen G2 — Agent Task Detail

Agent task:

```text
AGENT-RUN-12
Implement Adapter
```

Show separately:

```text
Dialogue: STREAMING
Termination: —
Settlement: UNSETTLED
```

Metadata:

- Role: Implementation;
- Protocol: ACP;
- Endpoint: prod-anthropic;
- Provider/model: Anthropic / Claude Code;
- SWE_SEED active;
- Sponsor: Sam;
- Session: ACP-443.

Budget:

- Turns 4/12;
- Tokens 31%;
- Time 6m/20m;
- Binding limit: turns.

Structured dialogue region, not chat bubbles.

Pending permission:

```text
Agent requests:
write crates/agent/src/acp.rs

Boundary:
workspace path only
no network
one operation

[Open approval]
```

Transcript evidence:

- summary available;
- transcript sealed;
- digest;
- access request.

Bottom warning:

```text
Agent claims “implementation complete.”
This claim has no settlement standing.
```

### Screen H — Settlement Detail

Settlement:

```text
SET-22
REJECTED
```

Plain explanation:

```text
The command succeeded, but one committed criterion failed.
```

Expected vs observed outcome.

Criterion matrix:

1. Functional tests — pass;
2. Recovery after restart — fail;
3. Artifact identity — pass.

Show:

- evidence;
- reliability;
- declaration standing;
- consequence:
  - plan item rejected;
  - remediation activated;
  - no capability promotion;
  - recovery regression recorded.

Dominant recovery action:

```text
Open activated remediation
```

Secondary:

- retry as new episode;
- inspect evidence.

No generic Accept Settlement button.

### Screen I — Capability Detail

Capability:

```text
Repair Adapter Implementation
DEMONSTRATED
```

Summary:

```text
Demonstrated in 3 cases.
Not yet proven under endpoint-disconnect variation.
```

Show:

- ladder:
  - Attempted complete;
  - Demonstrated current;
  - Proven not reached;
- evidence strength:
  - 5 accepted settlements;
  - reliability 0.88;
  - 1 regression;
- variation coverage;
- recovery coverage;
- orchestration burden trend;
- promotion conditions:
  - three varied cases complete;
  - disconnect recovery missing;
  - reliability ≥0.90 missing;
- source settlements;
- next possible test;
- next currently spendable proof path.

Dominant action:

```text
Start governed proof case
```

This returns to New Case rather than changing capability directly.

---

## 5. Required failure and recovery variants

Create separate frames or clear state variants for these.

### Variant 1 — Integrity Halt

On Readiness:

```text
INTEGRITY HALTED
Ledger stream verification failed.
Mutating actions are unavailable.
```

Actions:

- `Open Integrity`;
- `View last verified checkpoint`;
- `Open maintenance`.

Do not show Create Case.

### Variant 2 — Approval Escalation

On Preflight:

```text
APPROVAL REQUIRED
External API use requires an eligible approver.
```

Show exact resource, boundary, request digest, expiry, and `Open approval`.

### Variant 3 — Agent Permission Denied

On Agent Task:

- permission state rejected;
- agent remains active within remaining grant;
- no automatic cancellation;
- decision visible in structured turn stream.

### Variant 4 — Command Succeeds, Settlement Fails

Use the Run → Settlement transition to emphasize:

```text
Execution: SUCCEEDED
Settlement: REJECTED
```

Then show remediation activation.

### Variant 5 — Stale Preflight

```text
PREFLIGHT INVALIDATED
Policy bundle changed from PB-14 to PB-15.
Review the diff and rerun preflight.
```

Commit action disabled with reason.

### Variant 6 — Event Stream Stale

On Operations or Run:

```text
Last confirmed event: EVT-1182
Live state may be stale.
Reconnecting…
```

Keep last verified state visible.

---

## 6. Interaction rules

- One dominant action per screen.
- Clicking a status opens Why or Evidence.
- Clicking a source/freshness badge opens Record metadata.
- Disabled actions always show why.
- Back navigation never implies undo.
- Commit, approve, cancel, retry, and declaration use explicit decision dialogs.
- Cancel enters `cancelling`; it does not jump to `cancelled`.
- Retry creates a new episode.
- Board/List/Graph change only projection.
- Evidence and machine records remain secondary but reachable in one action.
- Do not fabricate backend operations for visual completeness.

---

## 7. Copy style

Use:

- “Execution completed. Settlement evaluation continues.”
- “No path is currently spendable.”
- “Approval APR-07 expires in 1h 58m.”
- “Commit blocked: the policy bundle changed.”
- “This agent claim has no settlement standing.”
- “One qualifying observation was added.”
- “Capability remains demonstrated; disconnect recovery is unproven.”

Avoid:

- “Congratulations!”
- “Mission accomplished!”
- “The AI finished.”
- “Everything looks good.”
- “Try again.”
- generic “Error” without cause and path.

---

## 8. Visual constraints

- Dark-only.
- No gradients.
- No glassmorphism.
- No oversized rounded cards.
- Panel radius 4px; dialogs up to 6px.
- Compact 32–48px rows.
- Inter and JetBrains Mono.
- Lucide-style icons.
- Blue for current focus and active evaluation.
- Green only for exact accepted/allowed/verified states.
- Amber for attention, escalation, and stale state.
- Red for denial, rejection, failure, and integrity halt.
- Purple for quarantine.
- Color never acts alone.

---

## 9. Mockup settlement criteria

The mockup is successful when:

1. all primary screens visibly belong to one product;
2. Readiness → Case → Execution → Settlement → Capability can be followed without explanation outside the UI;
3. authority, evidence, and source freshness are visible throughout;
4. execution success cannot be confused with settlement acceptance;
5. agent termination cannot be confused with settlement;
6. capability is conservative and evidence-backed;
7. failure states produce clear recovery paths;
8. the product looks like governed mission control, not project management or chat;
9. dense operational information remains readable at 1440×900;
10. no unsupported score, command, choice, or backend object is invented.
