
# SEA Forge Primary Settlement Path
# Screen-Level Wireframe and Interaction Specification

**Status:** Draft v0.1  
**Depends on:**  
- `sea-forge-governed-workbench-ux-epic-v0.1.md`
- `sea-forge-gui-atomic-design-breakdown-v0.1.md`
- `sea-forge-gui-view-flow-transition-spec-v0.1.md`

**Purpose:** Define the screen composition, interaction hierarchy, progressive disclosure, operational states, keyboard behavior, responsive behavior, and acceptance criteria for SEA Forge's primary governed-work path.

**Primary path:**

```text
P2 Readiness
→ P11 New Case
→ P12 Case Configuration
→ P13 Case Preflight
→ P15 Case Overview
→ P16 Case Horizon
→ P22 Run Detail or P23 Agent Task Detail
→ P27 Settlement Detail
→ P32 Capability Detail
```

**Boundary:** This specification defines the user-facing screen contract. It does not prescribe React, Tauri, server transport, CSS framework, or backend persistence.

---

# 0. Experience outcome

A user should be able to move from a known installation state to a settled outcome without needing to reconstruct system truth from logs, source files, or scattered CLI output.

At every step, the interface must expose:

```text
current state
→ why that state is true
→ evidence supporting it
→ next lawful action
→ authority/payment required
→ what settlement will mean
```

The interface must reduce the most dangerous representational errors:

- “The service is running, so the installation is ready.”
- “The asset is declared, so it is available.”
- “The plan was accepted, so execution is authorized.”
- “The command succeeded, so the work is complete.”
- “The agent said it finished, so the outcome is valid.”
- “The outcome settled once, so capability is proven.”
- “The dashboard shows it, so it must be source truth.”

---

# 1. Shared application shell

## 1.1 Desktop shell

Recommended desktop composition:

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Global bar: Cell | Actor/Role | Policy | Integrity | Search | Inbox | User │
┌──────────────┬───────────────────────────────────────────┬───────────────────┤
│ Primary nav  │ Main content                              │ Context / Evidence│
│              │                                           │ drawer (optional) │
│ Inbox        │ Breadcrumbs                               │ Why this state    │
│ Memory       │ Page header                               │ Evidence refs     │
│ Artifacts    │ Primary screen regions                    │ Provenance        │
│ Domain Models│                                           │ Raw record        │
│ Capabilities │                                           │                   │
│ Cases        │                                           │                   │
│ Federation   │                                           │                   │
│ Administration│                                          │                   │
├──────────────┴───────────────────────────────────────────┴───────────────────┤
│ Optional persistent status bar: live connection | event cursor | stale state│
└──────────────────────────────────────────────────────────────────────────────┘
```

> **Primary nav labels are canonical.** The navigation uses the canonical
> kernel vocabulary: `Inbox` (approvals/human tasks), `Memory` (capability
> memory), `Artifacts` (artifact IP/capital), `Domain Models`
> (DomainForge models), `Capabilities` (capability records), `Cases`
> (case plan runs), `Federation` (cells/bundles), and `Administration`
> (policy/integrity config). Earlier draft labels (`Readiness`, `Thoth`,
> `Assets`, `Models`, `Operations`, `Evidence`, `Capability`, `Admin`)
> are retired; this shell is deliberately reduced to the primary-path
> surfaces and does not present the full administrative vocabulary.

Recommended width behavior:

| Viewport | Navigation | Evidence panel | Main behavior |
|---|---:|---:|---|
| ≥ 1440 px | 248–280 px | 360–420 px persistent | Three-column operational mode |
| 1100–1439 px | 224–248 px | Overlay/drawer | Two-column default |
| 768–1099 px | 64 px icon rail | Full-height overlay | Main regions stack selectively |
| < 768 px | Hidden behind menu | Full-screen sheet | Monitoring, approvals, and inspection prioritized |

**Recommended product constraint:** complex `.sea` authoring, case topology configuration, and large evidence matrices should be desktop-first. Small-screen access should still support readiness inspection, approvals, monitoring, cancellation, settlement review, and notifications.

## 1.2 Global bar

Left to right:

1. **Cell Context Selector**
   - cell name;
   - cell ID on expansion;
   - readiness status;
   - stale/degraded indicator.

2. **Actor Context Selector**
   - acting identity;
   - role;
   - sponsor when applicable.

3. **Policy Context Chip**
   - active bundle/version;
   - allow/deny/degraded load state;
   - digest on expansion.

4. **Integrity Indicator**
   - assurance level;
   - last verification;
   - direct route to P29.

5. **Global Search**
   - cases;
   - runs;
   - evidence;
   - assets;
   - artifacts;
   - capabilities;
   - exact IDs/hashes.

6. **Approval and Human Task Inbox**
   - pending count;
   - expiring count;
   - ACP permission count.

7. **Active Work Indicator**
   - active runs;
   - interrupted runs;
   - capacity saturation.

8. **Density Mode**
   - Guided;
   - Operational;
   - Audit.

## 1.3 Page header contract

Every primary-path screen includes:

```text
Breadcrumb
Page title
Typed status
One-sentence purpose/current-state explanation
Primary lawful action
Secondary actions
Source/freshness indicator
```

Example:

```text
Cases / CASE-01 / RUN-07

Execute generated contracts
Execution: succeeded     Settlement: evaluating

The command completed. SEA Forge is still evaluating the committed criteria.

[Open settlement] [Inspect evidence] [Retry unavailable]
```

## 1.4 Context and evidence drawer

The right-side drawer is shared across the path. It has four tabs:

1. **Why**
   - why this state;
   - governing rule;
   - blocker or satisfied condition;
   - next lawful move.

2. **Evidence**
   - evidence refs;
   - verification state;
   - content-access controls.

3. **Provenance**
   - origin → plan → authority → run → settlement → capability.

4. **Record**
   - canonical typed record;
   - version;
   - source cell;
   - machine-readable export.

The drawer never mutates the underlying record unless an explicit protected action is launched from it.

## 1.5 Global keyboard contract

| Shortcut | Action |
|---|---|
| `Ctrl/Cmd + K` | Command palette |
| `/` | Focus page search/filter |
| `g r` | Readiness |
| `g c` | Cases |
| `g o` | Operations |
| `g e` | Evidence |
| `g a` | Approvals/inbox |
| `g p` | Capabilities |
| `[` / `]` | Previous/next item in current collection |
| `e` | Toggle evidence drawer |
| `w` | Open “Why this state” |
| `Esc` | Close drawer/dialog; never undo committed state |
| `?` | Keyboard help |

Protected actions never use a single unmodified letter shortcut while a text input is focused.

---

# 2. Cross-screen continuity

## 2.1 Persistent journey ribbon

During case creation and execution, the page header may expose a compact lifecycle ribbon:

```text
Meaning ✓
Plan ✓
Authority ✓
Execution ●
Evidence ○
Settlement ○
Capability ○
```

Rules:

- The ribbon shows **state**, not navigation completion.
- A user may navigate backward without changing a completed lifecycle state.
- A stage becomes complete only from source records.
- A failed stage remains visible; it does not disappear behind the current step.

## 2.2 Origin preservation

Corrective detours preserve:

- draft data;
- originating page;
- intended action;
- selected asset/model/endpoint;
- prior preflight result;
- governing digests.

When the user returns, SEA Forge must indicate whether the preflight remains valid:

```text
Preflight invalidated:
Policy bundle changed from 7c2… to 9a1…
Review changes before committing.
```

## 2.3 Primary action hierarchy

Each screen has at most one visually dominant action.

Priority:

1. Resolve blocker required for the intended path.
2. Perform the next lawful lifecycle transition.
3. Inspect evidence.
4. Navigate elsewhere.
5. Administrative or destructive actions.

No screen should present “Run,” “Approve,” “Retry,” and “Terminate” with equal emphasis.

---

# 3. WF-P2 — Readiness Overview

**Route:** `/cells/:cellId/readiness`  
**Job:** Help the operator know whether the installation is safe and capable enough for the intended work.  
**Primary decision:** Continue, repair a blocker, or knowingly operate under an allowed degraded condition.  
**Primary output:** Verified readiness result and current self-model snapshot.

## 3.1 Desktop wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Cell: Foundry-01   Readiness: READY WITH 2 LIMITATIONS       [Run checks]   │
│ Last verified 4 min ago · Snapshot SM-019 · Policy PB-14 · Ledger witnessed │
├──────────────────────────────────────────────────────────────────────────────┤
│ Intended work                                                                │
│ [ Start governed case ▼ ]                                                    │
│ Readiness is evaluated against the operation you intend, not one global ping.│
├───────────────────────┬──────────────────────────────────────────────────────┤
│ Critical foundations  │ Operational capabilities                            │
│ ✓ Cell identity       │ ✓ Case engine                                       │
│ ✓ Actor resolved      │ ✓ Jail sandbox                                      │
│ ✓ Policy loaded       │ ! TLA+ projection unavailable                       │
│ ✓ Ledger integrity    │ ! Anthropic endpoint not probed                     │
│ ✓ Genesis model       │ ✓ Agent delegation                                  │
│ ✓ Self-model snapshot │ ✓ Settlement authorities                           │
├───────────────────────┴──────────────────────────────────────────────────────┤
│ Current affordances                                                          │
│ [Create case] [Ask Thoth] [Browse templates] [Repair 2 limitations]          │
├──────────────────────────────────────────────────────────────────────────────┤
│ Recent changes                                                               │
│ Extension endpoint.openai upgraded → snapshot stale → rebuilt successfully   │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 3.2 Region hierarchy

### R1 — Readiness header
Components:

- Governed Status Pill;
- snapshot reference;
- freshness badge;
- policy and integrity summary;
- primary `Run checks` action.

### R2 — Intended-work selector
Allows readiness to be evaluated against:

- create case;
- execute command;
- delegate agent;
- project model;
- run specification pipeline;
- transition artifact;
- import bundle;
- custom operation.

Why: an installation can be generally healthy but unable to support a specific path.

### R3 — Critical foundations
Ordered hard gates:

1. cell;
2. identity/sponsor;
3. policy;
4. ledger integrity;
5. bundled models;
6. current self-model snapshot.

A hard-gate failure appears first and suppresses any false “ready” banner.

### R4 — Operational capabilities
Shows:

- available;
- degraded;
- unavailable;
- stale;
- unproven.

Each row includes `Why`, evidence count, and repair action.

### R5 — Current affordances
Only actions that pass current hard guards appear as primary affordances.

Unavailable actions are not silently removed. They appear in a secondary “Other possibilities” disclosure with the missing path.

### R6 — Recent changes
Shows changes that affected readiness:

- extension/config mutation;
- policy reload;
- model drift;
- failed probe;
- ledger verification;
- snapshot rebuild.

## 3.3 Progressive disclosure

Default Guided view:

- readiness summary;
- blockers;
- next actions.

Operational expansion:

- subsystem versions;
- toolchain probe times;
- endpoint states;
- environment compatibility.

Audit expansion:

- source hashes;
- validation records;
- policy digest;
- ledger proof;
- snapshot composition.

## 3.4 States

### Loading
Do not show one indefinite spinner. Show ordered checks:

```text
1/6 Identity resolved
2/6 Policy loaded
3/6 Ledger verifying…
4/6 Genesis model waiting
```

### Empty
A new cell displays:

```text
This cell has no verified readiness result yet.
Run the checks before creating governed work.
```

### Ready
Use `Ready` only when required hard gates pass for the selected intended work.

### Ready with degradation
Must name:

- degraded subsystem;
- affected operations;
- accepted policy basis;
- next repair action.

### Blocked
The dominant action becomes the nearest repair route, not “Continue anyway,” unless policy explicitly permits degraded operation.

### Stale
Continue to show prior verified result, labeled:

```text
Last verified result: Ready
Current state: Stale after endpoint registry change
```

### Integrity halted
Hide mutating affordances and expose:

- affected scope;
- last verified checkpoint;
- P29 Integrity;
- P39 Maintenance.

## 3.5 Keyboard

- `r`: run checks;
- `i`: focus intended-work selector;
- `b`: first blocker;
- `Enter`: open focused subsystem;
- `c`: create case only when enabled.

## 3.6 Acceptance criteria

- Readiness is operation-sensitive.
- A stale prior result is never displayed as current.
- Hard-gate failure cannot be visually outweighed by many green optional checks.
- Each degraded/unavailable state shows affected work and corrective path.
- `Create case` transitions to P11 without changing system readiness.
- Readiness verification records are accessible from the evidence drawer.

---

# 4. WF-P11 — New Case

**Route:** `/cells/:cellId/cases/new`  
**Job:** Help the user choose the correct starting representation for governed work.  
**Primary decision:** Which entry source best matches the work?  
**Primary output:** A reversible CaseDraft skeleton.

## 4.1 Desktop wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ New governed case                                              Draft only    │
│ Choose how the work is represented. No authority is conferred at this step.  │
├──────────────────────────────────────────────────────────────────────────────┤
│ What are you trying to do?                                                   │
│ [ Describe the intended outcome or problem…                               ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│ Recommended starting paths                                                   │
│ ┌────────────────────┐ ┌────────────────────┐ ┌────────────────────────────┐ │
│ │ ADLC case          │ │ From template      │ │ External plan proposal     │ │
│ │ Develop an agent   │ │ Reuse governed work│ │ Validate untrusted plan    │ │
│ │ [Select]           │ │ [Browse]           │ │ [Upload]                   │ │
│ └────────────────────┘ └────────────────────┘ └────────────────────────────┘ │
│ ┌────────────────────┐ ┌────────────────────┐ ┌────────────────────────────┐ │
│ │ Sequential agents  │ │ Concurrent agents  │ │ Specification pipeline     │ │
│ │ Ordered delegation │ │ Parallel + rollup  │ │ Source → runtime → proof   │ │
│ └────────────────────┘ └────────────────────┘ └────────────────────────────┘ │
├──────────────────────────────────────────────────────────────────────────────┤
│ Other path: [Known intent] [ODI-grounded ADLC] [Start blank—advanced]        │
├──────────────────────────────────────────────────────────────────────────────┤
│                                             [Cancel] [Continue to configure] │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 4.2 Region hierarchy

### R1 — Purpose input
Captures:

- intended outcome;
- problem;
- need;
- desired direction;
- optional job/requirement origin.

The input is explanatory context, not authority or settlement criteria.

### R2 — Recommended entry cards
Cards display:

- what the entry path is for;
- what it creates;
- required assets;
- expected payment/complexity;
- current availability;
- source version.

### R3 — Other paths
Less common or advanced starting modes.

### R4 — Selection explanation
When a card is selected, the right drawer explains:

- generated structure;
- what remains editable;
- what is immutable later;
- expected authority surfaces;
- likely next screens.

## 4.3 Recommendation behavior

SEA Forge may recommend a path from the purpose text and available assets, but must label it:

```text
Recommended from your description
—not selected automatically
```

No LLM recommendation may create a plan without explicit user selection and validation.

## 4.4 States

### No available templates
Keep known-intent and external-plan paths available where possible. Explain why the template route is unavailable.

### Unsupported external plan
Do not reject upload based only on extension. Parse in a safe non-executing path and report schema/version issues.

### Readiness changed
If an entry mode depends on a capability that became stale, mark the card and route to P2/P7.

### Draft recovery
If a local draft exists:

```text
Resume draft from 14:32
or
Start a new draft
```

## 4.5 Keyboard

- Arrow keys navigate entry cards.
- `Space` selects a card.
- `Enter` selects and continues.
- `Ctrl/Cmd + S` saves local draft.
- `Esc` prompts only when unsaved draft content exists.

## 4.6 Acceptance criteria

- Entry modes are explained by user outcome, not crate name.
- Choosing a path creates only draft state.
- Unavailable paths show missing affordances.
- Recommendations do not auto-select or auto-commit.
- Purpose text survives selector detours.
- Continue is disabled with an explicit reason until one entry mode is selected.

---

# 5. WF-P12 — Case Configuration

**Route:** `/cells/:cellId/cases/new/configure`  
**Job:** Convert the selected case shape into a complete, reviewable plan proposal.  
**Primary decision:** What meaning, resources, criteria, delegates, environments, and boundaries should govern the case?  
**Primary output:** Fully instantiated CasePlanDraft.

## 5.1 Desktop wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Configure case: Build repair agent                         Draft · 68% valid │
│ Entry: ADLC case@0.1.0 · Domain: repair.system@1.2.0                        │
├──────────────────────┬───────────────────────────────────┬───────────────────┤
│ Configuration steps  │ Active configuration              │ Draft health      │
│                      │                                   │                   │
│ ✓ Purpose & owner    │ Stage: Settlement criteria        │ 12 valid          │
│ ✓ Domain model       │                                   │  3 incomplete     │
│ ● Criteria           │ Desired outcome                   │  1 unavailable    │
│ ○ Environment        │ [sea:outcome.ReduceRepairTime ▼]  │                   │
│ ○ Delegates          │                                   │ Missing path      │
│ ○ Limits             │ Criterion                         │ ACP endpoint is    │
│ ○ Review             │ [Repair test suite passes      ]  │ not probed.        │
│                      │ Evaluator [repair.tests ▼]         │ [Open endpoint]    │
│                      │ Evidence [JUnit report ▼]          │                   │
│                      │ Origin [Desired outcome ▼]         │                   │
│                      │ [+ Add criterion]                  │                   │
├──────────────────────┴───────────────────────────────────┴───────────────────┤
│ [Back]                    [Save draft]           [Continue to preflight →]   │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 5.2 Layout model

Three regions:

1. **Configuration navigation**
   - ordered sections;
   - validity status;
   - no implication that order equals runtime execution order.

2. **Active editor**
   - forms, selectors, topology, criteria, limits.

3. **Draft health rail**
   - hard errors;
   - incomplete required data;
   - warnings;
   - unavailable assets;
   - estimated authority/payment burden;
   - repair shortcuts.

## 5.3 Configuration sections

### C1 — Purpose and ownership
Inputs:

- case name;
- intent summary;
- desired outcome;
- case owner;
- sponsor where automated management is planned;
- origin refs.

### C2 — Domain model
Inputs:

- validated model/version;
- concept refs;
- drift policy;
- model limitations.

### C3 — Stages and plan items
Displays template-generated structure.

Allowed edits depend on entry source:

- enable/disable optional items;
- fill parameters;
- add permitted discretionary templates;
- not arbitrary mutation of protected template semantics without converting to a proposal.

### C4 — Settlement criteria
Each criterion includes:

- statement;
- criterion type;
- evaluator or verification method;
- expected result;
- required evidence;
- reliability requirement where applicable;
- origin ref;
- immutable snapshot preview.

### C5 — Environments and sandboxes
Keep separate:

```text
Environment = tools and evaluators
Sandbox = isolation boundary
```

Show compatibility and missing tools.

### C6 — Delegates
For each agent task:

- endpoint;
- provider/model;
- instruction packet source;
- continuation mode;
- transcript retention;
- SWE_SEED harness state;
- authority and tool surfaces.

### C7 — Limits and payment boundaries
Inputs:

- timeout;
- concurrency;
- turns;
- token budget;
- manager iterations;
- retention;
- network posture;
- allowed tools;
- optional cost estimate where the provider exposes one.

### C8 — Review
Summary before P13, but still editable.

## 5.4 Progressive disclosure

Guided mode:

- purpose;
- domain;
- required criteria;
- recommended environment/delegate;
- key limits.

Operational mode:

- complete topology;
- sentries;
- evaluators;
- exact authority surfaces.

Audit mode:

- source hashes;
- template expansion;
- descriptor/configuration digests;
- schema versions;
- expected record shapes.

## 5.5 Validation behavior

Validation categories remain separate:

```text
Error       — cannot preflight
Incomplete  — required input absent
Unavailable — selected path cannot currently execute
Warning     — permitted but risky/degraded
Notice      — explanatory
```

Selecting a validation result scrolls and focuses the exact field.

Cross-field validation examples:

- criterion evaluator not supplied by selected environment;
- agent endpoint model incompatible with requested tool protocol;
- sentry references optional item that was disabled;
- desired-outcome concept does not resolve;
- sandbox weaker than required operation;
- SoD impossible with selected actors;
- turn budget below minimum template requirement.

## 5.6 States

### Loading asset metadata
Keep editable fields available. Mark unresolved selectors as “checking,” not invalid.

### Asset becomes stale
Preserve selected value but block preflight:

```text
Selected endpoint configuration changed.
Review endpoint v2.1 before continuing.
```

### Autosave
Autosave is local/draft only. Header shows:

```text
Saved locally 8 sec ago
Not committed to SEA Forge
```

### Multi-user conflict
For v0.1, prefer single-editor draft ownership. If another actor submits a changed governed proposal, show a diff; do not merge silently.

## 5.7 Keyboard

- `Alt + ↑/↓`: previous/next configuration section.
- `Ctrl/Cmd + S`: save draft.
- `Ctrl/Cmd + Enter`: validate current section.
- `F8` / `Shift + F8`: next/previous validation issue.
- `Ctrl/Cmd + Shift + P`: open preflight when eligible.

## 5.8 Acceptance criteria

- Criteria cannot exist without an origin state (`resolved`, `explicitly unknown`, or `not required by type`).
- Environment and sandbox are visually and semantically separate.
- Agent limits are visible before P13.
- Errors and warnings are never flattened.
- Draft autosave cannot be mistaken for case commitment.
- Every selected asset is version-pinned.
- P13 cannot open while blocking validation errors remain.

---

# 6. WF-P13 — Case Preflight

**Route:** `/cells/:cellId/cases/new/preflight`  
**Job:** Let the user understand the exact governed contract before it becomes real.  
**Primary decision:** Commit, return to edit, or resolve an escalation/blocker.  
**Primary output:** Immutable case/plan/criteria records or governed denial/escalation.

## 6.1 Desktop wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Preflight: Build repair agent                           READY TO COMMIT       │
│ Nothing has executed. This review defines what SEA Forge will govern.         │
├──────────────────────────────────────────────────────────────────────────────┤
│ Contract summary                                                             │
│ Desired outcome  Reduce mean repair resolution time                           │
│ Domain model     repair.system@1.2.0 · hash 4f2…                              │
│ Template         adlc_case@0.1.0                                              │
│ Owner / sponsor  Sam / service:thoth-manager                                  │
│ Completion       Required items settled; no active work                       │
├──────────────────────┬──────────────────────┬─────────────────────────────────┤
│ Work shape           │ Authority/payment    │ Settlement                      │
│ 7 stages             │ 4 protected surfaces │ 12 immutable criteria           │
│ 18 plan items        │ 2 approvals possible │ 3 independent declarations      │
│ 4 agent tasks        │ Jail sandbox         │ ODI origin coverage: 100%       │
│ 2 human tasks        │ Network: allowlisted │ [Inspect criteria matrix]       │
├──────────────────────┴──────────────────────┴─────────────────────────────────┤
│ Preflight results                                                            │
│ ✓ Model references resolve                                                   │
│ ✓ Sentries satisfiable                                                       │
│ ✓ Environments provide evaluators                                             │
│ ! Anthropic endpoint validated, not demonstrated                              │
│ ✓ Creation authority allows                                                  │
├──────────────────────────────────────────────────────────────────────────────┤
│ [← Edit configuration] [Export proposal]            [Commit governed case]   │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 6.2 Content hierarchy

1. **Explicit non-execution notice**
   - “Nothing has executed.”

2. **Contract summary**
   - desired outcome;
   - source/template/model;
   - ownership/sponsorship;
   - completion semantics.

3. **Three burden panels**
   - work shape;
   - authority/payment;
   - settlement/proof.

4. **Preflight results**
   - hard passes;
   - degradations;
   - warnings;
   - denials;
   - escalations.

5. **Commit action**
   - includes impact preview.

## 6.3 Commit confirmation

The final commit dialog is not a generic “Are you sure?”

```text
Commit this governed case?

This will create:
• Case CASE-…
• Plan version 1
• 18 plan items
• 12 immutable settlement criteria
• pinned model/template/environment/endpoint references

This will not:
• start execution
• approve protected operations
• accept agent claims
• guarantee settlement

[Cancel] [Commit]
```

## 6.4 Escalation state

If case creation or initial activation escalates:

```text
Preflight complete — approval required

Approval needed:
Use endpoint prod-anthropic for external_api action

[Open approval request] [Return to edit]
```

Do not present “Commit” as pending without explanation.

## 6.5 Drift during review

If any governing digest changes:

- freeze commit;
- show diff;
- identify affected sections;
- rerun only relevant checks plus integrity-sensitive checks;
- retain prior result as historical draft evidence, not current preflight.

## 6.6 Keyboard

- `j/k`: move through checks.
- `Enter`: expand focused check.
- `e`: open evidence.
- `Ctrl/Cmd + Enter`: open commit dialog only when ready.
- `Alt + ←`: return to P12 without losing draft.

## 6.7 Acceptance criteria

- The screen states explicitly that nothing has executed.
- Work, authority/payment, and settlement burden are all visible.
- Commit output is enumerated before confirmation.
- Warnings cannot visually resemble passes.
- A changed digest invalidates commit.
- Escalation creates a traceable approval route.
- Successful commit routes to P15 and records the exact contract reviewed.

---

# 7. WF-P15 — Case Overview

**Route:** `/cells/:cellId/cases/:caseId`  
**Job:** Give a concise, trustworthy answer to “What is this case, where is it now, and what deserves attention?”  
**Primary decision:** Enter the current work horizon, resolve attention items, or inspect settlement/history.  
**Primary output:** Navigation and explicit governed action requests.

## 7.1 Desktop wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Build repair agent                  ACTIVE · 3 of 7 milestones settled       │
│ Desired outcome: Reduce repair resolution time                               │
│ Owner Sam · Plan v1 · ADLC · Domain repair.system@1.2.0                     │
├──────────────────────────────────────────────────────────────────────────────┤
│ Current state                                                                │
│ 2 spendable now · 1 active · 1 approval · 2 blocked · 8 future               │
│ [Open case horizon]                              [Ask Thoth to assess case]   │
├──────────────────────┬───────────────────────┬───────────────────────────────┤
│ Needs attention      │ Recent settlement     │ Capability effect             │
│ Approval expires 2h  │ Simulation rejected   │ Agent delegation: attempted   │
│ Endpoint probe stale │ Basis: test failures  │ Repair validation: +1 qual.   │
│ [Resolve]            │ [Inspect]             │ [Open capability]             │
├──────────────────────┴───────────────────────┴───────────────────────────────┤
│ Milestone trajectory                                                         │
│ Frame ✓ → Form ✓ → Build ● → Activate ○ → Governance ○                      │
├──────────────────────────────────────────────────────────────────────────────┤
│ Recent events                                                                │
│ 14:36 Run-07 settlement rejected → design remediation activated              │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 7.2 Regions

### R1 — Case identity
Includes:

- case name/ID;
- state;
- desired outcome;
- owner/sponsor;
- plan/model/template refs.

### R2 — Current horizon summary
Counts and one dominant action:

- spendable now;
- active;
- awaiting approval;
- blocked;
- future;
- settled.

### R3 — Needs attention
Only actionable issues:

- approval expiry;
- stale dependency;
- interrupted run;
- integrity problem;
- unsatisfied human task;
- manager cap exhaustion.

### R4 — Latest settlement
Shows consequence, not activity.

### R5 — Capability effect
Uses conservative language:

- no contribution;
- attempted;
- qualifying observation;
- promoted;
- regression/quarantine.

### R6 — Milestone trajectory
This is not a linear project bar. It may show reactivation/spiral indicators.

### R7 — Recent events
Small, causal timeline.

## 7.3 Actions

Primary: `Open case horizon`.

Secondary:

- inspect plan;
- inspect timeline;
- invoke bounded Thoth manager;
- reopen/terminate where eligible;
- add discretionary item;
- export case record.

Destructive or administrative actions remain in an overflow menu with impact preview.

## 7.4 States

### Parked
Header explains why and the exact restart affordance.

### Completed
Dominant action becomes `Inspect settlement and capability`, not `Run again`.

### Terminated
No restart masquerading as resume. Offer governed reopen/new case only if policy permits.

### Inconsistent derived summary
Show source record state and route to integrity; do not synthesize counts.

## 7.5 Keyboard

- `h`: open horizon.
- `t`: timeline.
- `p`: plan.
- `s`: latest settlement.
- `m`: Thoth manager, when eligible.
- `a`: first attention item.

## 7.6 Acceptance criteria

- The page prioritizes current consequence and attention over activity volume.
- The current horizon count resolves to P16 source items.
- Capability effect never overstates a single settlement.
- Milestones can display reactivation.
- No control directly edits case state.
- Every summary card exposes its source/freshness.

---

# 8. WF-P16 — Case Horizon

**Route:** `/cells/:cellId/cases/:caseId/horizon`  
**Job:** Show the currently spendable work paths and why other work is active, waiting, blocked, or future.  
**Primary decision:** Which lawful path should be attempted or resolved next?  
**Primary output:** Item activation, blocker resolution, approval navigation, or inspection.

## 8.1 Desktop wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Case horizon: Build repair agent                     [Board] [List] [Graph]  │
│ 2 spendable paths · Current stage pressure: Build                             │
├──────────────────────────────────────────────────────────────────────────────┤
│ Filters: [Stage ▼] [Item kind ▼] [Actor ▼] [State ▼]   Search [/________]   │
├─────────────────┬─────────────────┬─────────────────┬─────────────────┬────────────────────┤
│ SPENDABLE NOW   │ ACTIVE          │ BLOCKED         │ WAITING         │ FUTURE / SETTLED   │
│                 │                 │                 │                 │                    │
│ ┌─────────────┐ │ ┌─────────────┐ │ ┌─────────────┐ │ ┌─────────────┐ │ Milestone: Frame ✓ │
│ │Run repair   │ │ │Agent writes │ │ │Deploy test  │ │ │Awaiting     │ │ Milestone: Form ✓  │
│ │simulation   │ │ │adapter      │ │ │environment  │ │ │approval     │ │                    │
│ │Criteria: 3  │ │ │Turn 4/12    │ │ │Blocked by:  │ │ │TTL 18h      │ │ Future: Activate   │
│ │Jail sandbox │ │ │Tokens 31%   │ │ │failed sentry│ │ │[Expedite]   │ │ Sentry: Build OK   │
│ │[Start]      │ │ │[Open]       │ │ │[Re-run]     │ │ │             │ │                    │
│ └─────────────┘ │ └─────────────┘ │ └─────────────┘ │ └─────────────┘ │ Settled: Model ✓   │
│                 │                 │                 │                 │                    │
├─────────────────┴─────────────────┴─────────────────┴─────────────────┴────────────────────┤
│ Selected item drawer: Why spendable · Authority · Criteria · Evidence needed               │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

> **BLOCKED vs WAITING are distinct states with distinct resolution
> semantics.** BLOCKED means a ready action failed its sentry or authority
> gate and needs operator intervention to re-run or escalate. WAITING means
> the action is parked on an external dependency (approval TTL, human task,
> upstream case) and resolves when that dependency clears. They MUST be
> presented as separate columns/tabs and never collapsed into a combined
> "BLOCKED/WAITING" view.

## 8.2 View modes

### Board
Best for operational overview.

### List
Best for accessibility, sorting, and audit density.

### Graph
Optional advanced view showing:

- sentry/event relationships;
- stages;
- milestones;
- reactivation edges.

Graph is a projection, never the only control surface.

## 8.3 Card contract

Every plan-item card shows:

- name and kind;
- stage;
- typed state;
- actor/delegate;
- authority status;
- environment/sandbox;
- criteria count;
- primary blocker or activation reason;
- next lawful action.

Agent card additionally shows:

- endpoint/model;
- turns/tokens;
- permission wait.

Run card additionally shows:

- run episode;
- elapsed time;
- execution/settlement dual state.

## 8.4 “Spendable now” logic

An item belongs in Spendable Now only when:

- sentry/entry criteria pass;
- required resource is available;
- action is permitted to be requested;
- payment/limits are present;
- settlement access exists;
- no hard integrity or compatibility block exists.

This does **not** mean execution authority is pre-granted. The card action may still open authority evaluation or approval.

## 8.5 Interaction behavior

- Clicking a card selects it and opens the item drawer.
- Dragging may reorder a personal visual preference only; it cannot change case state or priority unless a separate governed reprioritization operation exists.
- `Start` creates a protected activation/run request.
- `Resolve` opens the exact blocker route.
- `Add discretionary item` launches a governed proposal dialog.

## 8.6 Empty states

### No spendable items, active work exists
```text
No new path is spendable while 2 runs are active.
[Open active work]
```

### No spendable or active items, approval exists
```text
The case is waiting for human judgment.
[Open approval]
```

### Stalled
```text
No sentry can currently activate.
Possible paths:
• Resolve missing environment
• Replan unsatisfiable sentry
• Invoke bounded Thoth manager
```

### Satisfied
Route to completion/settlement summary rather than showing an empty board.

## 8.7 Responsive behavior

At narrower widths:

- replace columns with segmented state tabs;
- preserve selected item drawer as full-screen sheet;
- keep spendable count visible;
- never horizontally compress cards until evidence/state labels become unreadable.

## 8.8 Keyboard

- `1–5`: state region.
- Arrow keys: move card focus.
- `Enter`: open item.
- `s`: start focused spendable item.
- `r`: resolve focused blocker.
- `d`: add discretionary item.
- `v`: cycle Board/List/Graph.

## 8.9 Acceptance criteria

- Board placement derives from case events and guards.
- Card movement cannot mutate state.
- Every blocked item names one primary blocker and exposes all blockers in detail.
- Spendable Now does not imply already authorized.
- Active agent and command work are visually distinguishable.
- Empty states recommend only source-supported recovery paths.
- The board updates from durable event cursor continuity.

---

# 9. WF-P22 — Run Detail

**Route:** `/cells/:cellId/cases/:caseId/runs/:runId`  
**Job:** Let the operator observe and control non-agent execution without mistaking process activity for accepted outcome.  
**Primary decision:** Monitor, cancel, inspect evidence, or recover after terminal state.  
**Primary output:** Execution evidence and settlement handoff.

## 9.1 Desktop wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ RUN-07 · Execute repair simulation                                           │
│ Execution: SUCCEEDED        Settlement: EVALUATING             [Cancel — disabled: run terminal] │
│ Plan item PI-12 · Jail sandbox · env repair-python@2.0 · 1m 42s              │
├──────────────────────────────────────────────────────────────────────────────┤
│ Lifecycle                                                                    │
│ Authority ✓ → Queued ✓ → Environment ✓ → Running ✓ → Evidence ✓ → Eval ●   │
├───────────────────────┬──────────────────────────────────────────────────────┤
│ Governed boundaries   │ Live / captured output                              │
│ File: workspace/**    │ [stdout] [stderr] [events]                           │
│ Network: deny         │ 14:42:12 test_repair_path ... ok                     │
│ Timeout: 10m          │ 14:42:13 test_recovery ... FAILED                    │
│ Evaluator: tests      │                                                      │
│ Criteria: 3           │ Output is execution evidence, not settlement.        │
├───────────────────────┴──────────────────────────────────────────────────────┤
│ Produced evidence                                                            │
│ ✓ stdout.txt verified   ✓ junit.xml verified   ✓ artifact repair-model.json │
├──────────────────────────────────────────────────────────────────────────────┤
│ Settlement preview                                                           │
│ 2 criteria passed · 1 failed · independent declaration pending               │
│                                                     [Open settlement detail] │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 9.2 Fixed dual-state header

Execution and settlement must remain visible while scrolling.

Examples:

```text
Execution: running        Settlement: unsettled
Execution: succeeded      Settlement: evaluating
Execution: succeeded      Settlement: rejected
Execution: failed         Settlement: rejected
Execution: cancelled      Settlement: rejected
```

Never show one generic “Run status.”

## 9.3 Regions

### R1 — Run identity and control
- run ID;
- plan item;
- episode/retry lineage;
- cancel;
- retry only after terminal settlement.

### R2 — Lifecycle tracker
Uses committed source events.

### R3 — Governed boundaries
Exact limits:

- sandbox;
- environment;
- filesystem;
- network;
- timeout;
- commands/tools;
- authority decision.

### R4 — Output viewer
Output is:

- bounded;
- virtualized;
- searchable;
- downloadable if authorized;
- clearly labeled as evidence, not truth.

### R5 — Evidence inventory
Each item:

- type;
- digest;
- verification;
- producer;
- criterion links;
- restricted status.

### R6 — Settlement preview
Previews current criterion results and links to P27.

## 9.4 Cancellation

The cancel dialog states:

```text
Cancel RUN-07?

This requests cancellation of this run only.
Sibling runs will continue.
Partial evidence will be preserved.
Settlement is expected to become rejected with basis: cancelled.

[Keep running] [Request cancellation]
```

The UI moves through:

```text
running → cancelling → cancelled
```

It does not immediately display cancelled after button press.

## 9.5 Retry

Retry is available only after terminal state and creates a new episode.

Dialog includes:

- prior run;
- unchanged/changed plan inputs;
- reason for retry;
- whether preflight/authority must be repeated;
- new run ID after commit.

## 9.6 Failure states

### Authority denied
No output console pretending execution began. Show the denied decision and no-side-effects evidence.

### Environment failed
Show preparation trace separately from command trace.

### Sandbox violation
Prominent boundary breach panel; settlement rejected; route to audit/integrity as appropriate.

### Server interruption
Show last confirmed event cursor and whether the episode is resumable.

### Output unavailable
Evidence inventory and state remain usable. Raw stream failure cannot erase run status.

## 9.7 Keyboard

- `c`: cancel when eligible.
- `o`: focus output.
- `1/2/3`: stdout/stderr/events.
- `f`: search output.
- `e`: evidence drawer.
- `s`: settlement detail.
- `r`: retry when eligible.

## 9.8 Acceptance criteria

- Dual state is always visible.
- Output cannot visually dominate settlement state.
- Cancel does not imply immediate terminal state.
- Retry creates a new run.
- Authority denial shows proof of no execution.
- Produced artifacts expose descriptors and hashes.
- Settlement detail is reachable in one action.
- Live reconnect uses durable cursor and stale labeling.

---

# 10. WF-P23 — Agent Task Detail

**Route:** `/cells/:cellId/cases/:caseId/agent-runs/:runId`  
**Job:** Let the operator supervise bounded agent labor while preserving authority, cost, permissions, transcript evidence, and independent settlement.  
**Primary decision:** Monitor, resolve permission, cancel, inspect proof, or recover.  
**Primary output:** Transcript/proof evidence and settlement handoff.

## 10.1 Desktop wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ AGENT-RUN-12 · Implement adapter                                             │
│ Dialogue: AWAITING PERMISSION Settlement: UNSETTLED               [Cancel]    │
│ ACP · Claude Code · SWE_SEED active · Sponsor Sam · Session ACP-443          │
├──────────────────────────────────────────────────────────────────────────────┤
│ Budget and boundary                                                         │
│ Turns 4/12   Tokens 31%   Time 6m/20m   Sandbox jail   Network: none        │
├───────────────────────┬──────────────────────────────────────────────────────┤
│ Instruction contract  │ Dialogue activity                                   │
│ Task digest a17…      │ Agent: inspecting adapter trait…                     │
│ Expected artifacts 3  │ Tool request: write crates/agent/src/acp.rs          │
│ Criteria 5            │ Permission: PENDING                                  │
│ Tools 4 allowed       │ [Open permission request]                            │
│ Context refs 8        │                                                      │
├───────────────────────┴──────────────────────────────────────────────────────┤
│ Harness proof                                                                │
│ ✓ SWE_SEED route   ● ProofStarted   ○ ProofCompleted                         │
├──────────────────────────────────────────────────────────────────────────────┤
│ Transcript evidence                                                         │
│ Summary available · canonical transcript sealed · digest 6d3…               │
│ [Inspect summary] [Request transcript access]                                │
├──────────────────────────────────────────────────────────────────────────────┤
│ Agent claims “implementation complete” — no settlement standing              │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 10.2 Critical distinction

The screen uses three separate concepts:

1. **Dialogue state**
   - connecting;
   - streaming;
   - awaiting permission;
   - disconnected;
   - terminated.

2. **Termination reason**
   - completed turn;
   - endpoint error;
   - turn cap;
   - cancelled;
   - disconnect.

3. **Settlement state**
   - unsettled;
   - evaluating;
   - accepted;
   - rejected;
   - escalated.

No single “Agent status” field is permitted.

## 10.3 Regions

### R1 — Agent identity
- endpoint;
- provider/model;
- ACP/HTTP;
- sponsor;
- continuation/session identity;
- configuration digest;
- SWE_SEED harness state.

### R2 — Budget and boundary
- turns;
- tokens;
- elapsed/timeout;
- sandbox;
- network;
- tools;
- transcript retention.

Binding constraint should be emphasized:

```text
Binding limit: 2 turns remaining
```

### R3 — Instruction contract
- instruction digest;
- context refs;
- expected artifacts;
- criteria;
- allowed tools.

Full prompt/instruction text is disclosure-controlled.

### R4 — Dialogue activity
The transcript is not a chat product surface.

Display:

- structured turns;
- tool calls;
- permission requests;
- returned artifacts;
- errors;
- summaries.

Avoid speech bubbles that imply social conversation is the operational truth.

### R5 — Permission request
Pending permission appears as a first-class interrupt:

```text
Agent requests:
write crates/agent/src/acp.rs

Requested boundary:
workspace path only
no network
one operation

[Open approval]
```

### R6 — Harness proof
SWE_SEED proof artifacts receive primary proof placement when applicable.

### R7 — Transcript evidence
Shows:

- retention mode;
- transcript digest;
- summary version;
- access policy;
- verification state.

### R8 — Settlement warning
When the agent asserts success:

```text
Agent completion claim recorded as untrusted output.
Criteria evaluation has not completed.
```

## 10.4 Permission behavior

- Permission request pauses only the relevant action/session behavior defined by ACP.
- Approval opens P20 in context.
- Denial does not automatically cancel the delegation.
- The run view shows the permission result and subsequent agent behavior.
- No permission can exceed the run's existing grant boundary.

## 10.5 Cancellation

Same scoped-cancellation contract as P22, plus:

- external session termination behavior;
- continuation identity retained for audit;
- transcript-so-far preserved;
- sibling runs unaffected.

## 10.6 States

### Endpoint unavailable before connection
Show no dialogue region; show exact endpoint failure and no fallback.

### Disconnect
Show reconnect/continuation eligibility from source policy. Never silently start a fresh session under the same identity.

### Turn cap reached
Termination reason `turn_cap_exceeded`; preserve output; settlement evaluates independently.

### Transcript restricted
Summary and digest remain visible if allowed; full content never fetched before access decision.

### Agent output contains sensitive data
Redaction/quarantine policy result is shown as evidence handling state, not hidden silently.

## 10.7 Keyboard

- `p`: pending permission.
- `b`: budget/boundary.
- `t`: transcript summary.
- `h`: harness proof.
- `c`: cancel.
- `s`: settlement.
- `j/k`: previous/next structured turn.

## 10.8 Acceptance criteria

- Dialogue, termination, and settlement are distinct.
- Agent narration cannot render as accepted outcome.
- Binding budget is visible and updates from confirmed usage.
- Permission routes through the common approval surface.
- Denying permission does not imply cancellation.
- Transcript retrieval is disclosure-gated before access.
- No provider fallback occurs silently.
- SWE_SEED proof artifacts cross-link to the run and settlement.

---

# 11. WF-P27 — Settlement Detail

**Route:** `/cells/:cellId/settlements/:settlementId`  
**Job:** Help the user determine what happened, whether it counts, and why.  
**Primary decision:** Accept the recorded consequence, provide an eligible declaration/review, or initiate recovery from rejection/escalation.  
**Primary output:** Inspectable terminal or pending settlement and downstream case/capability effects.

## 11.1 Desktop wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Settlement SET-22 · Run RUN-07                          REJECTED              │
│ The command succeeded, but one committed criterion failed.                   │
│ Basis: repair recovery test failed                                            │
├──────────────────────────────────────────────────────────────────────────────┤
│ Outcome summary                                                              │
│ Expected: Valid repair adapter passes functional and recovery verification   │
│ Observed: Functional tests pass; recovery test fails after restart           │
│ Reliability: 0.91 · Independent declaration present · Debt check passed      │
├──────────────────────────────────────────────────────────────────────────────┤
│ Criteria matrix                                                              │
│ Criterion                    Expected       Evidence        Result   Weight   │
│ Functional tests             all pass       junit.xml       PASS     qual.    │
│ Recovery after restart       all pass       recovery.log    FAIL     qual.    │
│ Artifact identity            hash matches   artifact desc.  PASS     local    │
├──────────────────────────────┬───────────────────────────────────────────────┤
│ Declarations                 │ Consequence                                   │
│ Local evaluator: rejected    │ Plan item: rejected                           │
│ SWE_SEED: evidence supplied  │ Sentry: Design remediation activated          │
│ Reviewer A: standing yes     │ Capability: no promotion; recovery regression │
├──────────────────────────────┴───────────────────────────────────────────────┤
│ Next lawful paths                                                           │
│ [Open activated remediation] [Retry as new episode] [Inspect evidence]       │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 11.2 Content order

1. Settlement state and plain explanation.
2. Expected versus observed outcome.
3. Reliability and standing.
4. Criterion-by-criterion matrix.
5. Declarations and evidence provenance.
6. Consequences for case and capability.
7. Next lawful paths.
8. Raw record and policy snapshot in audit disclosure.

This ordering prevents users from starting with raw hashes before understanding consequence.

## 11.3 Criterion matrix behavior

Columns:

- criterion;
- origin;
- expected;
- evaluator;
- evidence;
- result;
- reliability;
- declaration standing;
- settlement contribution.

Rows can expand to show:

- immutable criterion hash;
- evidence verification;
- missing or excluded inputs;
- policy rules;
- timestamps;
- actor.

Result vocabulary:

```text
pass
fail
unknown
not_evaluated
excluded
quarantined
```

Unknown is not pass.

## 11.4 Pending settlement

Header:

```text
Execution succeeded
Settlement pending

Waiting for:
• Independent declaration from reviewer role
• Sealed transcript verification
```

The screen should show exactly what can complete the settlement and who has standing.

## 11.5 Rejected settlement

Primary next action should be the **activated recovery affordance** when one exists.

Examples:

- open remediation plan item;
- revise plan;
- retry as new episode;
- resolve missing evidence;
- escalate ambiguous settlement.

Do not use a generic “Try again.”

## 11.6 Accepted settlement

Show:

- what was accepted;
- limitations;
- variation class;
- capability contribution;
- activated sentries/milestones.

Avoid celebratory language that overstates capability:

```text
Accepted settlement
Contributes one qualifying observation to repair-adapter capability.
```

Not:

```text
Capability mastered!
```

## 11.7 Settlement declaration controls

Only visible when:

- settlement is pending;
- actor has standing;
- SoD permits;
- request is current;
- required evidence is visible.

The decision panel shows:

- exact criterion/evidence;
- declaration scope;
- reliability basis;
- effect of declaration.

## 11.8 Keyboard

- `j/k`: criterion rows.
- `Enter`: expand row.
- `e`: evidence.
- `d`: declarations.
- `n`: next lawful path.
- `c`: capability effect.
- `r`: recovery/retry menu.

## 11.9 Acceptance criteria

- Execution result and settlement basis are both visible.
- Each criterion maps to evidence or explicit absence.
- Unknown/missing evidence cannot appear as pass.
- Declaration standing and independence are visible.
- Case and capability consequences are source-linked.
- Rejected outcomes expose diagnostic recovery.
- Accepted outcomes state limitations and contribution level.
- Raw record export remains available without becoming the primary explanation.

---

# 12. WF-P32 — Capability Detail

**Route:** `/cells/:cellId/capabilities/:capabilityId`  
**Job:** Help the user understand what the installation has actually demonstrated, under what variation, and what remains unproven.  
**Primary decision:** Use the capability as routing evidence, inspect source settlements, or choose the next variation/recovery test.  
**Primary output:** Grounded capability assessment and visible next developmental affordance.

## 12.1 Desktop wireframe

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Capability: repair-adapter implementation                    DEMONSTRATED     │
│ Demonstrated in 3 cases · Not yet proven under endpoint failure variation    │
├──────────────────────────────────────────────────────────────────────────────┤
│ Capability ladder                                                           │
│ Attempted ✓ → Demonstrated ✓ → Proven ○ → Metabolized / external layer ○    │
├───────────────────────┬───────────────────────┬──────────────────────────────┤
│ Evidence strength     │ Variation coverage    │ Recovery and burden          │
│ 5 accepted settlements│ Python 2 versions     │ Recovery: 1/2 passed         │
│ Reliability 0.88      │ 2 endpoint types      │ Avg human interventions 1.7  │
│ 1 regression          │ Missing: disconnect   │ Trend: decreasing            │
├───────────────────────┴───────────────────────┴──────────────────────────────┤
│ Promotion explanation                                                       │
│ Proven requires:                                                             │
│ ○ one accepted disconnect-recovery case                                      │
│ ○ reliability ≥ 0.90 across qualifying declarations                         │
│ ✓ three varied cases                                                         │
├──────────────────────────────────────────────────────────────────────────────┤
│ Source settlements                                                          │
│ SET-18 accepted · SET-20 accepted · SET-22 rejected recovery                 │
├──────────────────────────────────────────────────────────────────────────────┤
│ Next spendable proof path                                                    │
│ Run the existing disconnect-recovery template in staging.                    │
│ Payment: one case, endpoint test environment, independent reviewer           │
│ [Start governed case from this path] [Ask Thoth] [Inspect policy]            │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 12.2 Regions

### R1 — Capability identity and conservative status
Status ladder:

```text
attempted
demonstrated
proven
degraded/quarantined
```

Where “metabolized” belongs to the broader CognitiveOS layer, the GUI must not imply SEA Forge has proven it unless the implemented capability policy explicitly represents it.

### R2 — Evidence strength
- accepted/rejected settlements;
- declaration reliability;
- qualifying versus excluded observations;
- regressions;
- policy version.

### R3 — Variation coverage
A matrix or facets:

- context;
- environment;
- input;
- provider/endpoint;
- pressure;
- support level;
- recovery condition.

### R4 — Recovery and orchestration burden
- recovery attempts;
- successful re-entry;
- human interventions;
- manager iterations;
- retries;
- trend.

### R5 — Promotion explanation
Shows:

- satisfied thresholds;
- missing thresholds;
- exclusions;
- exact policy snapshot.

### R6 — Source settlements
Every claim links to P27.

### R7 — Next spendable proof path
This region is generated only when supported by:

- missing promotion conditions;
- available template/environment;
- authority and settlement access;
- current readiness.

It must separate:

```text
Possible next test
from
Currently spendable proof path
```

## 12.3 Routing use

When capability is used in P12/P13, show the exact effect:

```text
Routing result:
Allowed for staging repair adapter work.
Escalation required for production deployment.
```

Capability never confers authority by itself.

## 12.4 Degraded or quarantined capability

Prominent explanation:

- regression;
- integrity issue;
- policy change;
- missing proof;
- environment drift.

Prior demonstrated state remains historical; current usable state is reduced.

## 12.5 Keyboard

- `l`: ladder.
- `v`: variation.
- `r`: recovery.
- `p`: promotion explanation.
- `s`: source settlements.
- `n`: next proof path.
- `Enter`: start case from eligible path.

## 12.6 Acceptance criteria

- Capability status is no higher than source records and policy allow.
- Variation and recovery are visible, not hidden behind total success count.
- Regressions reduce or qualify current state.
- Promotion requirements are exact and versioned.
- The next proof path is labeled spendable only when current conditions support it.
- Starting a proof case routes through P11/P12/P13 and confers no privilege.
- Thoth and routing use the same underlying capability records.

---

# 13. Cross-screen overlays on the primary path

## 13.1 Why This State drawer

Used on P2, P15, P16, P22, P23, P27, and P32.

Structure:

```text
State
Source records
Governing rule or criterion
Satisfied/failed condition
Affected actions
Next lawful path
```

## 13.2 Evidence drawer

Used on every screen after P13.

Minimum item row:

```text
Type | Producer | Digest | Verification | Criterion/claim | Open
```

## 13.3 Approval tray

Persistent when approvals are pending.

It shows:

- count;
- nearest expiry;
- case;
- requested action;
- whether current work is blocked.

It does not permit blind quick approval from the tray. It opens P20.

## 13.4 Active work tray

Shows:

- active runs;
- state;
- elapsed time;
- binding limit;
- scoped cancel affordance.

## 13.5 Recovery menu

Available only after a typed failure or rejected settlement.

Options are generated from actual state:

- retry as new episode;
- open remediation;
- revise plan;
- resolve approval;
- repair dependency;
- rebuild derived state;
- escalate ambiguity.

No generic catch-all retry.

---

# 14. Loading, stale, empty, and failure language

## 14.1 Loading language

Prefer observable work:

```text
Verifying 4 ledger streams…
Resolving 12 semantic references…
Waiting for concurrency slot 2/4…
Evaluating criterion 3 of 5…
```

Avoid:

```text
Please wait…
Working…
Processing…
```

## 14.2 Stale language

A stale view always shows:

- last verified state;
- invalidation event;
- affected claims/actions;
- rebuild/refresh route.

## 14.3 Empty language

An empty state explains which of these applies:

- nothing has been created;
- nothing is visible under current scope;
- no item is currently spendable;
- all work has settled;
- derived index is unavailable;
- disclosure policy omitted results.

## 14.4 Failure language

Format:

```text
What failed
Where it failed
Whether side effects occurred
What evidence remains
What can happen next
```

Example:

```text
Agent endpoint connection failed before dialogue began.

No task content was sent.
No provider fallback was attempted.
The endpoint configuration and connection error were recorded.

Next:
• Probe the endpoint
• Select another endpoint and replan
• Park the case
```

---

# 15. Accessibility and cognitive ergonomics

## 15.1 Non-color distinctions

Every status uses:

- label;
- icon;
- placement;
- optional pattern/border;
- accessible description.

## 15.2 Focus order

Focus follows:

```text
page identity
→ current state
→ primary action
→ attention/blockers
→ main content
→ evidence/history
```

## 15.3 Screen reader announcements

Live pages announce only meaningful transitions:

- authority result;
- run started;
- approval required;
- cancellation confirmed;
- execution terminal;
- settlement terminal;
- integrity state changed.

Do not announce every log line.

## 15.4 Reduced motion

- state transitions use minimal fades;
- no animated graph movement required to understand sentry activation;
- live counters update without layout shift.

## 15.5 Cognitive load

- one dominant action;
- progressive disclosure;
- consistent status vocabulary;
- exact distinction between execution and settlement;
- blockers grouped by causal layer;
- raw records secondary but always reachable.

---

# 16. Telemetry and UX settlement measures

Interface telemetry is not product truth, but it can test whether the GUI reduces representational error.

Recommended measures:

| Measure | Desired signal |
|---|---|
| Preflight correction rate | Detect issues before commit rather than during execution |
| False-completion interaction | Users do not treat execution success as settlement |
| Blocker resolution time | “Why blocked” leads to correct repair route |
| Evidence reachability | Terminal states reach evidence within one action |
| Approval decision quality | Fewer decisions made without opening context |
| Retry correctness | Retries create new episodes, not accidental duplicate clicks |
| Stale-state recognition | Users respond to invalidation before protected action |
| Capability interpretation | Users distinguish demonstrated from proven |
| Draft abandonment location | Reveals excessive payment in P11/P12 |
| Keyboard completion | Operational users can traverse without pointer dependency |

Do not optimize for raw clicks or time-on-screen where slower review is the intended governance behavior.

---

# 17. Prototype sequence

The first interactive prototype should cover one complete happy path and four failure variations.

## 17.1 Happy path

```text
P2 Ready
→ P11 ADLC case
→ P12 complete configuration
→ P13 commit
→ P15 case overview
→ P16 start command run
→ P22 execution succeeds
→ P27 settlement accepted
→ P32 capability receives qualifying observation
```

## 17.2 Failure variation A — readiness blocker

```text
P2 ledger integrity invalid
→ P29 inspect
→ P39 repair/verify
→ return P2
```

## 17.3 Failure variation B — approval escalation

```text
P13 external API escalated
→ P20 approve
→ return P13/commit
```

## 17.4 Failure variation C — agent permission and rejection

```text
P16 start agent task
→ P23 permission request
→ P20 deny permission
→ P23 continues
→ agent terminates
→ P27 settlement rejected
→ P16 remediation activated
```

## 17.5 Failure variation D — command succeeds but settlement fails

```text
P22 execution succeeded
→ P27 criterion failed
→ P16 prior design/remediation reactivated
```

These variants test the product thesis better than a happy-path-only prototype.

---

# 18. Wireframe settlement criteria

This screen specification is complete enough to move into visual design and interactive prototyping when:

1. every primary-path screen has one dominant user decision;
2. all screen regions map to defined atomic organisms and molecules;
3. execution, dialogue termination, and settlement remain distinct;
4. every blocker exposes evidence and the next lawful path;
5. draft, proposal, committed, and settled states are visually distinct;
6. corrective detours preserve origin and invalidate stale preflight when needed;
7. empty/loading/stale/error states are specified;
8. keyboard and screen-reader behavior are defined for operational transitions;
9. responsive behavior preserves status and evidence rather than merely shrinking layouts;
10. the prototype variations prove denial, escalation, cancellation, rejection, recovery, and capability restraint.

The next implementation-facing artifact should project this specification into a **frontend application architecture and component contract**, including:

```text
route modules
page state machines
query/command boundaries
event subscriptions
typed view models
component props/events
permission-aware data loading
test fixtures
storybook states
end-to-end test scenarios
```
