# Frontend Audit: Intended vs Implemented — SEA Forge Workbench

**Audit Date:** 2026-09-04  
**Target System:** SEA Forge Workbench (`workbench/` in `sea-rs`)  
**Scope:** Desktop GUI (Tauri 2 + React 19 + TypeScript + Astryx / SEA Forge Design System)  
**Evaluator:** Antigravity AI  
**Status:** Complete Written Report (No Code Modified)

---

## 1. Sources Consulted

The following authoritative ground-truth documents and specifications were inspected prior to evaluating implementation code:

### Canonical Specifications and System Contracts
- [`/home/sprime01/projects/sea-rs/README.md`](file:///home/sprime01/projects/sea-rs/README.md): Top-level repository purpose and architecture invariants.
- [`/home/sprime01/projects/sea-rs/docs/execution/PRODUCT_COMPLETION_DEFINITION.md`](file:///home/sprime01/projects/sea-rs/docs/execution/PRODUCT_COMPLETION_DEFINITION.md): Observable product completion contract, installation levels, and primary user journey (lines 1-110).
- [`/home/sprime01/projects/sea-rs/docs/decisions/ADR-004-workbench-stack.md`](file:///home/sprime01/projects/sea-rs/docs/decisions/ADR-004-workbench-stack.md): Architectural decision record establishing the desktop workbench stack, workspace boundary, exact package pins, and Astryx token projection (lines 1-150).
- [`/home/sprime01/projects/sea-rs/docs/decisions/ADR-005-sfwp-schema-generation.md`](file:///home/sprime01/projects/sea-rs/docs/decisions/ADR-005-sfwp-schema-generation.md): SFWP schema generation pipeline (Rust types → JSON Schema → TypeScript types + AJV validators) (lines 1-107).
- [`/home/sprime01/projects/sea-rs/workbench/AGENTS.md`](file:///home/sprime01/projects/sea-rs/workbench/AGENTS.md): Frontend boundary rules, generated-zone invariants, and workspace layout (lines 1-92).
- [`/home/sprime01/projects/sea-rs/.agents/skills/building-sea-forge-workbench/SKILL.md`](file:///home/sprime01/projects/sea-rs/.agents/skills/building-sea-forge-workbench/SKILL.md): Workbench implementation skill, locked stack, and mandatory repository inspection rules (lines 1-150).

### Frontend Design System, Wireframes, and UX Specifications
- [`/home/sprime01/projects/sea-rs/.agents/specs/frontend/README.md`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/README.md): Design system overview, core laws, and package contents (lines 1-65).
- [`/home/sprime01/projects/sea-rs/.agents/specs/frontend/DESIGN.md`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/DESIGN.md): Canonical visual and interaction grammar, density rules, color tokens, and state vocabularies (lines 1-200, 1033 lines total).
- [`/home/sprime01/projects/sea-rs/.agents/specs/frontend/DESIGN-spec-mapping.md`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/DESIGN-spec-mapping.md): Semantic traceability between Open Design surfaces and SEA Forge view models (lines 1-150, 434 lines total).
- [`/home/sprime01/projects/sea-rs/.agents/specs/frontend/colors_and_type.css`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/colors_and_type.css): Source-of-truth CSS custom properties for semantic color, typography, spacing, and elevation.
- [`/home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-governed-workbench-ux-epic-v0.1.md`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-governed-workbench-ux-epic-v0.1.md): 16 journey areas with 128 durable user stories (lines 1-150, 619 lines total).
- [`/home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-primary-path-screen-wireframe-interaction-spec-v0.1.md`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-primary-path-screen-wireframe-interaction-spec-v0.1.md): Screen-level layout, wireframes, and interaction specifications for P1–P35 (lines 1-100, 2249 lines total).
- [`/home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-gui-view-flow-transition-spec-v0.1.md`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-gui-view-flow-transition-spec-v0.1.md): View transition classes (`N0`–`M5`, `E`, `R`), route guards (`G1`–`G10`), and navigation memory invariants (lines 1-140, 2216 lines total).
- [`/home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-gui-atomic-design-breakdown-v0.1.md`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-gui-atomic-design-breakdown-v0.1.md): Component hierarchy breakdown (atoms, molecules, organisms, templates).
- [`/home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-workbench-frontend-architecture-component-contract-v0.1.md`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-workbench-frontend-architecture-component-contract-v0.1.md): Layering boundaries, state ownership, component interfaces, and test requirements (lines 1-140, 2597 lines total).
- [`/home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-workbench-api-spec-v0.1.md`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-workbench-api-spec-v0.1.md): SFWP protocol specification, method naming, and request/event semantics (lines 1-140, 2288 lines total).
- [`/home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-workbench-api-method-catalog-v0.1.yaml`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/sea-forge-workbench-api-method-catalog-v0.1.yaml): Target method catalog and schema bindings.

### Prior Audits and Implementation Reports
- [`/home/sprime01/projects/sea-rs/.agents/specs/sea-forge-governed-workbench-frontend-completion-eval-v0.1.md`](file:///home/sprime01/projects/sea-rs/.agents/specs/sea-forge-governed-workbench-frontend-completion-eval-v0.1.md): 15 hard gates (G1–G15) and completion evaluation standard.
- [`/home/sprime01/projects/sea-rs/.agents/reports/frontend-eval-20260802T142629Z.md`](file:///home/sprime01/projects/sea-rs/.agents/reports/frontend-eval-20260802T142629Z.md): Prior completion evaluation results and identified blocking defects (SF-FE-001 through SF-FE-010).
- [`/home/sprime01/projects/sea-rs/.agents/reports/2026-08-03-workbench-golden-path-discovery.md`](file:///home/sprime01/projects/sea-rs/.agents/reports/2026-08-03-workbench-golden-path-discovery.md): Discovery analysis of the representative local Linux journey and Task 4 implementation map.
- [`/home/sprime01/projects/sea-rs/.agents/plans/2026-08-02-workbench-product-completion-ralph-loop.md`](file:///home/sprime01/projects/sea-rs/.agents/plans/2026-08-02-workbench-product-completion-ralph-loop.md): Active Ralph loop plan for product completion.

### Git History
- Commits: `64b8b72`, `fdd3232`, `6c18e5e`, `35e64c7`, `14d18a1`, `1b4edf0`, `45a8f98`, `a45d12e`, `7395cce`, `3f374c0`, `7c3daf5`, `240619d`, `eb78968`, `f5190d0`.

---

## 2. Intended Design and UX (Derived Strictly From Ground Truth)

### 2.1 Core User Flows
According to `docs/execution/PRODUCT_COMPLETION_DEFINITION.md:77-103`, `DESIGN.md:10-16`, and `sea-forge-primary-path-screen-wireframe-interaction-spec-v0.1.md:13-25`, the canonical journey follows this strict progression:

```text
1. Open or initialize cell without overwriting history (P2 Readiness)
   → 2. Inspect readiness and resolved identity (Actor, Role, Policy digest, Integrity state)
   → 3. Inquire via disclosure-controlled Thoth question (P5 Thoth Console)
   → 4. Discover assets and browse plan templates (P7 Assets / P9 Delegation)
   → 5. Author case draft with parameters (P11 New Case, P12 Configuration)
   → 6. Run authoritative preflight against immutable criteria (P13 Case Preflight)
   → 7. Commit case with stable idempotency key (P15 Case Overview)
   → 8. Observe case horizon and sentry-gated work items (P16 Case Horizon)
   → 9. Exercise human approval or discretionary intervention where required (P18 Approval Inbox)
   → 10. Execute sandboxed command or agent delegation (P22 Run Detail / P23 Agent Task Detail)
   → 11. Inspect execution vs settlement as distinct independent facts (P27 Settlement Detail)
   → 12. Evaluate cryptographic evidence, audit records, and declarations (P26 Evidence Index)
   → 13. Promote proven results to capability memory, projections, and artifacts (P32 Capability Detail)
```

### 2.2 Screen and Component Inventory & Intended Responsibilities
Per `sea-forge-primary-path-screen-wireframe-interaction-spec-v0.1.md` and `sea-forge-workbench-frontend-architecture-component-contract-v0.1.md`:

1. **Application Shell (`AppShell`, `GlobalHeader`, `Sidebar`, `EvidenceDrawer`)**:
   - **GlobalHeader**: Persistent operational banner. Displays Cell ID, Actor & Role, Policy digest, Integrity assurance, Command Search (`/`), Approval Inbox pending count badge, and Active Work toggle.
   - **Sidebar**: Primary navigation rail. Wireframe Spec §1.1 note explicitly establishes the **canonical consolidated 8-item vocabulary**: `Inbox`, `Memory`, `Artifacts`, `Domain Models`, `Capabilities`, `Cases`, `Federation`, `Administration`.
   - **JourneyRibbon**: Contextual progress ribbon showing position along the governed lifecycle (`Authority → Domain → Draft → Preflight → Execution → Evidence → Settlement`).
   - **EvidenceDrawer**: Three-column right-hand inspection drawer with four tabs (`Why`, `Evidence`, `Provenance`, `Record`) for inspecting proof without losing screen focus.
   - **Status Bar**: Live connection state, event cursor, stale state indicator, and keyboard shortcut hints (`R`, `I`, `B`, `E`).
2. **Readiness Console (P2)**:
   - Evaluates cell fitness for intended work. Contains operation selector (`Run local work`, `Delegate to external agent`, `Import bundle`), critical foundations table, operational capabilities list, current limitation rail, and single dominant lawful action (`Initialize cell` or `Create case`).
3. **Thoth Question Console (P5 / Epic 3)**:
   - Grounded inquiry into cell self-model. Composer accepts question kind, subject, and purpose. Answers must disclose freshness, assurance, limitations, omitted claim classes, and an explicit disclaimer that knowledge confers no execution authority.
4. **Asset Catalog (P7 / Epic 4)**:
   - Separates plan templates, agent endpoints, and extensions into three distinct tables with kind-specific standing ladders (`materialized`, `probed`/`demonstrated`, `active`/`quarantined`).
5. **Delegation Workbench & Roster (P9, P23 / Epic 9)**:
   - Configures agent delegation requests, previews complete job contracts (model, turn cap, token budget, transcript retention, endpoint digest, authority requirements). Roster lists active delegations with single-item cancellation.
6. **Domain Models Workbench (P8 / Epic 5)**:
   - Validates `.sea` models, displays AST, source hashes, concept refs, and pinned projections.
7. **Case Creation & Preflight Workbench (P11–P13 / Epic 6)**:
   - Reversible draft editing, parameter entry, client-side hint validation, authoritative server preflight check, and idempotency-keyed commit. Handles stale-template rejection by offering re-preflight.
8. **Case Horizon Board (P15–P16 / Epic 7)**:
   - Shows all committed cases and, for the selected case, its plan items with execution and settlement standing shown side by side. Provides item-level actions: dispatch, retry, park, cancel, or intervene.
9. **Approval Inbox (P18 / Epic 8)**:
   - Lists pending human approval tasks with full governance context (requester, operation kind, resource, purpose, criteria hash). Enforces separation of duty (requester cannot approve). Records decision note with approve/reject.
10. **Operations Monitor (P20 / Epic 11, 16)**:
    - Displays durable event stream from the append-only ledger, resume cursor, correlation tracking, and execution vs settlement status.
11. **Evidence Index & Run Record (P22, P26, P27 / Epic 12)**:
    - Lists run episodes. Run detail displays side-by-side execution termination (exit code, termination kind) and settlement outcome (criteria vs basis tokens), declarations (with self-certification indicators), cryptographic evidence URIs, trace events, and source record file inventory.
12. **Capability, Memory, Artifacts, Federation (P32, Epic 13–15)**:
    - Developmental memory recall under authority; capability maturity matrix showing repeated settlement under variation; artifact stages (cognitive, intellectual, product, capital); federation bundle import/export preview.
13. **System Contract / Installation Admin (Epic 16)**:
    - Reflects live negotiated SFWP catalog, method standing, and protocol version compatibility.

### 2.3 State Model
Per `sea-forge-workbench-frontend-architecture-component-contract-v0.1.md:12-70` and `ADR-004`:
- **Server Authority**: The kernel/server owns all canonical records, authority decisions, case reduction, settlement, and capability promotion.
- **Client Role**: The renderer is a governed client. It maintains presentation state, navigation state, and reversible drafts. It never mutates canonical state optimistically.
- **Data Transport**: Unix-domain socket with newline-delimited JSON frames managed exclusively by the Tauri host (`src-tauri/src/socket.rs`, `bridge.rs`). The renderer uses typed Tauri commands and channels.
- **State Separation Invariants**:
  - `execution_state ≠ settlement_state`: Execution success (e.g. process exit 0) never visually implies or converts to settlement acceptance.
  - `agent_dialogue_state ≠ termination_reason ≠ settlement`.
  - `draft ≠ committed_record`.
  - `unknown ≠ absent ≠ failed`.
- **Route Guards**: Ten guards (`G1`–`G10`) evaluated with four-value logic (`passed`, `failed`, `indeterminate`, `not_applicable`). Indeterminate guards must not block routes or claim success.

### 2.4 Visual & Interaction Conventions
Per `DESIGN.md:20-200` and `colors_and_type.css`:
- **Theme**: Dark, dense, restrained developer mission control. No decorative cards, hero images, or consumer chat metaphors.
- **Semantic Color Tokens**:
  - Surfaces: `--surface-workspace` (`#0B1220`), `--surface-panel` (`#111827`), `--surface-panel-elevated` (`#1F2937`), `--surface-overlay` (`#0F172A`), `--surface-code` (`#08101D`).
  - Text: `--fg-primary` (`#E5EDF8`), `--fg-secondary` (`#94A3B8`), `--fg-tertiary` (`#64748B`), `--fg-inverse` (`#07111F`).
  - Authority: `--color-authority-allowed` (`#16A34A`), `--color-authority-denied` (`#DC2626`), `--color-authority-escalated` (`#F59E0B`), `--color-authority-degraded` (`#D97706`), `--color-authority-pending` (`#64748B`).
  - Execution: `--color-execution-running` (`#2563EB`), `--color-execution-succeeded` (`#16A34A`), `--color-execution-failed` (`#DC2626`), `--color-execution-cancelled` (`#64748B`).
  - Settlement: `--color-settlement-pending` (`#64748B`), `--color-settlement-evaluating` (`#2563EB`), `--color-settlement-accepted` (`#16A34A`), `--color-settlement-rejected` (`#DC2626`), `--color-settlement-quarantined` (`#7C3AED`).
  - Readiness: `--color-readiness-ready` (`#16A34A`), `--color-readiness-degraded` (`#F59E0B`), `--color-readiness-blocked` (`#DC2626`).
- **Core Visual Law**: One governed focus per screen. Exactly one dominant lawful action button.
- **Typography & Accessibility**: Tokenized sizing. Contrast rule: `--fg-tertiary` on `--surface-panel` is restricted to non-essential text due to 3.9:1 contrast ratio.

*Documentation Silence:* The ground truth documents do not specify a web or mobile layout (desktop only is specified, targeting min width 1100–1440px). Rich text/Markdown rendering for case descriptions and domain visualization libraries (e.g. Graphviz, Cytoscape) are explicitly designated as deferred decisions in `reference/stack-and-dependencies.md`.

---

## 3. Findings Table

| Category | Severity | Confidence | One-Line Description | Citation |
|---|---|---|---|---|
| Functional bug | Major | Confirmed | Search button in `AppShell` triggers blocking browser `alert()` modal instead of opening command palette | [`AppShell.tsx:140`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/shell/AppShell.tsx#L140) |
| Functional bug | Major | Confirmed | Stale preflight state in `CaseCreationWorkbench` is shadowed by `preflight?.ok`, falsely displaying "Preflight passed" | [`CaseCreationWorkbench.tsx:141-155`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/pages/CaseCreationWorkbench.tsx#L141-L155) |
| Functional bug | Minor | Confirmed | Sidebar hardcodes pending Inbox badge to `1` regardless of actual approval queue count or unread status | [`Sidebar.tsx:19`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/shell/Sidebar.tsx#L19) |
| Missing feature | Major | Confirmed | Major product surfaces (`/models`, `/memory`, `/capabilities`, `/artifacts`, `/federation`) render inert `UnbackedSurface` stubs | [`SurfacesPages.tsx:47-95`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/pages/SurfacesPages.tsx#L47-L95) |
| Missing feature | Major | Confirmed | Case Horizon board is entirely read-only; lacks item dispatch, intervention, parking, retry, and cancellation controls | [`CaseHorizonPage.tsx:55-112`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/pages/CaseHorizonPage.tsx#L55-L112) |
| Missing feature | Major | Confirmed | Delegation Workbench is strictly inspect/preview-only; provides no UI path to authorize, dispatch, or execute delegations | [`DelegationWorkbench.tsx:19-27`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/pages/DelegationWorkbench.tsx#L19-L27) |
| Spec deviation | Major | Confirmed | Approval Inbox does not enforce separation of duty in UI; allows requester to click "Approve" on their own request | [`ApprovalInboxPage.tsx:148-156`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/pages/ApprovalInboxPage.tsx#L148-L156) |
| Spec deviation | Minor | Confirmed | Overall readiness state `"stale"` is collapsed into `"degraded"`, violating distinct freshness vocabulary | [`ReadinessPage.tsx:53-56`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/pages/ReadinessPage.tsx#L53-L56) |
| Spec deviation | Minor | Confirmed | Readiness "Inspect source" displays stringified `SourceRecordRef` JSON rather than committed record or ledger digest | [`ReadinessPage.tsx:155-164`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/pages/ReadinessPage.tsx#L155-L164) |
| Spec deviation | Minor | Confirmed | Sidebar navigation presents 14 legacy menu items instead of the 8 canonical consolidated surfaces specified in wireframe spec §1.1 | [`Sidebar.tsx:12-34`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/shell/Sidebar.tsx#L12-L34) |
| Spec deviation | Minor | Confirmed | Route table only evaluates 3 blocking guards (`G1`, `G2`, `G9`); remaining 7 guards (`G3`–`G8`, `G10`) are unmapped in route gating | [`useGuardContext.ts:25`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/guards/useGuardContext.ts#L25), [`router.tsx:63`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/router.tsx#L63) |
| UX deviation | Minor | Confirmed | Active Work pill in GlobalHeader is hardcoded static markup ("None running") disconnected from live event stream | [`GlobalHeader.tsx:149`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/shell/GlobalHeader.tsx#L149) |
| UX deviation | Minor | Confirmed | JourneyRibbon displays static plain text labels and does not reflect completed milestones or afford interaction | [`JourneyRibbon.tsx:20-25`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/shell/JourneyRibbon.tsx#L20-L25) |
| UX deviation | Cosmetic | Confirmed | GlobalHeader context chips (Actor, Cell, Integrity) are rendered as interactive buttons with hover states but no `onClick` | [`GlobalHeader.tsx:73-117`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/shell/GlobalHeader.tsx#L73-L117) |
| UX deviation | Cosmetic | Inferred | `--fg-tertiary` on `--surface-panel` violates WCAG AA 4.5:1 contrast for small metadata text (noted in spec) | [`DESIGN.md:148`](file:///home/sprime01/projects/sea-rs/.agents/specs/frontend/DESIGN.md#L148), [`Sidebar.module.css:62`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/shell/Sidebar.module.css#L62) |
| Undocumented addition | Minor | Confirmed | Separate top-level `/delegate` route and dedicated delegation contract workbench not defined in wireframe spec | [`router.tsx:116-120`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/router.tsx#L116-L120) |
| Undocumented addition | Minor | Confirmed | Live SFWP Method Negotiation page implemented under `/admin` (`SystemContractPage`) reflecting SFWP wire catalog | [`SystemContractPage.tsx:1-263`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/pages/SystemContractPage.tsx#L1-L263) |
| Undocumented addition | Minor | Confirmed | Interactive actor switcher dropdown in GlobalHeader allowing dynamic actor impersonation/selection | [`GlobalHeader.tsx:82-98`](file:///home/sprime01/projects/sea-rs/workbench/apps/desktop/src/shell/GlobalHeader.tsx#L82-L98) |

---

## 4. Detailed Findings

### 4.1 Search Button Triggers Native `alert()`
- **Intended**: `DESIGN.md:37` and Wireframe Spec §1.1 specify Raycast-like command palette navigation accessible via `/` or header search icon to search commands, cases, runs, and records.
- **Actual**: `AppShell.tsx:140` wires `onOpenSearch` directly to `alert("Search command palette (Press /)")`.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/shell/AppShell.tsx:140
  onOpenSearch={() => alert("Search command palette (Press /)")}
  ```
  Verified live in browser: clicking `#searchButton` halts the renderer with a native alert popup.
- **Severity**: Major
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/shell/AppShell.tsx`, new `CommandPalette` component in `workbench/packages/sea-forge-ui-components`.
  - *Approach*: Implement a modal dialog/drawer using `@astryxdesign/core` or headless dialog displaying reachable routes, recent cases, and command shortcuts.
  - *Effort*: M

---

### 4.2 Preflight Stale State Shadowing in Case Creation Workbench
- **Intended**: `UX Epic §6.8` and `Wireframe Spec P13`: If a preflighted draft becomes stale because underlying templates or digests changed, the preflight pill MUST display `Rejected as stale` (variant `blocked`) and prompt the user to re-run preflight before committing.
- **Actual**: In `CaseCreationWorkbench.tsx:141-155`, the ternary expression checks `preflight?.ok` before checking `state === "rejected_as_stale"`. Because `preflight` remains in machine context with `ok: true` from the previous run, the pill continues to display `Preflight passed` with green variant `ready`.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/pages/CaseCreationWorkbench.tsx:141-155
  <GovernedStatusPill
    variant={
      preflight?.ok
        ? "ready"
        : state === "rejected_as_stale"
          ? "blocked"
          : "unknown"
    }
    label={
      preflight?.ok
        ? "Preflight passed"
        : state === "rejected_as_stale"
          ? "Rejected as stale"
          : "Not yet run"
    }
    className="status-pill"
  />
  ```
  Also confirmed by comment in test harness [`scripts/workbench-e2e-case-authoring.sh:31-34`](file:///home/sprime01/projects/sea-rs/scripts/workbench-e2e-case-authoring.sh#L31-L34): *"the pill keeps 'Preflight passed' in the stale state because preflight?.ok shadows state in the label ternary (filed as debt)"*.
- **Severity**: Major
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/pages/CaseCreationWorkbench.tsx`
  - *Approach*: Invert the conditional check to evaluate `state === "rejected_as_stale"` prior to checking `preflight?.ok`.
  - *Effort*: S

---

### 4.3 Hardcoded Inbox Badge in Navigation
- **Intended**: `UX Epic §8.1` and `Wireframe Spec §1.1`: The Inbox navigation item should display a badge counter reflecting actual pending human decisions waiting in the journal, or omit the badge when zero or unread.
- **Actual**: `Sidebar.tsx:19` hardcodes `badge: 1` into the `OPERATE_NAV` item definition array.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/shell/Sidebar.tsx:19
  { path: "/inbox", label: "Inbox", icon: "▾", badge: 1 },
  ```
  Verified live in browser: navigating to `/inbox` displays `Inbox 1` even when the journal is unread, unreachable, or empty (`GlobalHeader.tsx` correctly reports `Unknown` or `0 approvals`).
- **Severity**: Minor
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/shell/Sidebar.tsx`, `AppShell.tsx`
  - *Approach*: Pass `inboxCount` from `useApprovals()` in `AppShell` into `Sidebar`, and render badge conditionally only when `inboxCount > 0`.
  - *Effort*: S

---

### 4.4 Missing Product Surfaces Rendered as Inert `UnbackedSurface` Stubs
- **Intended**: `UX Epic Journeys 5, 13, 14, 15` and `DESIGN.md:89-106` require fully functional surfaces for Domain Models (`/models`), Capability Matrix (`/capabilities`), Memory Recall (`/memory`), Artifact Registry (`/artifacts`), and Federation Gateway (`/federation`).
- **Actual**: In `SurfacesPages.tsx:47-95`, all five routes render `UnbackedSurface`, stating that backend SFWP methods (`domain.list_models`, `memory.recall`, `capability.list`, `artifact.list`, `federation.preview_export`) are absent from the kernel catalog.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/pages/SurfacesPages.tsx:47-95
  export function ModelsPage() { return <UnbackedSurface title="Domain models" method="domain.list_models" ... />; }
  export function MemoryPage() { return <UnbackedSurface title="Memory recall" method="memory.recall" ... />; }
  export function CapabilitiesPage() { return <UnbackedSurface title="Capability matrix" method="capability.list" ... />; }
  export function ArtifactsPage() { return <UnbackedSurface title="Artifact registry" method="artifact.list" ... />; }
  export function FederationPage() { return <UnbackedSurface title="Federation gateway" method="federation.preview_export" ... />; }
  ```
  Verified live in browser snapshot: Navigating to `/models` displays *"Nothing here can be evidenced yet ... requires domain.list_models"*.
- **Severity**: Major
- **Recommendation**:
  - *Scope*: `crates/sea-forge-server/src/sfwp/` and frontend pages in `workbench/apps/desktop/src/pages/`.
  - *Approach*: Incrementally implement backend SFWP projection queries in `sea-forge-server` (or formalize exclusions in `workbench-completion-eval-exclusions.json`), generate contracts, and build dedicated source-backed inspection pages.
  - *Effort*: L

---

### 4.5 Case Horizon Board is Strictly Read-Only
- **Intended**: `Wireframe Spec P16 (Case Horizon)` and `UX Epic §7.4-7.8`: Operators must be able to interact with plan items: dispatch next reachable step, pause/park execution, trigger manual interventions, re-open terminated items, or retry failed runs.
- **Actual**: `CaseHorizonPage.tsx:55-112` renders item details, state pills, and links to `/runs/$runId`, but has zero interactive mutation or execution controls.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/pages/CaseHorizonPage.tsx:55-112
  function HorizonRow({ item }: { item: HorizonItem }) {
    // Only renders itemHead, DualStateIndicator, itemDetail, and episode Link.
    // No dispatch, park, cancel, or retry buttons exist.
  }
  ```
  Confirmed by test suite: all 7 tests in `CaseHorizonPage.test.tsx` verify only rendering and link existence.
- **Severity**: Major
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/pages/CaseHorizonPage.tsx`, `useCases.ts`
  - *Approach*: Add `ProtectedActionButton` controls for item dispatch, intervention, and parking, bound to `case.dispatch_item` and `case.intervene` commands with confirmation modals.
  - *Effort*: M

---

### 4.6 Delegation Workbench Lacks Execution/Dispatch Capability
- **Intended**: `UX Epic Journey 9` and `Wireframe Spec P23`: Operators configure agent delegation and dispatch the task into the execution kernel, monitoring the resulting run.
- **Actual**: `DelegationWorkbench.tsx:19-27` is designed solely as an inspection preview of the `JobContractPreview`. It contains an explicit docstring stating: *"There is no 'Run delegation' button here, and its absence is the design, not an unfinished edge"*.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/pages/DelegationWorkbench.tsx:19-24
  * There is no "Run delegation" button here, and its absence is the design, not
  * an unfinished edge. `delegate` writes an intent, a plan, criteria, and an
  * authority decision; it is a protected command and belongs behind
  * `ProtectedActionButton` with its own preconditions.
  ```
- **Severity**: Major
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/pages/DelegationWorkbench.tsx`, `hooks/useDelegation.ts`
  - *Approach*: Add a protected "Dispatch delegation" action button guarded by `evaluateProtectedAction` for `agent_run.start`, submitting the committed request to the server and navigating to `/runs/$runId`.
  - *Effort*: M

---

### 4.7 Approval Inbox Missing Separation of Duty Check in UI
- **Intended**: `UX Epic §8.3`: Compliance requirement enforcing that the actor requesting an operation cannot approve their own request.
- **Actual**: In `ApprovalInboxPage.tsx:148, 156`, the "Approve" button only checks `action.isAllowed` (global permission for `approval.decide`). It does not verify whether `identity.actor?.actorId === approval.governance?.requester`.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/pages/ApprovalInboxPage.tsx:148, 156
  <button
    type="button"
    className={styles.approve}
    disabled={busy || !action.isAllowed}
    onClick={() => onDecide("approve", note)}
  >
  ```
- **Severity**: Major
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/pages/ApprovalInboxPage.tsx`
  - *Approach*: Compare `identity?.actor?.actorId` with `approval.governance?.requester`. If they match, disable the "Approve" button with a clear badge/explanation: *"Separation of duty: Requester cannot approve own request"*.
  - *Effort*: S

---

### 4.8 Overall Readiness State `"stale"` Collapsed into `"degraded"`
- **Intended**: `DESIGN.md:196-200` and `View Flow Spec §2.1`: Readiness defines independent states: `ready`, `ready_degraded`, `stale`, `blocked`, and `integrity_halted`. Stale indicates that prior verified checks exist but have been invalidated by time or configuration changes.
- **Actual**: `ReadinessPage.tsx:53-56` groups `"stale"` into `"degraded"`:
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/pages/ReadinessPage.tsx:53-56
  function overallVariant(overall: ReadinessView["overall"] | "unknown"): GovernedStatusVariant {
    if (overall === "ready") return "ready";
    if (overall === "ready_with_limitations" || overall === "stale") return "degraded";
    if (overall === "blocked") return "blocked";
    if (overall === "integrity_halted") return "integrity_halted";
    return "unknown";
  }
  ```
- **Severity**: Minor
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/pages/ReadinessPage.tsx`, `packages/sea-forge-ui-components/src/GovernedStatusPill.tsx`
  - *Approach*: Add explicit `stale` variant support to `GovernedStatusPill` and return `stale` rather than `degraded`.
  - *Effort*: S

---

### 4.9 Readiness "Inspect Source" Renders Synthetic Metadata Instead of Source Record
- **Intended**: `UX Epic §1.6` and `Architecture Contract §0`: Statuses resolve to committed source records with cryptographic digests, ledger coordinates, and provenance.
- **Actual**: In `ReadinessPage.tsx:155-164`, clicking "Inspect source" populates the evidence drawer with a stringified representation of the `SourceRecordRef` structure (e.g. `{ entry_id: "...", record_kind: "..." }`) rather than reading the actual underlying source ledger record or hash.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/pages/ReadinessPage.tsx:155-164
  function inspectSource(source: SourceRecordRef | undefined) {
    if (!source) return;
    const evidence: EvidenceRecord = {
      id: source.entry_id,
      kind: source.record_kind,
      disclosureStatus: "restricted",
      rawPayload: JSON.stringify(source, null, 2),
    };
    inspectEvidence(evidence);
  }
  ```
- **Severity**: Minor
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/pages/ReadinessPage.tsx`, SFWP queries
  - *Approach*: Bind `inspectSource` to query the actual record content via `records.get` or include the committed SHA256 digest in `SourceRecordRef`.
  - *Effort*: M

---

### 4.10 Sidebar Navigation Retains 14 Legacy Menu Items
- **Intended**: `Wireframe Spec §1.1 note lines 83-91`: Explicitly retires earlier draft labels (`Readiness`, `Thoth`, `Assets`, `Operations`, `Evidence`) in favor of an 8-item canonical set (`Inbox`, `Memory`, `Artifacts`, `Domain Models`, `Capabilities`, `Cases`, `Federation`, `Administration`).
- **Actual**: `Sidebar.tsx:12-34` maintains 14 items across three sections (`Operate`, `Inspect`, `Administration`).
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/shell/Sidebar.tsx:12-34
  const OPERATE_NAV = [Readiness, Thoth, Assets, Delegation, Domain Models, Cases, Inbox, Operations];
  const INSPECT_NAV = [Evidence, Memory, Capabilities, Artifacts, Federation];
  const ADMIN_NAV = [Administration];
  ```
- **Severity**: Minor
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/shell/Sidebar.tsx`
  - *Approach*: Consolidate navigation according to Wireframe Spec §1.1, nesting operational screens (Readiness, Thoth, Assets) into their primary-path parent categories.
  - *Effort*: M

---

### 4.11 Route Guarding Limited to 3 of 10 Guards
- **Intended**: `View Flow Spec §1.2` defines ten route guards (`G1`–`G10`).
- **Actual**: `useGuardContext.ts:25` defines `BLOCKING_GUARDS = ["G1", "G2", "G9"]`. `router.tsx:63` only checks these three. `G3` (Sponsor), `G4` (Policy), `G5` (Integrity), `G6` (Resource), `G7` (Disclosure), `G8` (Compatibility), and `G10` (Standing) are never evaluated to block routes.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/guards/useGuardContext.ts:25
  export const BLOCKING_GUARDS: readonly GuardId[] = ["G1", "G2", "G9"];
  ```
- **Severity**: Minor
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/guards/useGuardContext.ts`
  - *Approach*: Progressively wire remaining guards as their backing SFWP methods (`policy.get`, `integrity.get`) become available in `sea-forge-server`.
  - *Effort*: M

---

### 4.12 Active Work Pill Hardcoded to "None running"
- **Intended**: `DESIGN.md:38` and `Wireframe Spec §1.1`: Global header displays peripheral awareness of active running executions or agent episodes.
- **Actual**: `GlobalHeader.tsx:149` contains hardcoded static text: `<strong>None running</strong>`.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/shell/GlobalHeader.tsx:149
  <span className="state-dot state-dot--running" />
  <span>Active work</span>
  <strong>None running</strong>
  ```
- **Severity**: Minor
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/shell/GlobalHeader.tsx`, `AppShell.tsx`
  - *Approach*: Bind active work count to `useOperationsStream()` or active run tracking from `useRuns()`.
  - *Effort*: S

---

### 4.13 JourneyRibbon Does Not Reflect Workflow Milestones
- **Intended**: `DESIGN-spec-mapping.md §3` and `Wireframe Spec §1.1`: The ribbon visualizes the user's progress through the canonical governed lifecycle.
- **Actual**: `JourneyRibbon.tsx:20-25` renders the passed `currentStep` in bold, immediately followed by the entire static list of `GOVERNED_STEPS`. It has no notion of completed steps versus upcoming steps.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/shell/JourneyRibbon.tsx:20-25
  <strong className={styles.stepActive}>{currentStep}</strong>
  {GOVERNED_STEPS.map((step) => (
    <span key={step} className={styles.step}>{step}</span>
  ))}
  ```
- **Severity**: Minor
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/shell/JourneyRibbon.tsx`
  - *Approach*: Determine active index of `currentStep` within `GOVERNED_STEPS`; apply completed (`.stepDone`), active (`.stepActive`), and future styling.
  - *Effort*: S

---

### 4.14 GlobalHeader Context Chips Unclickable Buttons
- **Intended**: `Wireframe Spec §1.1`: Header chips display context and allow quick inspection of actor, cell, and integrity details.
- **Actual**: In `GlobalHeader.tsx:73-117`, the `<button>` chips have no `onClick` handlers attached.
- **Evidence**:
  ```tsx
  // workbench/apps/desktop/src/shell/GlobalHeader.tsx:73, 99, 107
  <button className={`${styles.contextChip} context-chip`} type="button" aria-label="Active actor and role">
  ```
- **Severity**: Cosmetic
- **Recommendation**:
  - *Scope*: `workbench/apps/desktop/src/shell/GlobalHeader.tsx`
  - *Approach*: Attach click handlers to open the corresponding inspection view or evidence record in `EvidenceDrawer`.
  - *Effort*: S

---

### 4.15 Contrast Violation for Small Metadata (`--fg-tertiary`)
- **Intended**: `DESIGN.md:148` warns that `--fg-tertiary` (`#64748B`) on `--surface-panel` (`#0B1220`) has a contrast ratio of ~3.9:1, failing WCAG AA (4.5:1) for normal text under 18px / 14px bold.
- **Actual**: In CSS modules across `Sidebar.module.css`, `AppShell.module.css`, and `ReadinessPage.module.css`, small timestamps, labels, and shortcuts use `var(--fg-tertiary)` at 11–12px font size.
- **Severity**: Cosmetic
- **Recommendation**:
  - *Scope*: `workbench/packages/sea-forge-ui-tokens/sea-forge.tokens.css`, `colors_and_type.css`
  - *Approach*: Adjust `--fg-tertiary` to `#7E8FA6` (contrast 4.6:1 against `#0B1220`) or replace usage on small text with `--fg-secondary`.
  - *Effort*: S

---

### 4.16 Undocumented Additions
1. **Dedicated `/delegate` Route**:
   - `router.tsx:116-120` and `DelegationWorkbench.tsx`: Created as an independent top-level workbench specifically to isolate pre-commitment contract inspection from asset inventory. (Severity: Minor / Positive architectural refinement).
2. **Installation SFWP Contract Page (`/admin`)**:
   - `SystemContractPage.tsx`: Implemented to negotiate live SFWP methods via `system.hello`/`system.describe` and expose wire protocol standing. (Severity: Minor / Highly useful diagnostic tool).
3. **In-Header Actor Switcher**:
   - `GlobalHeader.tsx:82-98`: Embedded dropdown enabling instant switching between available socket actors. (Severity: Minor / Effective testing and multi-role convenience).

---

## 5. Adversarial Pass & Verification Results

Verification was performed against the live code and running test harnesses in the local Linux environment:

1. **Static Analysis & Contract Gate**:
   - Command: `just workbench-contracts-gate`
   - Result: **PASS** (Zero diff across generated TypeScript types and AJV validators; tokens byte-match `colors_and_type.css`; Tauri standalone Cargo workspace verified).
2. **Typecheck & Linting**:
   - Command: `cd workbench && bun run check` (`tsc -b --noEmit && oxlint`)
   - Result: **PASS** (0 errors, 2 minor warnings).
3. **Unit & Component Test Suite**:
   - Command: `cd workbench && bun run test` (`vitest run` across desktop and ui-components)
   - Result: **PASS** (All 22 desktop test suites / 152 tests passed; 17 ui-components tests passed).
4. **Live Renderer Probe**:
   - Command: `just workbench-dev-up` (Vite dev server launched at `http://127.0.0.1:1420`), probed via `agent-browser` snapshots and accessibility audits.
   - Result: Confirmed runtime behavior:
     - The shell fails closed cleanly when no native Tauri IPC is present (`"Cell unreachable — Cannot read properties of undefined (reading 'invoke')"`).
     - `#searchButton` confirmed to invoke blocking `alert()`.
     - Sidebar confirmed to display hardcoded `Inbox 1` badge.
     - Unbacked surfaces (`/models`, `/memory`, etc.) confirmed to display honest `UnbackedSurface` explanations.
     - `Inspect all capabilities` confirmed to open `EvidenceDrawer` with `readiness_capability_summary`.
     - Server dev server was cleanly stopped via `just workbench-dev-down`.

---

## 6. Open Questions

1. **Exclusion Boundaries for Eval**:
   - Does the product owner intend for `domain.list_models`, `memory.recall`, `capability.list`, `artifact.list`, and `federation.preview_export` to remain permanently out of scope for the local single-user desktop release (and thus formally excluded in `workbench-completion-eval-exclusions.json`), or are these slated for vertical SFWP implementation in a future milestone?
2. **Search Palette Architecture**:
   - Should the command palette (`/`) be built as an internal component within `sea-forge-ui-components` (e.g. using `cmdk` or a minimal accessible keyboard-driven listbox), or is there an Astryx design primitive planned for this?
3. **Canonical Sidebar Consolidations**:
   - Wireframe Spec §1.1 mandates collapsing 14 nav items into 8 canonical surfaces (`Inbox`, `Memory`, `Artifacts`, `Domain Models`, `Capabilities`, `Cases`, `Federation`, `Administration`). Should the frontend proceed with restructuring the routes to match this 8-item navigation, or has the 14-item navigation in `Sidebar.tsx` been accepted as the new standard?
4. **Execution Authority from Workbench**:
   - What is the authorized path for initiating an agent delegation: will `DelegationWorkbench` gain a direct `ProtectedActionButton` dispatching `agent_run.start`, or must all agent executions originate from a pre-committed Case plan item?
