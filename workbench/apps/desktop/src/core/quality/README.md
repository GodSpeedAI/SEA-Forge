# Adaptive Quality

## Overview
Frame-time-driven quality controller for the CORE renderer. `AdaptiveQuality` tracks an exponential moving average of frame duration and moves an index into `QUALITY_LADDER` up or down, with hysteresis and a cooldown, so the render pipeline scales resolution/passes to the device.

## Interface
- `QualityLevel` — one rung of the ladder [AdaptiveQuality.ts:QualityLevel]
- `QUALITY_LADDER` — ordered quality levels [AdaptiveQuality.ts:QUALITY_LADDER]
- `INITIAL_QUALITY_INDEX = 1` — starting rung [AdaptiveQuality.ts:INITIAL_QUALITY_INDEX]
- `AdaptiveQuality` — controller class (frame-sample input, quality-change notification) [AdaptiveQuality.ts:AdaptiveQuality]

## Invariants
- Downgrade only when the smoothed frame time exceeds `DOWNGRADE_MS = 34`, upgrade only below `UPGRADE_MS = 15`, separated by `CHANGE_COOLDOWN_MS = 2200` [AdaptiveQuality.ts:DOWNGRADE_MS]
- Smoothing uses `AVG_ALPHA = 0.92` / `AVG_BETA = 0.08` from `INITIAL_AVG_MS = 20` [AdaptiveQuality.ts:AVG_ALPHA]

## Side Effects
- ["None"]

## Verification
- Run: `bun run test` (from workbench/)
