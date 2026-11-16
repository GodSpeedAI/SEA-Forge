# @sea-forge/ui-components (sea-forge-ui-components)

## Overview

Bun workspace package (`@sea-forge/ui-components`, private, `"type": "module"`) providing the Workbench renderer's governed-UI component library: nine React 19 presentational components that visualize authority boundaries, execution vs. settlement state, evidence/provenance, record integrity, and source freshness, built on `@astryxdesign/core` primitives and themed via `@sea-forge/astryx-theme` and `@sea-forge/ui-tokens` [package.json:dependencies]. Entry point is `src/index.ts` (`main`, `types`, and `exports["."]`; `exports["./*.css"]` additionally exposes each CSS Module directly) [package.json:exports].

Tooling: TypeScript build check (`tsc -b --noEmit`), vitest + Testing Library suite with jsdom and axe-core, and Storybook 8 [package.json:scripts, package.json:devDependencies].

## Interface

- [./src/README.md] — contract for `src/` (production source): the `index.ts` barrel and components `GovernedStatusPill`, `DualStateIndicator`, `SourceFreshnessBadge`, `IntegrityIndicator`, `ProtectedActionButton`, `WhyStatePanel`, `EvidenceDrawer`, `AuthorityBoundaryPanel`, `AvailabilityLadder`, plus their CSS Modules.
- [./src/stories/README.md] — contract for `src/stories/` (Storybook stories; development-only).
- Package scripts: `check` (tsc), `test` (vitest run), `storybook`, `build-storybook` [package.json:scripts].
- `storybook-static/` — generated Storybook build output zone (emitted by `build-storybook` with `-o storybook-static`); not source, do not hand-edit.
- `vitest.config.ts` / `vitest.setup.ts` — vitest configuration for the package test suite.
- `tsconfig.json` — TypeScript project configuration.

## Invariants

- Status unions are closed and degrade safely: unknown status strings fall back to `"unknown"` in `GovernedStatusPill` and to `unverified`/`stale` display in `IntegrityIndicator`/`SourceFreshnessBadge` [src/GovernedStatusPill.tsx:KNOWN_VARIANTS, src/IntegrityIndicator.tsx:statusClasses, src/SourceFreshnessBadge.tsx:modeLabels].
- Governed actions are fire-walled in the UI: `ProtectedActionButton` only executes when allowed, standing, preflight, and loading conditions all permit, and confirmation-gated actions require an explicit two-step confirm [src/ProtectedActionButton.tsx:isDisabled, src/ProtectedActionButton.tsx:handlePrimaryClick].
- Restricted evidence (`disclosureStatus === "restricted"`) never renders its raw payload; a G7 Disclosure notice is shown instead [src/EvidenceDrawer.tsx:disclosure].
- Displayed state is non-authoritative in the renderer [src/EvidenceDrawer.tsx:Governing-condition-section].

## Side Effects

- Component-level only: `EvidenceDrawer` adds a window `keydown` listener for Escape-to-close; `ProtectedActionButton` awaits the caller-supplied `onClick()` promise. No package-level I/O; the build scripts write `storybook-static/` [src/EvidenceDrawer.tsx:useEffect].

## Verification

- Run: `bun run test` (from this directory; `vitest run`). Type check: `bun run check`. Repo gate: `just workbench-check`. Workspace-wide tests: `bun run test` from `workbench/`.
