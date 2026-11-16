# sea-forge-ui-components/src

## Overview

Source of the `@sea-forge/ui-components` package: React 19 presentational components that render SEA Forge's governed-kernel concepts (authority, execution vs. settlement state, evidence, integrity, source freshness) in the Workbench renderer. `index.ts` is the package entry point (`main`/`types`/`exports["."]` in `../package.json`) and re-exports nine components, each with a co-located CSS Module. Components are built on `@astryxdesign/core` primitives and the `@sea-forge/astryx-theme`/`@sea-forge/ui-tokens` token sets.

The `stories/` child directory contains Storybook stories (not production code; documented separately in [./stories/README.md]).

## Interface

- Barrel: `export * from` each of the nine component modules [index.ts].
- `GovernedStatusPill.tsx`: `type GovernedStatusVariant` (13 variants: `ready`, `degraded`, `blocked`, `integrity_halted`, `running`, `finished`, `cancelled`, `failed`, `pending`, `committed`, `rejected`, `unsupported`, `unknown`), `interface GovernedStatusPillProps`, `function GovernedStatusPill` [GovernedStatusPill.tsx].
- `DualStateIndicator.tsx`: `interface DualStateIndicatorProps { executionState, settlementState, className }`, `function DualStateIndicator` [DualStateIndicator.tsx].
- `SourceFreshnessBadge.tsx`: `type FreshnessMode` (`live` | `cached` | `stale` | `offline` | `checking`), `interface SourceFreshnessBadgeProps`, `function SourceFreshnessBadge` [SourceFreshnessBadge.tsx].
- `IntegrityIndicator.tsx`: `type IntegrityStatus` (`verified` | `compromised` | `checking` | `unverified`), `interface IntegrityIndicatorProps`, `function IntegrityIndicator` [IntegrityIndicator.tsx].
- `ProtectedActionButton.tsx`: `interface ProtectedActionButtonProps` (incl. `isAllowed`, `hasStanding`, `preflightState: "idle" | "evaluating" | "passed" | "failed"`, `requiresConfirmation`), `function ProtectedActionButton` [ProtectedActionButton.tsx].
- `WhyStatePanel.tsx`: `interface StateCondition { id, name, status: "pass" | "fail" | "warn", explanation }`, `interface WhyStatePanelProps`, `function WhyStatePanel` [WhyStatePanel.tsx].
- `EvidenceDrawer.tsx`: `interface EvidenceRecord { id, kind, ledgerUlid?, digest?, disclosureStatus: "permitted" | "restricted" | "gated", rawPayload? }`, `interface EvidenceDrawerProps`, `function EvidenceDrawer` [EvidenceDrawer.tsx].
- `AuthorityBoundaryPanel.tsx`: `interface AuthorityBoundaryPanelProps` (actor/role/sponsor/policy-bundle display, `allowedBoundaries`) [AuthorityBoundaryPanel.tsx].
- `AvailabilityLadder.tsx`: `interface CapabilityLevel`, `interface AvailabilityLadderProps`, `export const DEFAULT_LEVELS: CapabilityLevel[]` (four levels: Local Kernel Execution, Local SFWP Transport, External Agent Delegation, Federated Peer Settlement, each with a `spendableProofPath`), `function AvailabilityLadder` [AvailabilityLadder.tsx].
- CSS Modules: every component has a sibling `<Component>.module.css` exposed via the package export map (`"./*.css": "./src/*.css"` in `../package.json`).
- Child subdirectory contract: [./stories/README.md].

## Invariants

- Unknown status strings degrade safely: `GovernedStatusPill` resolves any variant not in `KNOWN_VARIANTS` to `"unknown"` and renders with the `unknown` style class [GovernedStatusPill.tsx:KNOWN_VARIANTS].
- A protected action may only fire when all of `isAllowed`, `hasStanding`, `preflightState !== "failed"`, not `isLoading`, and not mid-execution hold; otherwise the button is disabled and a reason is shown (`"Actor lacks required standing for this decision"`, `"Preflight evaluation failed"`, or `"Operation not permitted by policy"` when no explicit `disabledReason` is given) [ProtectedActionButton.tsx:isDisabled, ProtectedActionButton.tsx:resolvedDisabledReason].
- Confirmation-gated actions require a two-step flow: the first click on a `requiresConfirmation` action only enters the confirming state; execution happens on the explicit Confirm click [ProtectedActionButton.tsx:handlePrimaryClick].
- Restricted evidence payloads must not be rendered: when `evidence.disclosureStatus === "restricted"` the raw payload is omitted and a G7 Disclosure notice is shown instead [EvidenceDrawer.tsx:disclosure].
- The renderer's displayed state is explicitly non-authoritative: `EvidenceDrawer` states the displayed state "is derived from SEA Forge records and remains non-authoritative in the renderer" [EvidenceDrawer.tsx:Governing-condition-section].
- `EvidenceDrawer` resets the active tab to `"why"` whenever a different evidence record is selected (`useEffect` on `evidence?.id`) [EvidenceDrawer.tsx:useEffect].
- Component state labels map through closed unions: `IntegrityIndicator.statusClasses` and `SourceFreshnessBadge.modeLabels` define display text for exactly their type's members, falling back to `unverified`/`stale` respectively on out-of-union input [IntegrityIndicator.tsx:statusClasses, SourceFreshnessBadge.tsx:dotClasses].

## Side Effects

- `EvidenceDrawer` registers a `window` `keydown` listener (Escape closes the drawer) while mounted, removing it on unmount; it also handles backdrop clicks to close [EvidenceDrawer.tsx:useEffect, EvidenceDrawer.tsx:backdrop].
- `ProtectedActionButton` maintains local React state (`confirming`, `executing`) and awaits the caller-provided `onClick()` promise, which may perform caller-driven network/RPC work; the component itself makes no network calls [ProtectedActionButton.tsx:handlePrimaryClick].
- All components are pure DOM renderers otherwise; there is no storage, logging, or RPC in this directory.

## Verification

- Run: `bun run test` (from `workbench/packages/sea-forge-ui-components/`; vitest + Testing Library, per `../package.json` scripts). Workspace-wide: `bun run test` from `workbench/`. Type check: `bun run check` (`tsc -b --noEmit`).
