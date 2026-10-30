# sea-forge-ui-components/src/stories

## Overview

Storybook stories for the nine governed-UI components exported from `../index.ts`. This is development/documentation tooling, not production renderer code: it exists for visual review and manual component exploration via the package's `storybook` / `build-storybook` scripts in `../../package.json`. The single file `components.stories.tsx` defines one `Meta` per component family and named `StoryObj` exports exercising their variant/state matrices.

## Interface

- `pillMeta: Meta<typeof GovernedStatusPill>` (default export, title `Semantic/GovernedStatusPill`) with stories `AllPillVariants` (all 13 `GovernedStatusVariant` inputs) [components.stories.tsx:AllPillVariants]
- `DualStateSample: StoryObj<typeof DualStateIndicator>` [components.stories.tsx:DualStateSample]
- `SourceFreshnessVariants: StoryObj<typeof SourceFreshnessBadge>` [components.stories.tsx:SourceFreshnessVariants]
- `IntegrityVariants: StoryObj<typeof IntegrityIndicator>` [components.stories.tsx:IntegrityVariants]
- `ProtectedActionStates: StoryObj<typeof ProtectedActionButton>` [components.stories.tsx:ProtectedActionStates]
- `WhyStateSample: StoryObj<typeof WhyStatePanel>` [components.stories.tsx:WhyStateSample]
- `EvidenceDrawerSample: StoryObj<typeof EvidenceDrawer>` [components.stories.tsx:EvidenceDrawerSample]
- `AuthorityBoundarySample: StoryObj<typeof AuthorityBoundaryPanel>` [components.stories.tsx:AuthorityBoundarySample]
- `AvailabilityLadderSample: StoryObj<typeof AvailabilityLadder>` [components.stories.tsx:AvailabilityLadderSample]

## Invariants

- `["None"]` — stories are inert UI fixtures; no behavioral contract is enforced here. They must keep compiling against the component props exported from `../index.ts`.

## Side Effects

- `["None"]` — renders components inside the Storybook iframe only; no network, storage, or process-level effects.

## Verification

- Run: `bun run build-storybook` (from `workbench/packages/sea-forge-ui-components/`; emits `../../storybook-static/`, a build-output zone). Interactive: `bun run storybook`. Stories are excluded from the vitest suite (`bun run test`).
