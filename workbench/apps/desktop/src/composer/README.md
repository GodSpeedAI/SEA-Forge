# Composer

## Overview
Persistent, context-bound command composer for the spatial Workbench shell. The composer is mounted by `AppShell` (see `/home/sprime01/projects/sea-rs/workbench/apps/desktop/src/shell/AppShell.tsx`) and renders the affordances projected for the currently active surface. Styling lives in the co-located CSS module `Composer.module.css`.

## Interface
- `ComposerProps` — `surfaceId` plus the affordance list to render [Composer.tsx:ComposerProps]
- `Composer({ surfaceId, affordances })` — the React component [Composer.tsx:Composer]

## Invariants
- ["None"]

## Side Effects
- ["None"]

## Verification
- Run: `bun run test` (from workbench/)
