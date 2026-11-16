# Core Chrome

## Overview
Overlay UI chrome around the persistent CORE viewport: identity context, live context readout, interaction hints, and the fatal render failure surface. All components are presentational; state comes from the shell and renderer.

## Interface
- `ContextIdentityProps` and `ContextIdentity` — actor/cell identity badge [ContextIdentity.tsx:ContextIdentity]
- `ContextReadoutProps` and `ContextReadout` — focus/surface/settlement/activity readout rows [ContextReadout.tsx:ContextReadout]
- `InteractionHints` — input hint overlay [InteractionHints.tsx:InteractionHints]
- `RenderFailureSurface` — shown on fatal renderer failure; receives `CoreFatalInfo` from `../CoreRenderer` [RenderFailureSurface.tsx:RenderFailureSurface]

## Invariants
- ["None"]

## Side Effects
- ["None"]

## Verification
- Run: `bun run test` (from workbench/)
