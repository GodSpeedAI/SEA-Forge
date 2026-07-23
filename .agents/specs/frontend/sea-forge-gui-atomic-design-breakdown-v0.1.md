# SEA Forge GUI Atomic Design Breakdown

**Status:** Draft v0.1  
**Source:** SEA Forge Governed Workbench UX Epic v0.1  
**Purpose:** Define the reusable GUI system before sequencing views, inputs, outputs, and transitions.  
**Boundary:** This artifact defines the interface inventory and composition hierarchy. It does **not** yet define navigation order, route transitions, or complete task flows.

---

## 0. Design rule

SEA Forge's GUI must not become a decorative projection of CLI commands. It must make the governed lifecycle understandable:

```text
meaning
→ plan
→ authority
→ execution
→ evidence
→ settlement
→ memory
→ capability
```

The atomic system must preserve these distinctions everywhere:

- source truth versus rebuildable projection;
- declared versus installed versus available versus validated versus demonstrated;
- execution state versus settlement state;
- denial versus escalation versus error;
- agent output versus accepted evidence;
- visible possibility versus currently spendable affordance;
- actor, sponsor, approver, executor, settler, and promoter;
- immutable historical record versus proposed next action.

Every visible state should answer:

1. What is this?
2. What state is it in?
3. What evidence supports that state?
4. What can the user do next?
5. What authority or payment is required?
6. What would count as settlement?

---

# 1. Foundations

Foundations are not visible components by themselves. They establish the rules from which atoms and larger structures are built.

## 1.1 Semantic status vocabulary

The GUI must use a single canonical status vocabulary.

### Capability and availability states

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

### Governance dispositions

```text
allow
deny
escalate
degraded
pending
expired
```

### Execution states

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

### Settlement states

```text
unsettled
evaluating
accepted
rejected
escalated
quarantined
```

### Case and plan-item states

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

### Integrity states

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

## 1.2 Status presentation contract

Status must never rely on color alone. Each state representation includes:

- icon;
- text label;
- optional severity or assurance qualifier;
- accessible description;
- source or freshness indicator when derived;
- tooltip or detail affordance for non-obvious states.

## 1.3 Visual hierarchy

The primary hierarchy is:

```text
Current state
→ Evidence for state
→ Next lawful action
→ Historical context
→ Raw machine record
```

The interface should not lead with raw logs, hashes, or implementation vocabulary unless the user expands the evidence layer.

## 1.4 Density modes

SEA Forge needs three density modes:

- **Guided:** simplified labels, explanations, and recommended next actions.
- **Operational:** dense case, run, approval, and monitoring views.
- **Audit:** evidence-first records, hashes, provenance, and machine-readable details.

The same underlying components should adapt rather than creating three unrelated products.

## 1.5 Interaction principles

- No destructive action without impact preview.
- No approval without exact decision context.
- No retry that overwrites the prior attempt.
- No derived status without source and freshness.
- No hidden side effect from selecting an item.
- No "success" label before settlement acceptance; the only positive-run message atom is `ExecutionSucceededMessage`, and it MUST reflect an accepted/verified settlement outcome, never a generic "success" state.
- No disabled control without a reason.
- No generic error when a typed governed outcome exists.
- No activity spinner without an observable state and cancellation rule.
- No agent conversation surface that implies the transcript is the source of truth.

## 1.6 Global layout foundations

Primary layout regions:

```text
Application shell
├── Global navigation
├── Workspace / cell context
├── Identity and policy context
├── Primary content
├── Contextual evidence drawer
├── Notifications / approval tray
└── Command palette
```

---

# 2. Atoms

Atoms are the smallest reusable interface elements. They should not carry complex business logic by themselves.

## 2.1 Typography atoms

- Product title
- View title
- Section heading
- Subsection heading
- Body text
- Supporting text
- Monospace value
- Record identifier
- Timestamp
- Annotation text
- Warning text
- Empty-state text

## 2.2 Icon atoms

Canonical icons for:

- cell / workspace;
- actor / identity;
- sponsor;
- authority;
- policy;
- model;
- template;
- environment;
- extension;
- endpoint;
- case;
- stage;
- plan item;
- sentry;
- milestone;
- run;
- sandbox;
- agent;
- Thoth;
- evidence;
- settlement;
- memory;
- capability;
- artifact;
- projection;
- ledger;
- integrity;
- import/export;
- approval;
- warning;
- denial;
- escalation;
- retry;
- reopen;
- cancel;
- inspect;
- compare;
- download/export;
- machine-readable source.

## 2.3 Status atoms

- Status dot
- Status icon
- Status label
- Status badge
- Assurance badge
- Freshness badge
- Source-truth badge
- Derived-view badge
- Stale indicator
- Restricted indicator
- Read-only indicator
- Immutable indicator
- Required indicator
- Optional indicator

## 2.4 Identity atoms

- Actor avatar
- Actor type badge
- Role badge
- Sponsor badge
- Cell ID label
- Organization label
- Identity source label

## 2.5 Input atoms

- Text input
- Search input
- Multiline input
- Code input
- Number input
- Duration input
- Token-budget input
- File picker
- Directory picker
- Date/time input
- Boolean switch
- Radio button
- Checkbox
- Select
- Multi-select
- Combobox
- Tag input
- Key/value input
- Secret-reference input
- Semantic-reference input
- Record-reference input
- Hash input/display
- Version input
- Namespace input

## 2.6 Action atoms

- Primary action button
- Secondary action button
- Tertiary text action
- Destructive action button
- Approval button
- Reject button
- Escalate button
- Cancel-run button
- Retry-as-new-episode button
- Reopen-case button
- Verify button
- Rebuild button
- Probe button
- Inspect-evidence button
- Copy-reference button
- Open-raw-record button
- Download/export button

## 2.7 Navigation atoms

- Navigation item
- Breadcrumb item
- Back control
- Tab
- Step indicator
- Pagination control
- Expand/collapse control
- Anchor link
- External-tool link
- Command-palette trigger

## 2.8 Data atoms

- Key label
- Value label
- Hash value
- Version value
- Duration value
- Cost/budget value
- Count value
- Progress value
- Reliability value
- Confidence value
- Criterion result
- Evidence count
- Pending count
- Variation count
- Recovery count

## 2.9 Feedback atoms

- Inline validation message
- Error message
- Warning message
- Information message
- Success message
- Denial message
- Escalation message
- Loading indicator
- Progress indicator
- Skeleton state
- Empty-state icon
- Toast
- Notification dot

---

# 3. Molecules

Molecules combine atoms into small, reusable units with one clear purpose.

## 3.1 Context molecules

### Cell Context Selector
Displays current cell, environment, assurance, and workspace status.

**Stories:** 1.1–1.7, 15.1–15.5

### Actor Context Selector
Displays acting identity, role, actor type, and sponsor.

**Stories:** 2.1–2.3

### Policy Context Chip
Shows active policy bundle, version, digest, and freshness.

**Stories:** 2.4–2.7

### Source Context Chip
Shows whether a value is authoritative, derived, imported, stale, or rebuilt.

**Stories:** cross-cutting, especially 3, 12, 13, 15

## 3.2 Status molecules

### Governed Status Pill
Combines state, icon, assurance, and explanatory tooltip.

### Dual-State Indicator
Shows execution state and settlement state separately.

**Example:**

```text
Execution: succeeded
Settlement: evaluating
```

**Stories:** 9.8–9.9, 12.3–12.4

### Availability Ladder
Displays:

```text
declared → installed → available → validated → demonstrated → proven
```

with the current achieved level and blockers.

**Stories:** 3.2, 4.7–4.8, 13.4–13.6

### Integrity Indicator
Displays ledger or record assurance with verification action.

**Stories:** 12.7–12.10, 15.1–15.3

## 3.3 Search and filter molecules

- Global Search Bar
- Asset Filter Bar
- Case Filter Bar
- Run Filter Bar
- Approval Filter Bar
- Evidence Filter Bar
- Audit Filter Bar
- Memory Recall Filter
- Capability Filter
- Event Stream Filter

## 3.4 Form molecules

### Semantic Reference Picker
Searches validated concepts and returns a pinned concept reference.

**Stories:** 5.3–5.5

### Record Reference Picker
Searches cases, runs, settlements, artifacts, capabilities, or evidence records.

### Versioned Asset Picker
Selects a template, environment, extension, model, or endpoint by exact version.

**Stories:** 4.1–4.5, 6.2, 6.7

### Authority Boundary Editor
Edits or displays exact action boundaries such as:

- resource;
- endpoint;
- host;
- path;
- sandbox;
- timeout;
- turn cap;
- token budget;
- allowed tools;
- secret reference.

**Stories:** 2.5–2.7, 7.7–7.8, 9.1–9.7

### Settlement Criterion Editor
Defines an immutable criterion with:

- criterion type;
- evaluator;
- expected result;
- origin;
- evidence requirement;
- optional reliability requirement.

**Stories:** 6.8–6.9, 7.5–7.6

### Origin Reference Editor
Links a criterion or work item to:

- intent;
- requirement;
- desired outcome;
- policy;
- template;
- specification stage;
- issue;
- external source.

### Agent Limit Editor
Controls model, turns, tokens, timeout, retention, continuation, and tools.

**Stories:** 9.4–9.7

## 3.5 Card molecules

- Cell Card
- Model Card
- Template Card
- Environment Card
- Extension Card
- Endpoint Card
- Capability Card
- Case Card
- Stage Card
- Plan Item Card
- Run Card
- Approval Card
- Evidence Card
- Settlement Card
- Memory Card
- Artifact Card
- Projection Card
- Import Bundle Card
- Integrity Check Card

Each card must expose:

- identity;
- current state;
- source or version;
- primary evidence indicator;
- next lawful action;
- detail affordance.

## 3.6 Explanation molecules

### Why This State
Explains the records and rules that produced a state.

### Why Blocked
Shows unmet conditions and the next possible corrective action.

### Why Denied
Shows the denied surface, reason, policy references, and whether an appeal or new request is possible.

### Why Escalated
Shows the missing judgment, evidence, or authority.

### Why Promoted / Not Promoted
Shows thresholds, qualifying evidence, and missing conditions.

### What Happens Next
Explains the next state transition and required payment.

## 3.7 Evidence molecules

### Evidence Reference
A compact evidence link with type, digest, source, and verification state.

### Criterion-Evidence Pair
Displays one criterion against supporting or failing evidence.

### Provenance Chain Segment
Displays:

```text
origin → plan item → run → evidence → settlement
```

### Artifact Identity Block
Displays content hash, artifact descriptor, producer, owner, license, and maturity.

### Transcript Evidence Summary
Displays transcript digest, retention mode, turn count, redaction state, and access action.

### Machine Record Link
Opens or downloads the authoritative typed record.

## 3.8 Timeline molecules

- Event Row
- Timeline Marker
- Parallel Episode Group
- Approval Event
- State Transition Event
- Settlement Event
- Reactivation Event
- Mutation Event
- Configuration Snapshot Event

---

# 4. Organisms

Organisms are substantial interface sections composed from molecules.

## 4.1 Application Shell

Contains:

- global navigation;
- current cell;
- actor and policy context;
- notification and approval indicators;
- global search;
- command palette;
- evidence drawer trigger.

Supports all journeys.

## 4.2 Readiness Console

Contains:

- cell readiness summary;
- startup requirements;
- self-model status;
- model realization;
- active policy;
- ledger integrity;
- extension status;
- environment/toolchain status;
- corrective actions.

**Stories:** 1.1–1.7, 15.1–15.5

## 4.3 Identity and Authority Header

Contains:

- acting identity;
- role;
- sponsor;
- policy bundle;
- authority preflight;
- exact configuration snapshot.

**Stories:** 2.1–2.7

## 4.4 Thoth Question Composer

Contains:

- typed question selector;
- target capability, operation, projection, environment, or failure;
- current actor/disclosure context;
- snapshot selector where permitted;
- submit action;
- disclosure preview.

**Stories:** 3.1–3.9

## 4.5 Thoth Answer Panel

Contains:

- answer summary;
- claim status;
- evidence links;
- snapshot;
- limitations;
- omitted claim classes;
- authority disclaimer;
- reproducibility details;
- governed denial presentation.

**Stories:** 3.1–3.9

## 4.6 Asset Catalog

One organism with configurable asset type:

- templates;
- environments;
- extensions;
- endpoints;
- capabilities;
- projections.

Contains:

- search and filters;
- status facets;
- version and compatibility;
- asset cards;
- detail drawer;
- readiness or unusability explanation.

**Stories:** 4.1–4.8

## 4.7 Domain Model Workbench

Contains:

- `.sea` source editor/file source;
- import tree;
- validation results;
- semantic concept explorer;
- model metadata;
- model pinning;
- projection actions;
- drift comparison.

**Stories:** 5.1–5.7

## 4.8 Case Creation Workbench

Contains:

- entry-source selector;
- intent input;
- template selector;
- external-plan upload;
- ADLC/ODI selection;
- multi-agent topology selection;
- spec-to-code pipeline selection;
- parameter form;
- model/environment binding;
- preflight results;
- immutable commit action.

**Stories:** 6.1–6.9

## 4.9 Plan Inspector

Contains:

- case purpose;
- desired outcome;
- stages;
- plan items;
- sentries;
- milestones;
- criteria;
- origins;
- authority boundaries;
- execution and agent limits;
- unresolved blockers.

**Stories:** 6.8–6.9, 7.1–7.4

## 4.10 Case Horizon Board

This is the primary live-case organism.

Columns or regions:

```text
Spendable now
Active
Awaiting authority / approval
Blocked
Future / sentry-gated
Settled
```

Contains:

- stage grouping;
- plan-item cards;
- milestone markers;
- sentry explanations;
- discretionary-item action;
- current spendable next moves.

**Stories:** 7.1–7.8

## 4.11 Case Timeline

Contains:

- committed plan;
- activations;
- approvals;
- runs;
- cancellations;
- settlements;
- milestones;
- replans;
- reactivations;
- termination.

**Stories:** 7.4–7.8, 11.1–11.8, 12.8

## 4.12 Approval Inbox

Contains:

- approval and human-task queues;
- filters;
- age and expiry;
- risk/impact indicator;
- assignment;
- decision context preview.

**Stories:** 8.1–8.7

## 4.13 Approval Decision Panel

Contains:

- actor;
- requested action;
- exact resource and boundaries;
- policy reason;
- evidence;
- downstream consequence;
- separation-of-duty warning;
- approve/reject controls;
- decision note.

**Stories:** 8.2–8.7

## 4.14 Execution Console

Contains:

- run identity;
- authority state;
- sandbox and environment;
- execution progress;
- bounded stdout/stderr;
- resource usage;
- cancellation;
- generated artifacts;
- evaluator status;
- settlement handoff.

**Stories:** 9.1–9.3, 9.8–9.9

## 4.15 Agent Task Console

Contains:

- endpoint and model;
- instruction packet;
- limits and consumption;
- continuation identity;
- streaming dialogue status;
- permission requests;
- transcript summary;
- SWE_SEED evidence;
- cancellation;
- termination versus settlement.

**Stories:** 9.4–9.9

## 4.16 Thoth Manager Panel

Contains:

- iteration grant;
- case classification;
- supporting records;
- selected proposal source;
- proposed task;
- proposal authority status;
- remaining iterations;
- stop/park/escalate state.

**Stories:** 10.1–10.6

## 4.17 Operations Monitor

Contains:

- event stream;
- active runs;
- parallel groups;
- concurrency saturation;
- parked cases;
- interruption recovery;
- notifications;
- scoped control actions.

**Stories:** 11.1–11.8

## 4.18 Run Evidence Inspector

Contains:

- plan item;
- authority decisions;
- trace;
- evidence;
- artifacts;
- transcript evidence;
- evaluator results;
- settlement;
- declarations;
- semantic envelope;
- machine records.

**Stories:** 12.1–12.10

## 4.19 Criterion Settlement Matrix

Rows are immutable criteria. Columns include:

- origin;
- expected result;
- evaluator;
- evidence;
- result;
- reliability;
- declaration standing;
- settlement contribution.

**Stories:** 12.2–12.6

## 4.20 Ledger and Integrity Inspector

Contains:

- ledger health;
- stream continuity;
- checkpoint list;
- witnesses;
- inclusion proof;
- consistency proof;
- verification action;
- quarantined or forked records.

**Stories:** 12.7–12.10, 15.1–15.3

## 4.21 Memory Recall Workspace

Contains:

- governed query;
- scope filters;
- source records;
- relevance;
- authority result;
- stale-index state;
- influence links to plans and runs.

**Stories:** 13.1–13.3

## 4.22 Capability Inspector

Contains:

- capability state ladder;
- source settlements;
- variation map;
- recovery evidence;
- reliability;
- regressions;
- orchestration burden;
- promotion policy;
- routing consequences.

**Stories:** 13.4–13.8

## 4.23 Projection Pipeline Inspector

Contains:

- source model/spec;
- pipeline stages;
- adapter/version;
- parameters;
- stage status;
- quarantine;
- outputs;
- hashes;
- last-mile work;
- acceptance settlement.

**Stories:** 5.7, 6.6, 14.1–14.4

## 4.24 Artifact Maturity Inspector

Contains:

- artifact descriptor;
- content identity;
- lineage;
- current maturity;
- lifecycle state;
- attestation;
- stage-gate requirements;
- transition history;
- derive/promote distinction;
- transition action.

**Stories:** 14.5–14.9

## 4.25 Federation Transfer Workbench

Contains:

- bundle contents;
- source cell;
- hashes and dependencies;
- secret exclusion;
- import verification;
- isolation state;
- adoption diff;
- adoption authority.

**Stories:** 14.10–14.14

## 4.26 Administration and Maintenance Console

Contains:

- self-model validate/rebuild;
- projection rebuilds;
- extension lifecycle;
- endpoint lifecycle;
- version skew;
- stale snapshots;
- unsettled runs;
- expired approvals;
- quarantined records;
- operational debt.

**Stories:** 15.1–15.5

---

# 5. Templates

Templates define page-level composition without binding to a particular record.

## 5.1 Catalog Template

```text
Page header
├── Context and readiness
├── Search and filter bar
├── Status summary
├── Asset grid/list
└── Detail drawer
```

Used by:

- templates;
- environments;
- extensions;
- endpoints;
- capabilities;
- artifacts;
- projections.

## 5.2 Record Detail Template

```text
Record header
├── Identity and state
├── Primary actions
├── Summary
├── Evidence / provenance tabs
├── History / timeline
└── Raw machine record
```

Used by:

- cases;
- runs;
- settlements;
- capabilities;
- artifacts;
- approvals;
- imports.

## 5.3 Workbench Template

```text
Page header
├── Source/input region
├── Validation/preflight region
├── Main authoring or configuration region
├── Context/evidence drawer
└── Review and commit footer
```

Used by:

- domain model authoring;
- case creation;
- projection configuration;
- artifact transition;
- federation transfer.

## 5.4 Live Operations Template

```text
Operational header
├── Current state and controls
├── Live stream / timeline
├── Active work region
├── Capacity / approval / blocker side rail
└── Evidence and settlement drawer
```

Used by:

- case operations;
- execution console;
- agent console;
- Thoth manager;
- server monitor.

## 5.5 Decision Template

```text
Decision header
├── Requested action
├── Actor and authority context
├── Exact boundaries
├── Evidence and impact
├── SoD / risk warnings
└── Approve / reject / escalate controls
```

Used by:

- operator approvals;
- ACP permission requests;
- artifact transitions;
- imported asset adoption;
- policy-sensitive plan mutation.

## 5.6 Audit Template

```text
Audit header
├── Verification status
├── Source records
├── Provenance chain
├── Integrity proofs
├── Filters and comparison
└── Machine-readable export
```

Used by:

- run evidence;
- ledger verification;
- authority history;
- disclosure history;
- capability promotion;
- projection verification.

## 5.7 Guided Explanation Template

```text
Question / problem
├── Direct answer
├── Current state
├── Why this state
├── Evidence
├── Limitations
└── Next lawful action
```

Used by:

- Thoth;
- readiness failures;
- denied operations;
- blocked plan items;
- failed runs;
- promotion explanations.

---

# 6. Page and view families

Pages are routable or independently addressable views. Modal dialogs and drawers are listed separately afterward.

## 6.1 Global and startup pages

### P1 — Cell Gateway
Open, create, migrate, or reconnect to a cell.

**Stories:** 1.1–1.3

### P2 — Readiness Overview
System readiness, self-model, identity, policy, ledger, extensions, environments, and corrective actions.

**Stories:** 1.4–1.7

### P3 — Identity and Authority Context
Identity resolution, sponsorship, role, active policy, and authority preflight.

**Stories:** 2.1–2.7

## 6.2 Thoth pages

### P4 — Ask Thoth
Typed question composition and answer history.

**Stories:** 3.1–3.9

### P5 — Thoth Answer Detail
Claim-by-claim evidence, disclosure decision, snapshot, limitations, and reproducibility.

**Stories:** 3.7–3.9

## 6.3 Asset pages

### P6 — Asset Catalog
Unified entry to templates, environments, extensions, endpoints, capabilities, and projections.

**Stories:** 4.1–4.8

### P7 — Asset Detail
Versioned asset status, evidence, compatibility, dependencies, and actions.

**Stories:** 4.1–4.8

## 6.4 Domain pages

### P8 — Domain Models
List validated, invalid, stale, and imported models.

**Stories:** 5.1–5.7

### P9 — Domain Model Workbench
Author/select, validate, inspect, pin, compare drift, and project `.sea`.

**Stories:** 5.1–5.7

### P10 — Projection Detail
Projection request, source, adapter, parameters, validation, hashes, and outputs.

**Stories:** 5.7, 14.1–14.4

## 6.5 Case creation and planning pages

### P11 — New Case
Choose intent, template, external plan, ADLC/ODI, topology, or pipeline.

**Stories:** 6.1–6.7

### P12 — Case Configuration
Parameters, domain model, environments, evaluators, agents, criteria, and limits.

**Stories:** 6.4–6.8

### P13 — Case Preflight
Complete plan preview, blockers, authority requirements, and immutable commit.

**Stories:** 6.8–6.9

## 6.6 Case operation pages

### P14 — Cases
Case catalog with purpose, state, current horizon, owner, and settlement summary.

**Stories:** 7.1–7.8

### P15 — Case Overview
Purpose, desired outcome, progress, current affordances, and primary actions.

**Stories:** 7.1–7.8

### P16 — Case Horizon
Stage and plan-item board showing spendable, active, blocked, future, and settled work.

**Stories:** 7.2–7.8

### P17 — Case Timeline
All activations, mutations, approvals, runs, settlements, milestones, and reactivations.

**Stories:** 7.4–7.8, 11.1–11.8

### P18 — Plan Detail
Immutable plan, stages, sentries, milestones, criteria, origins, boundaries, and versions.

**Stories:** 7.1–7.4

## 6.7 Approval pages

### P19 — Approval and Human Task Inbox
Cross-case queue.

**Stories:** 8.1–8.7

### P20 — Approval Detail
Exact decision context and controls.

**Stories:** 8.2–8.7

### P21 — Human Task Detail
Task instructions, evidence submission, and completion.

**Stories:** 8.5

## 6.8 Execution pages

### P22 — Run Detail
Execution status, authority, environment, logs, artifacts, evaluators, and settlement.

**Stories:** 9.1–9.3, 9.8–9.9

### P23 — Agent Task Detail
Endpoint, model, packet, dialogue, permissions, transcript, proofs, and settlement.

**Stories:** 9.4–9.9

### P24 — Thoth Manager
Case judgment, bounded proposals, iteration budget, and escalation.

**Stories:** 10.1–10.6

### P25 — Operations Monitor
Live events, concurrency, active runs, interruption recovery, and controls.

**Stories:** 11.1–11.8

## 6.9 Evidence and audit pages

### P26 — Evidence Explorer
Searchable evidence across cases and runs.

**Stories:** 12.1–12.6

### P27 — Settlement Detail
Criteria matrix, evidence, declarations, reliability, and status.

**Stories:** 12.3–12.6

### P28 — Audit History
Authority, approvals, disclosure, agent permissions, and mutations.

**Stories:** 12.7–12.10

### P29 — Integrity and Ledger
Ledger health, checkpoints, witnesses, proofs, and verification.

**Stories:** 12.7–12.10

## 6.10 Memory and capability pages

### P30 — Memory Recall
Governed search and influence trace.

**Stories:** 13.1–13.3

### P31 — Capability Catalog
Capability states, evidence, variation, recovery, and routing readiness.

**Stories:** 13.4–13.8

### P32 — Capability Detail
Promotion logic, source settlements, reliability, burden, regression, and Thoth use.

**Stories:** 13.4–13.8

## 6.11 Projection, artifact, and federation pages

### P33 — Specification Pipeline
Stage-by-stage source-to-runtime work.

**Stories:** 14.1–14.4

### P34 — Artifact Catalog
Artifact identity, maturity, lifecycle, ownership, and status.

**Stories:** 14.5–14.9

### P35 — Artifact Detail
Lineage, evidence, stage gates, transition history, and attestation.

**Stories:** 14.5–14.9

### P36 — Artifact Transition
Synthesize, productize, or capitalize with gate preview and approval.

**Stories:** 14.5–14.9

### P37 — Federation Bundles
Export, import, verify, isolate, review, and adopt.

**Stories:** 14.10–14.14

## 6.12 Administration pages

### P38 — Administration
Extensions, endpoints, models, environments, policy state, and versions.

**Stories:** 15.1–15.5

### P39 — Maintenance and Debt
Stale snapshots, rebuilds, unresolved runs, expired approvals, quarantines, and skew.

**Stories:** 15.1–15.5

---

# 7. Dialogs, drawers, and overlays

These are reusable transient views, not standalone pages.

## 7.1 Dialogs

- Create/Open Cell
- Migration Preview
- Sponsor Actor
- Authority Preflight
- Endpoint Probe
- Commit Case
- Add Discretionary Item
- Replan Case
- Reopen Case
- Terminate Case
- Approval Decision
- ACP Permission Decision
- Cancel Run
- Retry as New Episode
- Resume Interrupted Work
- Rebuild Projection
- Rebuild Self-Model
- Verify Ledger
- Import Bundle
- Adopt Imported Asset
- Artifact Transition
- Export Machine Records

## 7.2 Drawers

- Context Drawer
- Evidence Drawer
- Raw Record Drawer
- Why This State Drawer
- Why Blocked Drawer
- Why Denied Drawer
- Provenance Drawer
- Criterion Detail Drawer
- Artifact Detail Drawer
- Transcript Summary Drawer
- Event Detail Drawer
- Diff Drawer

## 7.3 Overlays

- Global Command Palette
- Global Search Results
- Notification Center
- Approval Tray
- Active Run Tray
- Keyboard Shortcut Guide

---

# 8. Cross-cutting component requirements

## 8.1 Every action component must expose

- required authority;
- estimated side effect;
- affected record or resource;
- whether a new immutable event will be created;
- whether approval may be required;
- whether the action can be cancelled or reversed;
- what success will mean.

## 8.2 Every record component must expose

- typed identity;
- version;
- source cell;
- creation time;
- actor;
- state;
- integrity or verification status;
- related source records;
- raw machine representation.

## 8.3 Every derived-view component must expose

- source records;
- projection or adapter version;
- rebuild status;
- freshness;
- stale behavior;
- verification action.

## 8.4 Every failure component must expose

- typed failure class;
- failed boundary;
- side-effect status;
- evidence retained;
- recommended next lawful move;
- whether retry, replan, approval, or escalation is appropriate.

## 8.5 Every agent component must expose

- agent identity and endpoint;
- sponsor;
- model/provider;
- instruction or packet digest;
- tool and authority boundaries;
- budget use;
- permission events;
- transcript evidence;
- termination reason;
- settlement status.

---

# 9. Suggested component namespaces

For implementation and design-system organization:

```text
foundations/
atoms/
  action/
  data/
  feedback/
  identity/
  input/
  navigation/
  status/
molecules/
  context/
  evidence/
  explanation/
  forms/
  status/
  timeline/
organisms/
  admin/
  approval/
  assets/
  audit/
  capability/
  case/
  domain/
  execution/
  federation/
  memory/
  thoth/
templates/
pages/
```

Example component names:

```text
StatusBadge
AssuranceBadge
DualStateIndicator
AvailabilityLadder
AuthorityBoundary
CriterionEvidencePair
ProvenanceChain
WhyBlocked
CaseHorizonBoard
ApprovalDecisionPanel
AgentTaskConsole
RunEvidenceInspector
CapabilityInspector
LedgerIntegrityInspector
```

---

# 10. Atomic coverage check by user journey

| Journey | Primary organisms | Primary pages |
|---|---|---|
| 1. Cell readiness | Readiness Console | P1–P2 |
| 2. Identity and authority | Identity and Authority Header | P3 |
| 3. Ask SEA Forge | Thoth Question Composer, Thoth Answer Panel | P4–P5 |
| 4. Discover assets | Asset Catalog | P6–P7 |
| 5. Domain meaning | Domain Model Workbench | P8–P10 |
| 6. Create case | Case Creation Workbench, Plan Inspector | P11–P13 |
| 7. Navigate case | Case Horizon Board, Case Timeline | P14–P18 |
| 8. Human judgment | Approval Inbox, Approval Decision Panel | P19–P21 |
| 9. Execute/delegate | Execution Console, Agent Task Console | P22–P23 |
| 10. Thoth management | Thoth Manager Panel | P24 |
| 11. Monitor/recover | Operations Monitor | P25 |
| 12. Evidence/audit | Run Evidence Inspector, Criterion Settlement Matrix, Ledger Inspector | P26–P29 |
| 13. Memory/capability | Memory Recall Workspace, Capability Inspector | P30–P32 |
| 14. Projection/artifact/federation | Projection Pipeline, Artifact Inspector, Federation Workbench | P33–P37 |
| 15. Maintain/evolve | Administration Console | P38–P39 |

---

# 11. Settlement for this design stage

This atomic breakdown is sufficiently complete for the next stage when:

1. every epic journey maps to at least one organism and page;
2. repeated concepts use shared atoms and molecules rather than page-specific duplicates;
3. execution and settlement remain separate throughout;
4. source records and derived views remain distinguishable;
5. authority, evidence, and next lawful action can be represented on every operational page;
6. no page requires raw logs as its primary means of understanding state;
7. the component inventory can support guided, operational, and audit density modes.

The next artifact should sequence the page families as stateful flows by defining, for every view:

```text
entry conditions
inputs
user decisions
system decisions
outputs
state mutations
authority checks
evidence produced
settlement conditions
next possible views
failure and recovery transitions
```
