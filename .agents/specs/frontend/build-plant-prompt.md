# Repository-Grounded SEA Forge GUI Implementation Specification and Plan

You are working inside the SEA Forge repository.

Your task is to inspect the actual codebase and then produce a repository-grounded implementation specification and execution plan for the SEA Forge Workbench GUI.

Do **not** implement the GUI in this task.

Do **not** assume the architecture documents accurately describe what already exists in the repository. Treat them as target UX and architectural requirements, then determine how they map onto the actual substrate.

The final outputs must be detailed enough that a separate coding agent can implement the GUI incrementally without inventing architecture, duplicating existing systems, bypassing governance boundaries, or creating unnecessary technical debt.

---

## 1. Required source documents

Read these documents as a connected specification set:

1. `sea-forge-governed-workbench-ux-epic-v0.1.md`
2. `sea-forge-gui-atomic-design-breakdown-v0.1.md`
3. `sea-forge-gui-view-flow-transition-spec-v0.1.md`
4. `sea-forge-primary-path-screen-wireframe-interaction-spec-v0.1.md`
5. `sea-forge-workbench-frontend-architecture-component-contract-v0.1.md`

Also inspect the SEA Forge rewrite specifications already present in the repository, including the specifications covering:

* minimum governed lifecycle;
* full SEA Forge architecture;
* Genesis Self-Model;
* ADLC and ODI;
* Thoth;
* agent connectivity and providers;
* governed delegation;
* ACP;
* SWE_SEED integration;
* settlement and capability;
* artifacts and federation.

Use repository-local versions as authoritative where they exist.

Before producing outputs, locate and follow all applicable:

* `AGENTS.md` files;
* repository contribution instructions;
* architecture decision records;
* crate-level documentation;
* existing plans and specifications;
* coding and testing conventions.

---

## 2. Core operating rule

Inspect before prescribing.

Every implementation recommendation must be grounded in actual repository evidence, such as:

* existing crates;
* modules;
* traits;
* commands;
* query types;
* event types;
* server transports;
* socket protocols;
* configuration structures;
* generated schemas;
* UI code;
* test harnesses;
* build tooling;
* package-management conventions;
* extension points.

For every important claim, cite the relevant repository path and, where useful, the symbol, type, trait, command, or module.

Do not invent file paths, APIs, services, commands, or abstractions because they would make the design look cleaner.

When something cannot be confirmed from the repository, mark it explicitly as:

```text
UNKNOWN — repository evidence not found
```

Do not silently classify unknown work as missing.

---

## 3. Architectural invariants

The GUI must be designed as a governed SEA Forge client, not as a second implementation of SEA Forge.

The frontend may own:

* navigation;
* view composition;
* reversible drafts;
* local preferences;
* filtering and sorting;
* formatting;
* keyboard interactions;
* accessibility;
* non-authoritative client validation hints;
* presentation state.

The frontend must not independently own or recreate:

* authority evaluation;
* identity binding;
* sponsor eligibility;
* sentry activation;
* case reduction;
* settlement evaluation;
* declaration standing;
* capability promotion;
* ledger verification;
* disclosure policy;
* sandbox enforcement;
* agent permission grants;
* artifact maturity transitions;
* federation trust or adoption;
* canonical record mutation.

Preserve these distinctions:

```text
navigation ≠ mutation
draft ≠ proposal
proposal ≠ committed plan
authority approval ≠ successful execution
execution success ≠ accepted settlement
agent termination ≠ settlement
accepted settlement ≠ proven capability
derived view ≠ source truth
imported record ≠ local authority
```

Any recommended architecture that violates one of these distinctions must be rejected or redesigned.

---

## 4. Inspection sequence

Perform the inspection in the following order.

### 4.1 Repository orientation

Identify:

* workspace structure;
* languages;
* build systems;
* package managers;
* primary binaries;
* server process;
* CLI process;
* persistence model;
* event and ledger model;
* existing frontend or desktop code;
* generated-code pipelines;
* test layout;
* release and packaging structure.

Produce a concise repository topology diagram.

### 4.2 Existing GUI and frontend substrate

Search for:

* existing desktop applications;
* Tauri, Wry, WebView, Dioxus, Leptos, React, Vue, Svelte, Next.js, or other frontend dependencies;
* UI bridge crates or packages;
* IPC commands;
* socket clients;
* event subscriptions;
* static assets;
* route modules;
* component libraries;
* Storybook or equivalent;
* frontend test tooling.

Determine whether the GUI should:

1. extend an existing application;
2. replace an incomplete application;
3. create a new application in the existing workspace;
4. remain framework-neutral until another architectural decision is made.

Do not select a framework merely because the target architecture document mentioned one as a reference profile.

### 4.3 Server and transport substrate

Identify the actual interaction surface available to a GUI:

* server binary;
* Unix socket or network protocol;
* request and response envelopes;
* command registry;
* query registry;
* event stream;
* authentication or identity propagation;
* protocol negotiation;
* error representation;
* request correlation;
* idempotency;
* cancellation;
* reconnect behavior.

Determine whether the GUI can use existing server contracts directly or whether a UI bridge is required.

### 4.4 Canonical data and lifecycle contracts

Locate the actual owners of:

* cell identity and readiness;
* actor identity, roles, and sponsorship;
* authority decisions;
* models and semantic validation;
* templates;
* cases;
* plans;
* stages;
* sentries;
* milestones;
* plan items;
* runs;
* sandboxes;
* environments;
* agent sessions;
* ACP permissions;
* evidence;
* settlement;
* memory;
* capability;
* artifacts;
* integrity;
* federation.

For each, identify:

* canonical type;
* storage or ledger representation;
* reducer or projection;
* relevant commands;
* relevant queries;
* relevant events;
* test coverage.

### 4.5 Schema projection capability

Inspect whether canonical Rust types can already be projected into frontend-compatible schemas or generated TypeScript.

Look for:

* Serde schemas;
* JSON Schema generation;
* OpenAPI;
* TypeScript generators;
* protobuf;
* MessagePack schemas;
* custom protocol descriptors;
* versioned envelopes.

Determine whether frontend contracts should be:

* generated directly;
* generated through an intermediate schema;
* manually adapted at a stable boundary;
* deferred because the canonical contracts are not yet ready.

Do not recommend handwritten duplicate enums unless no safer projection path exists, and document the debt if that fallback is unavoidable.

### 4.6 Existing tests and harnesses

Locate reusable infrastructure for:

* unit tests;
* integration tests;
* protocol tests;
* fixture cells;
* ledger fixtures;
* event-stream tests;
* CLI/server tests;
* sandbox tests;
* agent endpoint tests;
* settlement tests;
* capability tests;
* ACP tests;
* SWE_SEED tests.

Determine what can be reused for GUI contract and end-to-end testing.

---

## 5. Required capability-delta analysis

For every major requirement in the five GUI documents, classify the repository state as one of:

```text
EXISTS
PARTIAL
MISSING
CONFLICTS
UNKNOWN
NOT REQUIRED
```

Include at least these areas:

1. Desktop/workbench application shell
2. GUI-to-server transport
3. Protocol negotiation
4. Request correlation
5. Idempotent request-status recovery
6. Query contracts
7. Protected-command contracts
8. Event subscription and durable cursor
9. Event-gap recovery
10. Generated frontend types
11. Cell readiness queries
12. Actor and sponsor context
13. Policy and authority visibility
14. Source/freshness/integrity envelopes
15. Case entry options
16. Local case drafts
17. Server preflight
18. Atomic case commit
19. Case summary
20. Case horizon
21. Sentry and blocker explanations
22. Run detail
23. Scoped cancellation
24. Retry as a new episode
25. Agent-run detail
26. Agent budgets
27. ACP permission workflow
28. Transcript evidence and disclosure
29. SWE_SEED proof integration
30. Settlement detail and criterion matrix
31. Settlement declarations and standing
32. Capability detail and promotion explanation
33. Evidence explorer
34. Integrity inspection
35. Storybook/component fixture infrastructure
36. Frontend accessibility testing
37. Desktop end-to-end testing
38. Packaging and distribution
39. Upgrade/version compatibility
40. Operational observability

For every `PARTIAL`, `MISSING`, or `CONFLICTS` classification, explain:

* what exists;
* what is absent or incompatible;
* why the delta matters;
* the smallest coherent change that closes it;
* which repository module should own that change.

---

## 6. Payment-burden analysis

Estimate implementation burden across:

* engineering time;
* architectural complexity;
* testing burden;
* protocol risk;
* security risk;
* frontend/backend coordination;
* generated-code maintenance;
* release packaging;
* migration burden;
* technical debt;
* opportunity cost.

Do not use vague labels such as “easy” or “hard” alone.

For each major slice, describe the main source of burden and the most affordable implementation path.

Prefer reuse and adaptation over adding new subsystems.

---

## 7. Required output files

Create these files:

```text
.agents/reports/sea-forge-gui-repository-grounding-report.md
.agents/specs/sea-forge-workbench-gui-implementation-spec.md
.agents/plans/sea-forge-workbench-gui-implementation-plan.md
```

Use an existing repository convention instead if the repository already has canonical directories for reports, specifications, or plans. Document any changed output paths.

Do not modify existing implementation files.

---

# Output 1: Repository Grounding Report

File:

```text
.agents/reports/sea-forge-gui-repository-grounding-report.md
```

Include:

## A. Executive conclusion

State:

* what GUI substrate already exists;
* whether the proposed frontend architecture fits it;
* which major architectural assumptions survive;
* which assumptions need correction;
* the most affordable implementation direction.

## B. Repository topology

Include a diagram and concise explanation of:

* binaries;
* crates/packages;
* server;
* transport;
* storage;
* events;
* UI-related modules;
* test infrastructure.

## C. Existing interaction surfaces

Document the actual:

* commands;
* queries;
* events;
* protocols;
* bridge possibilities;
* schema-generation options.

Cite repository paths and symbols.

## D. Canonical ownership matrix

Use a table:

| Concern | Canonical owner | Types/symbols | Commands | Queries | Events | Tests |
| ------- | --------------- | ------------- | -------- | ------- | ------ | ----- |

Cover the governed lifecycle from meaning through capability.

## E. GUI requirement delta matrix

Use the classification system:

```text
EXISTS / PARTIAL / MISSING / CONFLICTS / UNKNOWN / NOT REQUIRED
```

Cite evidence for each classification.

## F. Architecture corrections

Identify any recommendation from the GUI architecture documents that should be changed because of repository reality.

Examples:

* a bridge is unnecessary because the server protocol already supports the renderer;
* generated TypeScript already exists under another package;
* Tauri conflicts with an existing desktop shell;
* event cursor semantics differ from the draft architecture;
* a view model already exists under another name;
* the proposed module boundary duplicates an existing crate.

## G. Risks and unresolved unknowns

Separate:

* confirmed risks;
* unresolved questions;
* repository areas that require deeper implementation-time inspection.

## H. Recommended implementation direction

Provide one primary recommendation and any justified alternatives.

Do not present multiple equal options merely to avoid making a decision.

---

# Output 2: Unified GUI Implementation Specification

File:

```text
.agents/specs/sea-forge-workbench-gui-implementation-spec.md
```

The specification must be repository-grounded and implementation-ready.

Include:

## 1. Purpose and outcome

Define the GUI outcome and primary settlement path.

## 2. Existing substrate to reuse

List exact:

* crates;
* modules;
* traits;
* commands;
* protocol types;
* event types;
* generators;
* test fixtures;
* build tooling.

## 3. Architecture

Define:

* desktop application placement;
* frontend framework only if supported by repository evidence;
* bridge or direct-client boundary;
* query client;
* protected-command client;
* event client;
* local draft storage;
* generated contract flow;
* cache ownership;
* route ownership;
* component ownership.

## 4. Repository module layout

Specify concrete files and directories to add or modify.

Every proposed path must be justified by existing repository organization.

For each new module, state:

* responsibility;
* dependencies;
* prohibited responsibilities;
* public interface;
* tests.

## 5. Protocol and contract mapping

Map each GUI requirement to existing or required:

* query;
* command;
* response;
* event;
* error;
* record reference;
* freshness/integrity metadata.

Do not invent wire contracts if an equivalent already exists.

Where a new server contract is genuinely required, specify it narrowly and identify its canonical owner.

## 6. Route modules

Specify routes for at least:

```text
P2  Readiness
P11 New Case
P12 Case Configuration
P13 Case Preflight
P15 Case Overview
P16 Case Horizon
P22 Run Detail
P23 Agent Task Detail
P27 Settlement Detail
P32 Capability Detail
```

For each route include:

* existing backend source;
* loader/query;
* event subscription;
* protected commands;
* route guards;
* stale-state behavior;
* failure behavior.

## 7. State machines

Define repository-compatible frontend state machines for:

* readiness;
* case draft;
* configuration;
* preflight;
* live case;
* run;
* agent task;
* settlement;
* capability.

Do not duplicate canonical backend reducers.

## 8. View models

Define the exact view models required.

Prefer projections from existing types rather than large frontend-only aggregates.

Identify which view models:

* already exist;
* can be composed client-side;
* should be projected server-side;
* require new narrow query types.

## 9. Component contracts

Map the Atomic Design inventory to concrete implementation modules.

Include at least:

* governed status;
* source freshness;
* dual execution/settlement state;
* authority boundary;
* evidence references;
* case horizon;
* run console;
* agent task console;
* settlement matrix;
* capability promotion panel.

## 10. Draft and cache policy

Specify:

* what may be stored locally;
* draft keys;
* invalidation;
* role/cell cache partitioning;
* secret and transcript prohibitions;
* stale-digest handling.

## 11. Event consistency

Specify:

* actual cursor/ordinal mechanism found in the repository;
* subscription lifecycle;
* duplicate handling;
* gap handling;
* unsupported event handling;
* reconnect behavior;
* authoritative refetch.

## 12. Error taxonomy

Reuse repository error types where possible.

Map them to:

* denial;
* escalation;
* expiry;
* execution failure;
* settlement failure;
* integrity failure;
* transport ambiguity;
* unsupported version.

## 13. Security and disclosure

Specify:

* identity propagation;
* sponsor context;
* disclosure-before-retrieval;
* secret references;
* transcript access;
* file import/export;
* role-switch cache clearing.

## 14. Test specification

Define:

* Rust/server contract tests;
* generated-schema tests;
* frontend state-machine tests;
* component fixtures;
* accessibility tests;
* route integration tests;
* desktop end-to-end tests.

Reuse repository test infrastructure wherever possible.

## 15. Packaging and release

Define:

* build commands;
* development workflow;
* test workflow;
* version negotiation;
* packaging;
* CI integration;
* release artifact;
* upgrade compatibility.

## 16. Acceptance criteria

Make all criteria binary or directly observable.

---

# Output 3: GUI Implementation Plan

File:

```text
.agents/plans/sea-forge-workbench-gui-implementation-plan.md
```

The plan must implement vertical settlements rather than building all backend contracts, then all components, then all screens separately.

Each task must include:

```text
Task ID and title
Goal
Repository evidence
Files/modules affected
Dependencies
Implementation steps
Tests
Acceptance criteria
Failure modes
Debt-prevention notes
Artifacts produced
```

Use stable numbering:

```text
Task 1.1
Task 1.2
Task 2.1
Task 2.2
```

Do not renumber existing task IDs when refining the plan later. Use subnumbers such as `2.3.1` for inserted detail.

Recommended milestone structure—adjust only when repository inspection supports a better sequence:

## Milestone 1 — Contract and transport proof

Prove:

* renderer/desktop host can communicate through the actual governed server surface;
* protocol versions negotiate;
* query, protected command, event, and error envelopes work;
* request status can be recovered after connection ambiguity;
* generated types or safe adapters work.

Settlement:

```text
A minimal diagnostic screen can query readiness,
submit one harmless protected operation,
receive its events,
and recover from a dropped connection without duplication.
```

## Milestone 2 — Design system and shell

Implement:

* application shell;
* cell/actor/policy/integrity context;
* route guards;
* source/freshness presentation;
* status vocabulary;
* evidence drawer;
* keyboard and accessibility foundations.

Settlement:

```text
A user can navigate source-backed fixture views
without losing cell, actor, freshness, or evidence context.
```

## Milestone 3 — Readiness vertical slice

Implement P2 end to end against real contracts.

Settlement:

```text
The GUI distinguishes ready, degraded, stale, blocked,
and integrity-halted states for an intended operation.
```

## Milestone 4 — Case authoring and preflight

Implement P11, P12, and P13.

Settlement:

```text
A local draft becomes a server-preflighted and atomically committed case,
while stale digests and submission ambiguity cannot create duplicate cases.
```

## Milestone 5 — Case overview and horizon

Implement P15 and P16.

Settlement:

```text
The GUI shows source-derived spendable, active, awaiting,
blocked, future, and settled work without client-owned state mutation.
```

## Milestone 6 — Command execution

Implement P22.

Settlement:

```text
A command can be started, observed, cancelled, evidenced,
and settled while execution success remains distinct from settlement acceptance.
```

## Milestone 7 — Agent delegation

Implement P23 and approval integration.

Settlement:

```text
An agent task exposes endpoint, budgets, permissions,
transcript evidence, termination, and settlement without silent fallback.
```

## Milestone 8 — Settlement

Implement P27.

Settlement:

```text
Every criterion links to evidence or explicit absence,
and rejected false-success paths activate lawful recovery.
```

## Milestone 9 — Capability

Implement P31/P32.

Settlement:

```text
Capability state, variation, recovery, regression,
promotion gaps, and next proof paths are grounded in qualifying settlements.
```

## Milestone 10 — Packaging and proof

Implement:

* real desktop packaging;
* release workflow;
* protocol compatibility;
* primary-path E2E;
* failure-path E2E;
* accessibility proof.

Settlement:

```text
A packaged Workbench completes the full primary path
and all required failure scenarios against a real fixture cell.
```

---

## 8. Required proof scenarios

The final specification and plan must include these executable scenarios:

### Scenario 1 — Happy command path

```text
ready
→ create/configure/preflight case
→ commit
→ start command
→ execution succeeds
→ settlement accepted
→ capability receives one qualifying observation
```

### Scenario 2 — False-success correction

```text
execution succeeds
→ committed criterion fails
→ settlement rejected
→ remediation item activates
```

### Scenario 3 — Approval escalation

```text
preflight escalates
→ eligible approver accepts exact request digest
→ case commit resumes
```

### Scenario 4 — Agent permission denial

```text
agent requests permission
→ permission denied
→ delegation continues within remaining grant
→ termination recorded
→ settlement evaluated independently
```

### Scenario 5 — Integrity halt

```text
integrity verification fails
→ affected mutations disabled
→ repair/verification path shown
→ history remains append-only
```

### Scenario 6 — Ambiguous submission

```text
commit request sent
→ connection drops
→ client queries request status
→ existing result recovered
→ no duplicate case
```

### Scenario 7 — Stale preflight

```text
preflight passes
→ policy or resource digest changes
→ commit is rejected as stale
→ diff displayed
→ new preflight required
```

### Scenario 8 — Event gap

```text
live view misses event sequence
→ UI marks state stale
→ authoritative view refetched
→ continuity restored
```

---

## 9. Debt-prevention rules

The generated specification and plan must explicitly prevent:

* direct renderer mutation of canonical files;
* frontend-owned case or run status;
* frontend authority or settlement logic;
* handwritten copies of canonical enums without a documented fallback;
* broad retrieval followed by client redaction;
* optimistic approval, cancellation, settlement, or capability;
* silent provider fallback;
* silent event loss;
* retry after transport error without request-status recovery;
* one generic status field collapsing multiple state domains;
* large new framework dependencies without repository-grounded justification;
* new abstractions that merely rename existing ones;
* speculative backend endpoints added for UI convenience;
* rebuilding existing server projections inside the frontend;
* changing SEA Forge semantics to make the GUI easier.

---

## 10. Quality bar

The outputs are acceptable only if another coding agent can answer all of these questions from them:

1. Which existing modules should be reused?
2. Which contracts already exist?
3. Which contracts are genuinely missing?
4. Which crate or module owns each missing contract?
5. Which files must be added or changed?
6. Which logic belongs to the server versus frontend?
7. How is state synchronized after reconnect?
8. How are stale inputs detected?
9. How are duplicate protected operations prevented?
10. How is every displayed status grounded in source records?
11. How is execution kept distinct from settlement?
12. How is agent termination kept distinct from settlement?
13. How is capability kept grounded in qualifying outcomes?
14. What test proves each implementation slice?
15. What result counts as settlement for each milestone?

Do not begin production implementation.

Inspect the repository thoroughly, produce the three requested artifacts, and end with a concise summary of:

* architecture chosen;
* major existing substrate reused;
* genuinely missing capabilities;
* highest-risk integration boundary;
* recommended first implementation task.
