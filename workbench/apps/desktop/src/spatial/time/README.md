# Time / Version Controller

## Overview
The time/version boundary: inspecting the same object or system through its developmental or historical states. Positions are opaque cursors (ledger entry ULID, event cursor, run id) minted by the kernel's event/ledger surfaces. A single `timeVersionController` instance lives for the application lifetime.

## Interface
- `TimePosition` — opaque `cursor` + `label` [TimeVersionController.ts:TimePosition]
- `TimeVersionController` — pin/returnToLive state with listener subscription [TimeVersionController.ts:TimeVersionController]
- `timeVersionController` — application-lifetime singleton [TimeVersionController.ts:timeVersionController]

## Invariants
- Positions are kernel-minted opaque cursors, never fabricated client-side [TimeVersionController.ts:TimePosition]
- `isLive()` is true exactly when no historical position is pinned (`position === null`) [TimeVersionController.ts:isLive]

## Side Effects
- ["None"]

## Verification
- Run: `bun run test` (from workbench/)
