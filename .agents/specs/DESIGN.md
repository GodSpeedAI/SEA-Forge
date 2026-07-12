# Settlement Design System

> Category: Developer Tools
> Governed-agent workbench. Dark semantic command center where every color, weight, and motion answers one question: what is preventing settlement? Meaning first, decoration never.

## 1. Visual Theme & Atmosphere

A **mission control center for governed cognition** — not a dashboard. The center of every screen is the current settlement; everything else orbits it peripherally. The aesthetic draws from Linear, Raycast, Cursor, and the Bloomberg terminal: high density, low noise, progressive disclosure. Never Jira, Monday, Asana, or ClickUp. The user should never see giant cards.

| Element | Hex | Role |
|---------|-----|------|
| Workspace | `#0B1220` | Deep navy canvas, primary depth |
| Panel | `#111827` | Elevated panels, rails |
| Panel Elevated | `#1F2937` | Focused/raised surfaces |
| Muted | `#334155` | Collapsed, background-weight content |
| Settlement Ready | `#2563EB` | The primary focus color — pending decisions |
| Authority Allowed | `#16A34A` | Permitted, accepted, complete states |
| Authority Denied | `#DC2626` | Blocked, rejected, failed, critical |
| Attention | `#F59E0B` | Evidence missing, escalation, warnings |
| Quarantine | `#7C3AED` | Quarantined evidence, untrusted data |
| Neutral State | `#64748B` | Pending, undetermined states |

*Every screen must answer: Where am I? What is preventing settlement? What is the cheapest next move? What evidence will prove it?*

### Use Cases

The Settlement Design System is purpose-built for:

- **The SEA Forge workbench (GodSpeed)** — Mission Control, Case Detail, Settlement Queue, Settlement Detail, Approvals & Human Tasks, Evidence, Policies, Capabilities, Traces, Domain Models, Spec Pipelines, Artifacts & IP, Templates & Environments, Memory, Ledger Integrity (view↔record mapping: `DESIGN-spec-mapping.md`)
- **Decision-support surfaces** — choice architecture with payment, risk, and confidence shown for every option
- **Governed-agent monitoring** — agent state as capability status (Research, Architecture, Implementation, Verification), never vendor personas
- **Any surface projected from the semantic state bus** — GUI, TUI, CLI themes, and docs share the same token source

### Prior Art

Linear (density and keyboard-first focus), Raycast (recognition over recall), Cursor (peripheral agent awareness), Bloomberg terminal (information density under pressure), NASA/SpaceX consoles (one primary focus, everything else peripheral). The governing frameworks are CMMN (long-lived cases), OODA (continuous observe–orient–decide–act navigation), and the CognitiveOS representational topology.

## 2. Color

The core rule: **tokens are named by cognitive function, never appearance.** `red_500` is banned; `authority_denied` is canonical. The chain is semantic state → visual meaning → design token → CSS. Source of truth is `design/godspeed-ui.tokens.toml`, compiled to `tokens.css`, `tailwind.theme.ts`, `cli-theme.json`, and docs — so "authority denied" means the same thing in GUI red, CLI red, notification rules, and evidence records.

### Surface Palette

| Token | Hex | Usage |
|-------|-----|-------|
| `--surface-workspace` | `#0B1220` | Page canvas |
| `--surface-panel` | `#111827` | Panels, rails, cards |
| `--surface-panel-elevated` | `#1F2937` | Current settlement, focused panels |
| `--surface-muted` | `#334155` | Collapsed logs, archived, background weight |

### Semantic State Palette

| Token | Hex | Usage |
|-------|-----|-------|
| `--color-authority-allowed` | `#16A34A` | Policy permits action |
| `--color-authority-denied` | `#DC2626` | Policy blocks action |
| `--color-authority-escalated` | `#F59E0B` | Requires higher authority |
| `--color-settlement-ready` | `#2563EB` | Settlement awaiting decision |
| `--color-settlement-accepted` | `#16A34A` | Settlement accepted |
| `--color-settlement-rejected` | `#DC2626` | Settlement rejected |
| `--color-settlement-pending` | `#64748B` | Settlement not yet ready |
| `--color-evidence-complete` | `#16A34A` | All required evidence present |
| `--color-evidence-missing` | `#F59E0B` | Required evidence absent |
| `--color-evidence-failed` | `#DC2626` | Proof command failed |
| `--color-evidence-quarantined` | `#7C3AED` | Evidence untrusted/isolated |
| `--color-risk-low` | `#16A34A` | Low-risk affordance |
| `--color-risk-medium` | `#F59E0B` | Medium-risk affordance |
| `--color-risk-high` | `#DC2626` | High-risk affordance |

### Extended Semantic States (SEA Forge records)

Aliases onto the same hues — new cognitive functions, no new pigments.

| Token | Hex | Usage |
|-------|-----|-------|
| `--color-approval-pending` | `#F59E0B` | ApprovalRequest awaiting a human; shows TTL countdown |
| `--color-approval-expired` | `#DC2626` | TTL expired → rejected |
| `--color-case-parked` | `#64748B` | `awaiting_approval` / no runnable item — a calm hold, never styled as failure |
| `--color-settlement-strong` | `#16A34A` | Independent qualifying declaration (`strength: strong`) |
| `--color-settlement-local` | `#64748B` | Kernel-local declaration only; never counts toward capability |
| `--color-capability-attempted` | `#64748B` | Raw observations only |
| `--color-capability-demonstrated` | `#2563EB` | One qualifying declaration |
| `--color-capability-proven` | `#16A34A` | Promotion policy thresholds met |
| `--color-capability-contracted` | `#F59E0B` | Status contracted; reason must be shown |
| `--color-assurance-verified` | `#16A34A` | `externally_verified` (witnessed) |
| `--color-assurance-partial` | `#64748B` | `checkpoint_signed` / `local_tamper_evident` / `legacy_digest_only` — label the exact level |
| `--color-assurance-pending` | `#F59E0B` | `integrity_pending` — never claim a verified history |
| `--color-assurance-failed` | `#DC2626` | `ledger_integrity_error` — halts affected work |
| `--color-artifact-stage` | `#94A3B8` | `cognitive → intellectual → product → capital` progress; capitalization requires human approval |

Every inspect surface MUST display the assurance level (spec-full §10.0a) — it is a spec MUST, not decoration. Sandbox class (`local | jail | microvm`) renders as metadata on every run row.

### Text Palette

| Token | Hex | Usage |
|-------|-----|-------|
| Primary | `#E5EDF8` | Settlement titles, primary content |
| Secondary | `#94A3B8` | Labels, evidence descriptors |
| Tertiary | `#64748B` | Timestamps, metadata, trace details |

### Dark Mode

Dark is the native and only mode for v1. The workbench is a long-session focus environment; the semantic state palette is calibrated against `#111827` panels. A light mode would require re-deriving the entire semantic color layer, not swapping values.

```css
:root {
  --surface-workspace: #0B1220;
  --surface-panel: #111827;
  --surface-panel-elevated: #1F2937;
  --surface-muted: #334155;
  --color-authority-allowed: #16A34A;
  --color-authority-denied: #DC2626;
  --color-authority-escalated: #F59E0B;
  --color-settlement-ready: #2563EB;
  --color-settlement-accepted: #16A34A;
  --color-settlement-rejected: #DC2626;
  --color-settlement-pending: #64748B;
  --color-evidence-complete: #16A34A;
  --color-evidence-missing: #F59E0B;
  --color-evidence-failed: #DC2626;
  --color-evidence-quarantined: #7C3AED;
  --color-risk-low: #16A34A;
  --color-risk-medium: #F59E0B;
  --color-risk-high: #DC2626;
  --fg-primary: #E5EDF8;
  --fg-secondary: #94A3B8;
  --fg-tertiary: #64748B;
}
```

CLI theme bindings mirror the same states: `authority_allowed=green`, `authority_denied=red`, `authority_escalated=yellow`, `evidence_missing=yellow`, `settlement_ready=blue`.

## 3. Typography

### Font Stack

```css
/* Monospace for commands, evidence refs, proof output, traces */
--font-mono: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;

/* Sans-serif for titles, labels, prose */
--font-sans: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
```

### Type Scale (named by semantic role, not size)

| Role | Size | Weight | Line Height | Font |
|------|------|--------|-------------|------|
| Mission Title | 20px | 600 | 1.2 | Inter, tracking-tight |
| Case Title | 18px | 600 | 1.25 | Inter |
| Settlement Title | 16px | 600 | 1.3 | Inter |
| Evidence Label | 14px | 500 | 1.4 | Inter |
| Body | 14px | 400 | 1.5 | Inter |
| Metadata | 12px | 400 | 1.4 | Inter, `--fg-tertiary` |
| Command | 14px | 400 | 1.5 | JetBrains Mono |

**Font labels for catalog extraction:**

```
Display: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif
Body: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif
Mono: "JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace
```

## 4. Spacing

4px baseline grid. Density is a semantic dimension: rows get taller as decisions get heavier.

```css
--space-1: 4px;   --space-2: 8px;   --space-3: 12px;  --space-4: 16px;
--space-6: 24px;  --space-8: 32px;  --space-12: 48px;

/* Density tokens — row height follows decision weight */
--row-compact: 32px;   /* lists, traces, logs */
--row-standard: 40px;  /* evidence items, agent rows */
--row-review: 48px;    /* settlement decisions — room to think */
--panel-padding: 16px;
--section-gap: 12px;
```

## 5. Layout & Composition

### The Mission Control Grid

The center panel is always the current settlement. Everything else orbits it. Exactly one primary focus per screen — enforced, not suggested.

```text
┌──────────────────────────────────────────────┐
│ Mission                                      │
├───────────────┬──────────────────────────────┤
│ Left rail     │ Center                       │
│ Mission       │ Current Situation            │
│ Cases         │ Current Settlement           │
│ Capabilities  │ Recommended Action           │
│ Policies      │ Payment · Authority          │
│               │ Evidence                     │
├───────────────┼──────────────────────────────┤
│ Right rail    │ Bottom                       │
│ Evidence      │ Timeline · Logs              │
│ Policy        │ CLI equivalent               │
│ Agent State   │ Proof commands               │
│ Trace         │ Notifications                │
└───────────────┴──────────────────────────────┘
```

Cognitive placement rules (encoded, not stylistic):

| Semantic content | Placement | Priority weight |
|---|---|---|
| Primary settlement | Center | 100 |
| Review required | Center-adjacent | 90 |
| Blocked items | Visible, flagged | 85 |
| Policy violation | Right rail, persistent | 90 |
| Evidence missing | Inline + notification | 90 |
| Agent progress | Peripheral | 30 |
| Raw logs | Collapsed by default | 30 |
| Archived | Hidden behind disclosure | 10 |

```css
.panel {
  background: var(--surface-panel);
  border: 1px solid var(--surface-muted);
  border-radius: 4px;
  padding: var(--panel-padding);
}

.panel--focus {
  background: var(--surface-panel-elevated);
  border-color: var(--color-settlement-ready);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--section-gap);
  padding-bottom: var(--space-2);
  border-bottom: 1px solid var(--surface-muted);
}
```

### Navigation hierarchy

Navigation follows the semantic hierarchy, never files/pages/widgets:

```text
Mission → Case → Situation → Settlement Queue → Current Settlement
          → Evidence / Policy / Action / Trace / Capability Update
```

Record grounding (spec-full.md): **Mission** = workspace/cell (`cell_id`); **Case** = the CMMN case (stages, plan items, milestones); **Situation** = the case-state projection — active/enabled items, satisfied/unsatisfied sentries, pending approvals; **Settlement Queue** = pending approvals + enabled human tasks + unsettled runs; **Current Settlement** = the SettlementEvent and its declarations. Case Detail must answer the sentry question: *why is this item not active yet?*

Chat, board, graph, terminal, and docs are never separate modes — each is a projection of the same semantic model. No mode switches.

## 6. Components

Every component corresponds to a semantic primitive (Situation, Affordance, Decision, Action, Evidence, Policy, Actor, Resource, Capability, Settlement, Trace, Notification), has a CLI equivalent, and leaves evidence. The spine — build these five first: **SituationHeader, AffordanceCard, PolicyGate, EvidenceChecklist, SettlementPanel.**

### Status Badge (semantic states)

```css
.badge {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 2px 8px;
  border-radius: 2px;
}

.badge--authority-allowed {
  background: rgba(22, 163, 74, 0.15);
  color: var(--color-authority-allowed);
  border: 1px solid rgba(22, 163, 74, 0.3);
}

.badge--authority-denied {
  background: rgba(220, 38, 38, 0.15);
  color: var(--color-authority-denied);
  border: 1px solid rgba(220, 38, 38, 0.3);
}

.badge--evidence-missing {
  background: rgba(245, 158, 11, 0.15);
  color: var(--color-evidence-missing);
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.badge--settlement-ready {
  background: rgba(37, 99, 235, 0.15);
  color: var(--color-settlement-ready);
  border: 1px solid rgba(37, 99, 235, 0.3);
}
```

### Settlement Panel

The primary focus organism. Semantic object: `Settlement`. User question: *Can I trust this outcome?*

```css
.settlement-panel {
  background: var(--surface-panel-elevated);
  border: 1px solid var(--color-settlement-ready);
  border-radius: 4px;
  padding: var(--panel-padding);
}

.settlement-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--fg-primary);
}

.settlement-decision-bar {
  display: flex;
  gap: var(--space-2);
  padding-top: var(--space-3);
  border-top: 1px solid var(--surface-muted);
  min-height: var(--row-review);
}
```

### Affordance Option (choice architecture)

Every decision offers exactly three options — Recommended, Safe, Experimental — each showing payment, risk, confidence. Never twenty equal buttons.

**Data-source boundary (spec-full §2.4):** SEA Forge never prices or ranks affordances — payment/valuation comes from CognitiveOS and routing from GodSpeed-Agent, both external feeds. SEA Forge supplies risk (policy verdicts), confidence (capability records), and observed `orchestration_burden`. With no external feed, the payment pill shows observed burden or hides, and the option set degrades to the spec-native queue: pending approvals + enabled items, ordered, top item recommended.

**Decisions are approvals, not settlements.** Settlement status is computed from criteria declared *before* execution (manufactured settlement is the spec's primary threat). The decision bar acts on `ApprovalRequest`s and human tasks (`sea-forge approve|reject`, `task complete`) — there is no "accept settlement" button.

```css
.affordance-option {
  display: grid;
  grid-template-columns: 1fr auto auto auto;
  align-items: center;
  gap: var(--space-3);
  min-height: var(--row-standard);
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--surface-muted);
  border-radius: 4px;
}

.affordance-option--recommended {
  border-color: var(--color-settlement-ready);
  background: rgba(37, 99, 235, 0.08);
}

.payment-pill {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--fg-secondary);
  padding: 2px 8px;
  border: 1px solid var(--surface-muted);
  border-radius: 999px;
}
```

### Confidence Meter

```css
.confidence-meter {
  height: 4px;
  background: var(--surface-muted);
  border-radius: 2px;
  overflow: hidden;
}

.confidence-meter-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 300ms ease-out;
}

.confidence-meter-fill.high   { background: var(--color-risk-low); }    /* ≥ 80% */
.confidence-meter-fill.medium { background: var(--color-risk-medium); } /* 50–79% */
.confidence-meter-fill.low    { background: var(--color-risk-high); }   /* < 50% */
```

### Evidence Checklist Row

States: complete, missing_required, stale, failed, pending. A missing required item blocks the accept action.

```css
.evidence-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  min-height: var(--row-standard);
  border-bottom: 1px solid var(--surface-muted);
}

.evidence-item--missing-required { color: var(--color-evidence-missing); }
.evidence-item--failed           { color: var(--color-evidence-failed); }
.evidence-item--quarantined     { color: var(--color-evidence-quarantined); }
```

### Policy Gate

Semantic object: `PolicyDecision`. When authority is denied: high-contrast warning, no green affordance anywhere in the panel, primary button disabled, explanation visible, next safe action highlighted.

```css
.policy-gate--denied {
  background: rgba(220, 38, 38, 0.08);
  border: 1px solid var(--color-authority-denied);
  border-left: 4px solid var(--color-authority-denied);
  border-radius: 4px;
  padding: var(--space-3) var(--space-4);
}

.policy-gate-explanation { color: var(--fg-primary); font-size: 14px; }
.policy-gate-next-move   { color: var(--color-settlement-ready); font-weight: 500; }
```

### Command Snippet (CLI equivalence)

Every panel exposes its CLI equivalent. Errors are corrective — they name the missing evidence and the next affordable move, never `Error: invalid state`.

```css
.command-snippet {
  font-family: var(--font-mono);
  font-size: 14px;
  color: var(--fg-secondary);
  background: var(--surface-workspace);
  border: 1px solid var(--surface-muted);
  border-radius: 4px;
  padding: var(--space-2) var(--space-3);
}
```

### Agent Status Pill

Agents render as capabilities (Research, Verification, Implementation…), never vendor names. Peripheral weight only.

```css
.agent-status-pill {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  font-size: 12px;
  color: var(--fg-secondary);
  background: var(--surface-panel);
  border: 1px solid var(--surface-muted);
  border-radius: 999px;
  padding: 2px 10px;
}

.agent-status-pill--blocked { border-color: var(--color-evidence-missing); }
```

## 7. Motion & Interaction

Motion is governance signal, not delight. Success gets **no motion** — settlement acceptance is calm. Only risk and required attention move.

| Interaction | Duration | Easing | Effect |
|-------------|----------|--------|--------|
| Attention pulse (authority/evidence) | 2s | ease-in-out | Subtle border glow, loops until addressed |
| Risk state change | 150ms | ease-out | Brief background flash |
| Success / settlement accepted | 0ms | — | No animation, state changes instantly |
| Background/peripheral updates | 0ms | — | No animation ever |
| Panel focus shift | 200ms | ease-out | Opacity + border transition |
| Confidence fill | 300ms | ease-out | Width transition |

```css
--transition-fast: 100ms ease-in;
--transition-base: 150ms ease-out;
--transition-slow: 300ms ease-out;
```

### prefers-reduced-motion

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

## 8. Voice & Brand

### Iconography

Lucide icons, 1.5px stroke, 16px default. Every icon communicates a semantic state; no decorative icons.

### Tone

- **Cybernetic, not dopamine**: never "Congratulations!" — instead "Evidence quality increased." "Settlement reliability improved." "Risk reduced."
- **Corrective**: every error teaches the next affordable move ("Settlement blocked: approval apr_0007 pending. Next move: run `sea-forge approve <run_id> apr_0007`.") — the CLI is `sea-forge`, and every panel's command snippet uses real spec commands (`run`, `resume`, `approve|reject`, `tasks`, `watch`, `capability show`, `ledger verify`)
- **Semantic**: notifications name state changes ("Settlement ready", "Authority required", "Evidence missing", "Capability promoted"), never mechanics ("Agent finished")
- **Recognition over recall**: the UI always shows the recommended next settlement; it never asks the user what to do

### Visual Signals

Color is the primary signal carrier and every hue is bound to a semantic token. The enforced test for any visual distinction: *what cognitive job does it perform?* If it has no answer, it does not ship.

## 9. Anti-patterns

- Do not name tokens by appearance (`red_500`) — only by cognitive function (`authority_denied`)
- Do not organize navigation around files, chats, tasks, or agents — files are artifacts, not navigation primitives
- Do not show more than one primary focus per screen — exactly one highlighted settlement
- Do not animate success or background updates — motion is reserved for risk and required attention
- Do not present more than three decision options — Recommended / Safe / Experimental, each with payment, risk, confidence
- Do not notify on mechanics ("agent finished") — only on semantic state changes
- Do not use giant cards or card soup — high density, progressive disclosure
- Do not show green success affordances anywhere in an authority-denied panel
- Do not name agents by vendor (Claude, GPT, Gemini) — capabilities only
- Do not ship a component without its CLI equivalent, `.sea` mapping, and evidence obligation
