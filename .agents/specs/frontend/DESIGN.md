# SEA Forge Workbench Design System

> Category: Developer Tools  
> A dark, high-density workbench for governed agentic operations. Every surface makes state, authority, evidence, and the next lawful action visible. Meaning first; decoration never.

## Product Context

SEA Forge is a governed cognition and execution environment for operators, sponsors, reviewers, and automated actors. Its interface must preserve the distinctions between navigation and mutation, draft and committed work, approval and execution, execution and settlement, agent termination and settlement, and settlement and capability.

The canonical journey is:

```text
readiness → identity and authority → domain meaning → case draft → preflight
→ committed case → horizon → governed execution → evidence → settlement
→ capability and artifact maturity
```

This package is grounded in the copied SEA Forge UX epic, atomic breakdown, view-flow specification, primary-path wireframes, frontend architecture contract, mockup brief, and semantic mapping. Source provenance is recorded in `context-provenance.md`.

## Visual Theme & Atmosphere

SEA Forge is a **mission-control workbench for governed cognition**, not a generic project dashboard and not a chat application.

The interface should feel:

- rigorous;
- calm under pressure;
- dense but legible;
- keyboard-first;
- source-grounded;
- operational rather than promotional;
- cybernetic rather than dopamine-driven.

Reference qualities:

- **Linear:** density, keyboard focus, restrained chrome;
- **Raycast:** recognition over recall and command-driven navigation;
- **Cursor:** peripheral awareness of active agent work;
- **Bloomberg Terminal:** information density without decorative card soup;
- **NASA and SpaceX consoles:** one primary operational focus with supporting context around it.

Avoid the visual language of Jira, Monday, Asana, ClickUp, consumer chat apps, and oversized analytics dashboards.

### Governing visual law

The center of a screen is the **current governed focus**, which varies by view:

- readiness condition;
- draft or plan contract;
- current case horizon;
- approval or human judgment;
- active execution;
- agent delegation;
- pending or terminal settlement;
- capability assessment;
- integrity or maintenance condition.

A settlement panel becomes central only when settlement is the current object of inspection or judgment.

Every screen must answer:

1. Where am I?
2. What state is this in?
3. Why is that state true?
4. What evidence supports it?
5. What can happen next?
6. What authority, effort, or coordination is required?
7. What result would count as settlement?

### Density

The native density is compact and operational.

Use:

- compact tables and rows;
- progressive disclosure;
- fixed headers for critical state;
- drawers for evidence and provenance;
- virtualized logs and event streams;
- restrained whitespace;
- no giant cards;
- no marketing-style hero regions inside the product.

Exactly one action or condition receives dominant visual emphasis per screen.

### Product surfaces

The design system must support:

- Readiness;
- Thoth;
- Assets;
- Domain Models;
- Cases;
- Inbox and Approvals;
- Operations;
- Evidence;
- Memory;
- Capabilities;
- Specification Pipelines;
- Artifacts and IP;
- Federation;
- Integrity;
- Administration and Maintenance.

Within a case, support:

- Overview;
- Horizon;
- Timeline;
- Plan;
- Command Runs;
- Agent Tasks;
- Settlement;
- Evidence.

## Color Palette & Roles

Tokens are named by **cognitive and operational function**, never by appearance.

`red_500`, `green_success`, and similar generic names are prohibited. Use names such as `authority_denied`, `settlement_accepted`, and `evidence_quarantined`.

Color is a redundant semantic signal, never the sole carrier. Every state must also use:

- a text label;
- an icon or shape;
- stable placement;
- an accessible description.

### Surface palette

| Token | Hex | Role |
|---|---:|---|
| `--surface-workspace` | `#0B1220` | Main application canvas |
| `--surface-panel` | `#111827` | Standard panels, rails, tables |
| `--surface-panel-elevated` | `#1F2937` | Current governed focus |
| `--surface-muted` | `#334155` | Dividers, collapsed or background-weight content |
| `--surface-overlay` | `#0F172A` | Drawers, dialogs, command palette |
| `--surface-code` | `#08101D` | Logs, commands, machine records |

### Text palette

| Token | Hex | Role |
|---|---:|---|
| `--fg-primary` | `#E5EDF8` | Titles, principal values, current state |
| `--fg-secondary` | `#94A3B8` | Labels, explanations, evidence descriptors |
| `--fg-tertiary` | `#64748B` | Timestamps, supporting metadata. **Note:** `#64748B` on `--surface-panel` (`#0B1220`) does not meet WCAG AA for 12px text (contrast ≈ 3.9:1, AA requires 4.5:1 for normal text). Restrict this token to non-essential or large text (≥18px / ≥14px bold, where AA-large 3:1 applies). For essential small text on `--surface-panel`, use `--fg-secondary` (`#94A3B8`, ≈ 6.0:1). |
| `--fg-inverse` | `#07111F` | Text on bright semantic fills |

### Governance and authority

| Token | Hex | Role |
|---|---:|---|
| `--color-authority-allowed` | `#16A34A` | The requested action is allowed |
| `--color-authority-denied` | `#DC2626` | The requested action is denied |
| `--color-authority-escalated` | `#F59E0B` | Human or higher authority required |
| `--color-authority-degraded` | `#D97706` | Allowed with explicit limitation |
| `--color-authority-pending` | `#64748B` | Decision not yet made |

### Execution and dialogue

| Token | Hex | Role |
|---|---:|---|
| `--color-execution-running` | `#2563EB` | Command or operation in progress |
| `--color-execution-waiting` | `#64748B` | Waiting for capacity, dependency, or authority |
| `--color-execution-succeeded` | `#16A34A` | Execution process completed successfully |
| `--color-execution-failed` | `#DC2626` | Execution process failed |
| `--color-execution-cancelled` | `#64748B` | Scoped cancellation completed |
| `--color-dialogue-streaming` | `#2563EB` | Agent dialogue actively streaming |
| `--color-dialogue-permission` | `#F59E0B` | Agent permission request pending |
| `--color-dialogue-terminated` | `#64748B` | Agent session terminated |

Execution success never visually implies settlement acceptance.

### Settlement and evidence

| Token | Hex | Role |
|---|---:|---|
| `--color-settlement-pending` | `#64748B` | Not yet evaluated or awaiting evidence |
| `--color-settlement-evaluating` | `#2563EB` | Criteria are being evaluated |
| `--color-settlement-accepted` | `#16A34A` | Committed criteria accepted |
| `--color-settlement-rejected` | `#DC2626` | One or more required criteria failed |
| `--color-settlement-escalated` | `#F59E0B` | Judgment or additional evidence required |
| `--color-settlement-quarantined` | `#7C3AED` | Settlement basis is untrusted or isolated |
| `--color-evidence-complete` | `#16A34A` | Required evidence present and valid |
| `--color-evidence-missing` | `#F59E0B` | Required evidence absent |
| `--color-evidence-failed` | `#DC2626` | Verification or proof failed |
| `--color-evidence-stale` | `#D97706` | Evidence or projection invalidated |
| `--color-evidence-quarantined` | `#7C3AED` | Evidence isolated or untrusted |

### Readiness, integrity, and freshness

| Token | Hex | Role |
|---|---:|---|
| `--color-readiness-ready` | `#16A34A` | Required conditions pass |
| `--color-readiness-degraded` | `#F59E0B` | Usable with named limitations |
| `--color-readiness-blocked` | `#DC2626` | Required condition prevents action |
| `--color-freshness-current` | `#16A34A` | View reflects current source records |
| `--color-freshness-stale` | `#F59E0B` | View invalidated; prior verified result remains |
| `--color-integrity-verified` | `#16A34A` | Required integrity verified |
| `--color-integrity-partial` | `#64748B` | Partial or local assurance; exact level required |
| `--color-integrity-pending` | `#F59E0B` | Verification incomplete |
| `--color-integrity-failed` | `#DC2626` | Integrity failure affects work |

### Capability

| Token | Hex | Role |
|---|---:|---|
| `--color-capability-attempted` | `#64748B` | Attempts or observations only |
| `--color-capability-demonstrated` | `#2563EB` | Qualifying demonstrated evidence |
| `--color-capability-proven` | `#16A34A` | Promotion policy thresholds met |
| `--color-capability-degraded` | `#F59E0B` | Regression or reduced current applicability |
| `--color-capability-quarantined` | `#7C3AED` | Capability claim cannot currently be trusted |

### Selection and focus

| Token | Hex | Role |
|---|---:|---|
| `--color-focus-primary` | `#2563EB` | Current governed focus, selected path |
| `--color-focus-ring` | `#60A5FA` | Keyboard focus outline |
| `--color-attention` | `#F59E0B` | Time-sensitive or unresolved attention |
| `--color-danger` | `#DC2626` | Destructive or integrity-sensitive action |

### Dark mode

Dark is the native and only mode for v1. Do not synthesize a light theme by simply inverting values.

```css
:root {
  --surface-workspace: #0B1220;
  --surface-panel: #111827;
  --surface-panel-elevated: #1F2937;
  --surface-muted: #334155;
  --surface-overlay: #0F172A;
  --surface-code: #08101D;

  --fg-primary: #E5EDF8;
  --fg-secondary: #94A3B8;
  --fg-tertiary: #64748B;
  --fg-inverse: #07111F;

  --color-focus-primary: #2563EB;
  --color-focus-ring: #60A5FA;
  --color-authority-allowed: #16A34A;
  --color-authority-denied: #DC2626;
  --color-authority-escalated: #F59E0B;
  --color-evidence-quarantined: #7C3AED;
}
```

Do not invent additional pigments. New semantic states should alias the existing palette unless a genuinely new cognitive distinction requires a new token.

## Typography Rules

### Font stacks

```css
--font-sans: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
--font-mono: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
```

Use sans-serif for:

- navigation;
- titles;
- labels;
- explanations;
- decisions;
- status text.

Use monospace for:

- commands;
- record IDs;
- hashes;
- durations and budgets where alignment matters;
- logs;
- traces;
- machine-readable values.

### Semantic type scale

| Role | Size | Weight | Line height | Usage |
|---|---:|---:|---:|---|
| Application title | 20px | 600 | 1.2 | Cell or major workspace identity |
| Page title | 18px | 600 | 1.25 | Current route |
| Governed focus title | 16px | 600 | 1.3 | Readiness, plan, run, settlement, capability |
| Section heading | 14px | 600 | 1.4 | Panel grouping |
| Body | 14px | 400 | 1.5 | Explanations and ordinary content |
| Label | 12px | 500 | 1.4 | Field names and state dimensions |
| Metadata | 12px | 400 | 1.4 | Timestamps, source, versions |
| Machine value | 12–14px | 400 | 1.5 | IDs, hashes, logs |

Use tabular numerals for:

- budgets;
- TTL countdowns;
- durations;
- event ordinals;
- criterion counts;
- reliability values.

Avoid oversized headings. Product hierarchy comes from placement, state, and contrast—not marketing typography.

## Component Stylings

### Global component law

Every substantial component must expose or be able to reveal:

- typed identity;
- current state;
- why the state is true;
- source and freshness;
- evidence or provenance;
- next lawful action;
- disabled-action reason.

Presentational components do not create evidence. Governed actions, decisions, executions, verifications, and settlements create records and evidence.

### Application Shell

Desktop composition:

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ Cell | Actor/Role | Policy | Integrity | Search | Inbox | Active Work       │
├──────────────┬───────────────────────────────────────────┬───────────────────┤
│ Primary nav  │ Main governed focus                      │ Context / Evidence│
│              │                                           │ drawer            │
├──────────────┴───────────────────────────────────────────┴───────────────────┤
│ Optional connection, cursor, and stale-state bar                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

The global bar keeps cell, actor, policy, integrity, approval, and live-work context visible.

### Governed Focus Header

The primary header for any operational page.

Must display:

- page title;
- typed state;
- one-sentence explanation;
- source/freshness indicator;
- one dominant lawful action;
- secondary inspection actions.

```css
.governed-focus {
  background: var(--surface-panel-elevated);
  border: 1px solid var(--surface-muted);
  border-left: 3px solid var(--color-focus-primary);
  border-radius: 4px;
  padding: 16px;
}
```

The left border color follows the current domain state, but the text label remains mandatory.

### Governed Status Pill

Use compact, rectangular pills with:

- icon;
- uppercase or concise label;
- semantic border;
- accessible text.

```css
.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 22px;
  padding: 2px 8px;
  border: 1px solid currentColor;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: .04em;
  text-transform: uppercase;
}
```

Do not render a generic `SUCCESS` badge. Always label the state domain:

- `EXECUTION SUCCEEDED`;
- `SETTLEMENT ACCEPTED`;
- `AUTHORITY ALLOWED`;
- `INTEGRITY VERIFIED`.

### Dual State Indicator

Command runs always display execution and settlement separately.

```text
Execution: SUCCEEDED
Settlement: EVALUATING
```

Agent tasks display three separate dimensions:

```text
Dialogue: TERMINATED
Termination: TURN CAP
Settlement: REJECTED
```

Use a fixed header or paired row so these distinctions remain visible while scrolling.

### Source Freshness Badge

Displays:

- current, stale, or unknown;
- projection or source type;
- last verified time;
- invalidation cause on expansion.

Stale views retain the prior verified state but never present it as current.

### Protected Action Button

Only one protected action may be visually dominant per screen.

States:

- enabled;
- disabled with reason;
- submitting;
- accepted for processing;
- committed;
- denied;
- escalated;
- submission unknown.

Button press is not success. Authoritative state appears only after the governed result is returned.

### Why State Panel

Structure:

```text
Current state
Reason
Governing condition or rule
Source records
Satisfied conditions
Failed conditions
Affected operations
Next lawful paths
```

This panel replaces generic error banners.

### Evidence Drawer

Tabs:

- Why;
- Evidence;
- Provenance;
- Record.

Evidence rows show:

- type;
- producer;
- digest;
- verification;
- criterion or claim;
- access action.

Raw logs and machine records are secondary but always reachable.

### Readiness Console

Show:

- operation-sensitive readiness;
- critical foundations;
- operational capabilities;
- current affordances;
- limitations;
- recent invalidations.

Hard-gate failure must remain visually dominant even when many optional checks pass.

### Case Creation Workbench

Use a three-region layout:

- configuration navigation;
- active editor;
- draft health and blockers.

Draft state must be labeled:

```text
Saved locally
Not committed to SEA Forge
```

### Plan Preflight Panel

Three burden groups:

- work shape;
- authority and coordination;
- settlement and evidence.

The screen must state:

```text
Nothing has executed.
```

The commit dialog lists what will be created and what will not happen yet.

### Case Horizon Board

Supported projections:

- Board;
- List;
- Graph.

These are alternate projections of the same case state, not separate truth models.

Regions:

- Spendable now;
- Active;
- Awaiting authority or approval;
- Blocked;
- Future;
- Settled.

Dragging may alter personal arrangement only. It cannot mutate authoritative state.

### Execution Console

Must include:

- dual execution/settlement state;
- authority boundary;
- sandbox class;
- environment;
- trace;
- bounded output;
- evidence inventory;
- settlement preview;
- scoped cancellation.

Output is labeled as execution evidence, not settlement truth.

### Agent Task Console

Do not render as a consumer chat application.

Show:

- task role or capability first;
- endpoint, provider, model, and protocol as metadata;
- instruction digest;
- allowed tools and boundaries;
- turns, tokens, time, and binding limit;
- structured dialogue turns;
- permission requests;
- transcript evidence;
- SWE_SEED proof;
- termination reason;
- settlement state.

### Approval Decision Panel

Shows:

- requesting actor and sponsor;
- exact action and resource;
- boundaries;
- policy reason;
- evidence;
- expiry;
- separation-of-duty warning;
- consequence.

Do not allow blind batch approval.

### Criterion Settlement Matrix

Rows:

- criterion;
- origin;
- expected result;
- evaluator;
- evidence;
- result;
- reliability;
- declaration standing;
- settlement contribution.

Valid result states:

- pass;
- fail;
- unknown;
- not evaluated;
- excluded;
- quarantined.

Unknown is never styled as pass.

### Capability Promotion Panel

Show:

- attempted, demonstrated, proven, degraded, or quarantined;
- qualifying settlements;
- variation;
- recovery;
- regressions;
- orchestration burden;
- satisfied promotion conditions;
- missing conditions;
- next possible test;
- next currently spendable proof path.

A proof path navigates through case creation and preflight. It does not directly promote capability.

### Decision Options

Show only the exact lawful actions supported by the current state.

Use `Recommended / Safe / Experimental` only when a trusted external routing or valuation feed supplies those distinctions.

Never fabricate an option to fill a three-choice layout.

Payment or cost displays may show:

- trusted external valuation;
- observed orchestration burden;
- time, token, coordination, or approval requirements.

Hide unsupported valuation rather than inventing it.

### CLI and machine equivalence

Where a real CLI operation exists, expose it through the Evidence or Record drawer rather than permanently occupying every panel.

Never invent a CLI command for visual symmetry.

### Motion

Motion is a governance signal, not decoration.

| Interaction | Duration | Behavior |
|---|---:|---|
| State change focus | 150–200ms | Brief border/background transition |
| New attention item | 150ms | One-time emphasis |
| Panel or drawer | 150–200ms | Opacity and position |
| Log or event append | 0ms | No animated layout movement |
| Settlement accepted | 0ms | Calm immediate state change |
| Background update | 0ms | No motion |
| Time-sensitive expiry | restrained | Motion only when genuinely urgent |

Do not use indefinitely looping attention pulses for ordinary warnings.

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

## Motion

Motion communicates a governed transition, never ambient activity. Use the shared `--motion-enter`, `--motion-exit`, and `--ease-governed` tokens. State focus, drawers, and newly arrived attention items may transition once; logs, background refreshes, and accepted settlements update immediately. Ordinary warnings never pulse. Reduced-motion preferences collapse all transitions to near-zero duration.

### Iconography

Use Lucide-style line icons:

- 1.5px stroke;
- 16px default;
- 14px compact;
- 20px page-level state.

No decorative icons. Every icon must communicate state, object kind, or action.

## Layout Principles

### Grid and spacing

Use a 4px baseline grid.

```css
--space-1: 4px;
--space-2: 8px;
--space-3: 12px;
--space-4: 16px;
--space-6: 24px;
--space-8: 32px;
--space-12: 48px;

--row-compact: 32px;
--row-standard: 40px;
--row-review: 48px;
--panel-padding: 16px;
--section-gap: 12px;
```

Rows become taller as judgment weight increases.

### Desktop layout

At ≥1440px:

- 248–280px primary navigation;
- flexible main content;
- 360–420px persistent context/evidence drawer.

At 1100–1439px:

- 224–248px navigation;
- evidence drawer overlays or opens on demand.

At 768–1099px:

- collapsed icon rail;
- evidence opens as full-height overlay;
- operational regions stack selectively.

### Navigation

Primary navigation:

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

Case navigation:

```text
Overview
Horizon
Timeline
Plan
Runs
Agent Tasks
Settlement
Evidence
```

Navigation labels are product concepts, not filesystem structure.

### Projection switches

Projection switches may change visibility, density, or arrangement. They may never create different operational truth.

Allowed examples:

- Board / List / Graph;
- stdout / stderr / events;
- Guided / Operational / Audit;
- Why / Evidence / Provenance / Record.

### Placement priorities

| Content | Placement |
|---|---|
| Current governed focus | Main center region |
| Dominant lawful action | Header or focus footer |
| Critical blocker | Adjacent to state and action |
| Approval or judgment | Persistent attention region |
| Evidence summary | Near consequence; full detail in drawer |
| Source/freshness | Header or record metadata |
| Agent progress | Operational but secondary to case consequence |
| Raw logs | Collapsed or dedicated console |
| Archived history | Timeline or disclosure |
| Destructive action | Overflow or explicit decision dialog |

### Forms

Use compact boxed or underlined inputs with clear labels.

For complex workbenches:

- sticky section navigation;
- visible validation taxonomy;
- exact field-level issue links;
- draft health rail;
- persistent Save Draft and Continue actions.

### Tables and matrices

Use:

- sticky headers;
- row focus;
- 32–40px rows;
- monospace for IDs and numeric alignment;
- column collapse by priority;
- drawer expansion for full detail.

Avoid card grids for evidence, approvals, runs, and criteria.

## Depth & Elevation

Use only three depth levels:

### Level 0 — Workspace

- no shadow;
- deep navy canvas;
- primary navigation and broad background.

### Level 1 — Standard panel

- `--surface-panel`;
- 1px muted border;
- 4px radius;
- no shadow.

### Level 2 — Governed focus, drawer, or dialog

- `--surface-panel-elevated` or `--surface-overlay`;
- semantic border;
- optional subtle shadow for separation only.

```css
.panel {
  background: var(--surface-panel);
  border: 1px solid var(--surface-muted);
  border-radius: 4px;
}

.panel--focus {
  background: var(--surface-panel-elevated);
  border-color: var(--color-focus-primary);
}

.overlay {
  background: var(--surface-overlay);
  box-shadow: 0 12px 36px rgba(0, 0, 0, .30);
}
```

Do not use:

- glassmorphism;
- neumorphism;
- glow as decoration;
- deep shadow stacks;
- floating cards without semantic reason;
- radius above 8px for operational panels.

## Do's and Don'ts

### Do

- Use semantic token names exclusively.
- Keep one dominant governed focus per screen.
- Display source, freshness, and integrity on inspectable operational views.
- Distinguish execution from settlement.
- Distinguish agent dialogue, termination, and settlement.
- Label waiting and parked states as normal holds, not failures.
- Show disabled-action reasons.
- Show the nearest lawful repair or recovery path.
- Preserve prior verified state when a view becomes stale.
- Use evidence and provenance drawers for depth.
- Use tables and compact rows for dense operational collections.
- Keep accepted states calm.
- Show provider, model, endpoint, and protocol as metadata when operationally relevant.
- Provide non-color state signals.
- Use exact user-facing language for the state domain.

### Don't

- Do not center every screen on settlement.
- Do not use giant cards or card soup.
- Do not organize the product around chats, files, or vendor agents.
- Do not turn the agent task view into a casual chat interface.
- Do not show a generic green success state.
- Do not merge execution success with settlement acceptance.
- Do not merge agent termination with settlement.
- Do not promote capability from a single unqualified success.
- Do not create an “Accept Settlement” button unless the screen represents an eligible, scoped declaration with standing.
- Do not invent Recommended, Safe, and Experimental choices.
- Do not invent payment or confidence scores.
- Do not treat drag-and-drop as authoritative case mutation.
- Do not broad-fetch restricted content and hide it client-side.
- Do not show raw logs as the primary explanation.
- Do not use color as the only signal.
- Do not animate ordinary background updates.
- Do not invent CLI commands, record types, or backend states.
- Do not hide blocked or unavailable paths without explaining what is missing.
- Do not imply that a draft has been committed.

## Voice

Write with calm operational precision. Name the state domain, explain why it is true, and identify the next lawful path. Prefer short declarative sentences:

- “Execution completed. Settlement evaluation continues.”
- “Commit blocked: policy bundle changed.”
- “No path is currently spendable. Approval APR-07 is pending.”
- “This timebox was missed. What should change?”

Avoid celebration, blame, vague confidence, and generic success language. Never use “Congratulations,” “Great job,” “Mission accomplished,” or “Something went wrong” when a specific governed reason is available.

## Anti-patterns

- Generic dashboards, card soup, marketing heroes, chat-first agent views, and ornamental data visualizations.
- Color-only status, generic `SUCCESS` badges, or any green treatment that collapses execution, settlement, integrity, and authority into one meaning.
- Hidden disabled paths, blind batch approval, unsupported recommendations, invented scores, invented CLI commands, or fabricated evidence.
- Oversized typography, radii above 8px, decorative glow, glassmorphism, gradients, deep shadow stacks, and animated background updates.
- Draft views that imply commitment, execution output presented as settlement truth, or agent termination presented as success.
- Small essential text using `--fg-tertiary`; use `--fg-secondary` unless the text qualifies as large.

## Responsive Behavior

### Desktop ≥1440px

- full application shell;
- persistent left navigation;
- persistent evidence/provenance drawer where useful;
- multi-column workbenches;
- full Case Horizon board;
- complete criterion matrix.

### Compact desktop 1100–1439px

- left navigation remains visible;
- evidence drawer opens as overlay;
- three-column content becomes two-column;
- secondary metadata collapses behind disclosure.

### Tablet 768–1099px

- icon navigation rail;
- one main content column with selected secondary rail;
- Case Horizon uses state tabs rather than compressed columns;
- tables preserve primary columns and move detail into drawers;
- approval, monitoring, cancellation, and settlement review remain fully usable.

### Phone <768px

The phone experience is operationally focused, not feature-complete.

Prioritize:

- readiness inspection;
- alerts and inbox;
- approval decisions;
- active-run monitoring;
- scoped cancellation;
- settlement review;
- integrity alerts;
- evidence summaries.

Complex `.sea` authoring, large topology configuration, graph editing, and wide criterion matrices are desktop-first. On phone, provide read-only inspection or a clear “Open on desktop” boundary rather than a broken miniature editor.

### Accessibility

- WCAG AA contrast minimum;
- visible `:focus-visible` rings;
- keyboard traversal through all primary-path actions;
- table semantics for matrices;
- list alternative for every graph;
- reduced-motion support;
- 200% zoom without state or action loss;
- live announcements only for meaningful governed transitions.

## Agent Prompt Guide

When generating SEA Forge mockups:

1. Treat `DESIGN.md` as the visual and interaction grammar.
2. Treat `DESIGN-spec-mapping.md` as semantic traceability.
3. Treat `MOCKUP-BRIEF.md` as the required screen and flow scope.
4. Do not infer backend implementation details from the mockup.
5. Use one shared application shell across all screens.
6. Keep cell, actor/role, policy, integrity, inbox, and active-work context visible.
7. Use exact status-domain labels:
   - `EXECUTION SUCCEEDED`;
   - `SETTLEMENT EVALUATING`;
   - `AUTHORITY ESCALATED`;
   - `CAPABILITY DEMONSTRATED`.
8. Never render one generic success state.
9. Use dense rows, panels, tables, and drawers—not oversized cards.
10. Keep the current governed focus in the center.
11. Put source, freshness, evidence, and provenance within one action.
12. Use one dominant action per screen.
13. Render unavailable actions with reasons and repair paths.
14. Show only lawful actions supplied in the mockup brief.
15. Do not invent pricing, confidence, authority, criteria, evidence, or capability states.
16. Do not use a chat-first visual for agent work.
17. Show provider/model/endpoint as metadata, not as persona branding.
18. Use calm, corrective copy:
    - “Execution completed. Settlement evaluation continues.”
    - “Commit blocked: policy bundle changed.”
    - “No path is currently spendable. Approval APR-07 is pending.”
19. Avoid celebratory copy such as “Congratulations,” “Great job,” or “Mission accomplished.”
20. Do not invent new colors, gradients, shadows, or rounded-card treatments.
21. Include loading, empty, stale, denied, escalated, failed, and recovery states where the brief requests them.
22. Preserve the distinction between:
    - navigation and mutation;
    - draft and committed plan;
    - approval and execution;
    - execution and settlement;
    - agent termination and settlement;
    - settlement and capability.
