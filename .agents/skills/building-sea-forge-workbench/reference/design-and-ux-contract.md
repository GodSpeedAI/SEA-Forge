# Design and UX contract

## Source priority

1. `.agents/specs/frontend/DESIGN.md` — canonical visual/interaction grammar.
2. `sea-forge-primary-path-screen-wireframe-interaction-spec-v0.1.md` — exact
   screen behavior and acceptance criteria.
3. `sea-forge-gui-atomic-design-breakdown-v0.1.md` — component tiers/naming.
4. `DESIGN-spec-mapping.md`, `MOCKUP-BRIEF.md` — traceability and scope.
5. `ui_kits/app/` — visual reference for the shell, Readiness, and static
   Operate-route scenarios; staleness notes in `source-map.md`.

The design-system skill `.agents/specs/frontend/SKILL.md`
(`sea-forge-workbench-design`) governs design QA of produced surfaces; apply
it when reviewing visual output.

## Astryx role vs SEA Forge ownership

Astryx (pinned beta) supplies primitives, interaction components, layout
patterns, theming plumbing. SEA Forge owns all semantics: state vocabulary,
navigation, governed actions, tokens. Styling stack: Astryx precompiled CSS →
SEA Forge Astryx theme → CSS Modules for bespoke domain components.

## Token ownership

`.agents/specs/frontend/colors_and_type.css` is the canonical token source:
dark scheme; surfaces (`--surface-workspace` #0B1220, `--surface-panel`,
`--surface-panel-elevated`, `--surface-muted`, `--surface-overlay`,
`--surface-code`); text (`--fg-primary/secondary/tertiary/inverse` — never
essential small text in tertiary); and the **semantic state families that
must never collapse into generic success/warning/error**:

```text
--color-authority-allowed / -denied / -escalated / -degraded / -pending
--color-execution-running / -waiting / -succeeded / -failed / -cancelled
--color-dialogue-streaming / -permission / -terminated
(settlement, integrity, evidence families per DESIGN.md:
 settlement_evaluating / settlement_accepted / settlement_rejected,
 integrity_failed, evidence_quarantined)
```

Projections generated/maintained from the canonical tokens:
`sea-forge.tokens.css`, `sea-forge.astryx-theme.css`, token documentation,
Storybook token fixtures. Type: Inter + JetBrains Mono stacks with system
fallbacks (no bundled fonts yet); 4px spacing grid; radii ≤ 8px; exactly
three depth levels; governed motion (no decorative animation); no gradients
or glow.

## Package layout (adjust names to workspace conventions when created)

```text
workbench/packages/
  sea-forge-ui-tokens/        canonical tokens + generated projections
  sea-forge-astryx-theme/     SEA Forge theme over Astryx
  sea-forge-ui-primitives/    thin Astryx wrappers where needed
  sea-forge-ui-components/    semantic components (below)
  sea-forge-ui-patterns/      domain organisms (below)
```

## Component hierarchy

Use Astryx directly for generic pieces. SEA Forge-owned **semantic
components**: `GovernedStatusPill`, `DualStateIndicator`,
`SourceFreshnessBadge`, `IntegrityIndicator`, `ProtectedActionButton`,
`WhyStatePanel`, `EvidenceDrawer`, `AuthorityBoundaryPanel`,
`AvailabilityLadder`. SEA Forge-owned **domain organisms**:
`ReadinessConsole`, `CaseCreationWorkbench`, `PlanPreflightPanel`,
`CaseHorizonBoard`, `ExecutionConsole`, `AgentTaskConsole`,
`ApprovalDecisionPanel`, `CriterionSettlementMatrix`,
`CapabilityPromotionPanel`, `IntegrityInspector`.

Every substantial operational component can reveal: typed identity, current
state, state reason, source + freshness, evidence/provenance, next lawful
action, disabled-action reason.

## Shell and page families

App shell: compact sidebar (Readiness, Thoth, Assets, Domain Models, Cases,
Inbox, Operations, Evidence, Memory, Capabilities, Artifacts, Federation,
Administration), governed-focus main region, right-hand evidence drawer
(Why / Evidence / Provenance / Record). Case-scoped nav: Overview, Horizon,
Timeline, Plan, Runs, Agent Tasks, Settlement, Evidence. Route table and
guards G1–G9: view-flow spec §7.3.

## Swizzling policy

```text
compose → theme → supported override → swizzle only when required
```

Every swizzle recorded in a `SWIZZLED.md` beside the component: Astryx
package version, original component, reason for ownership, behavior changed,
accessibility tests, upgrade responsibility. **Never patch `node_modules`.**

## UX laws (per screen)

One current governed focus; one dominant lawful action; exact state-domain
labels (never generic "success"); execution and settlement shown separately;
agent dialogue / termination / settlement shown separately; source and
freshness visible or one action away; disabled actions visible with reason
and repair path; dense rows/tables/matrices/drawers over card grids;
keyboard access with visible focus; reduced motion; non-color status cues;
responsive down to inspection-and-corrective-action on phones (never
compressed desktop authoring).

Do not: build generic giant-card dashboards; build a consumer-chat-first
agent interface; fabricate pricing, confidence, authority, evidence,
commands, or options; treat OpenDesign data as backend truth; create routes
for component snippets because the generated manifest calls them screens;
copy prototype JS state management; let Astryx stock patterns override
SEA Forge domain meaning.

## Accessibility contract (WCAG AA baseline)

Contrast AA minimum; `:focus-visible` rings; keyboard-only completion of the
primary path; focus return after dialogs/detours; table semantics for
matrices; a list alternative for every graph; reduced-motion support; 200%
zoom without state or action loss; live announcements only for meaningful
governed transitions (throttled). axe-core checks run in component tests and
Playwright flows.

## Visual-regression expectations

Storybook stories are the fidelity fixtures: every semantic component state
(each authority/execution/settlement/integrity variant) and each organism's
reference state gets a story.

For every UI change with a checked-in reference:

1. Serve the reference and implementation in real browsers.
2. Use the same viewport for both; include the reference's desktop width and
   each responsive breakpoint the change can affect.
3. Capture a reference screenshot and an implementation screenshot.
4. Compare computed geometry for the application shell, fixed bars, navigation,
   governed focus, content columns, drawers, and responsive overlays.
   Confirm the intended layout selectors are active in normal and
   reduced-motion modes; source presence alone does not prove a rule escaped
   an accidental media-query boundary.
5. Compare type roles, spacing density, semantic colors, borders, radii, and
   visible state/action hierarchy.
6. Add a stable regression check: source-projection drift when CSS is copied,
   DOM/region assertions, computed-geometry assertions, or reviewed visual
   snapshots as appropriate.
7. Check console errors, axe results, keyboard order, focus visibility, and
   200% zoom before calling the surface faithful.

For Readiness and Operate-route layout, compare against the rendered
`ui_kits/app/index.html` plus its `styles.css`; inspect `app.js` only to
understand the static kit's view hierarchy and interaction feel. Never copy its
state management or display-only records into production.
