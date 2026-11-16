# Focus Controller

## Overview
Global focus state for the spatial shell: what is currently in focus (CORE, an object, or an anchor) and the animated transition between focus targets. A single `focusController` instance lives for the application lifetime and the shell, surfaces, and CORE viewport subscribe to it.

## Interface
- `FocusTarget` — `{ kind: "core" } | { kind: "object"; objectId } | { kind: "anchor" }` [FocusController.ts:FocusTarget]
- `FocusController` — focus state + listener set (`subscribe`, set/clear focus) [FocusController.ts:FocusController]
- `focusController` — application-lifetime singleton [FocusController.ts:focusController]
- `FocusTransitionSpec`, `FOCUS_TRANSITION_MS = 1200`, `describeTransition(from, to)`, `easeFocusTravel(t)` — transition timing and easing [FocusTransition.ts:FOCUS_TRANSITION_MS]

## Invariants
- Focus targets are restricted to `core`, `object`, and `anchor` kinds [FocusController.ts:FocusTarget]

## Side Effects
- ["None"]

## Verification
- Run: `bun run test` (from workbench/)
