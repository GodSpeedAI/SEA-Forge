# Dev Tuning Panel

## Overview
Development-only overlay for tuning the CORE renderer's visual parameters live. Mounted by the shell behind dev gating (`AppShell` imports `CoreTuningPanel` from here). Sliders emit parameter changes upward to the shell/renderer; styling is in `CoreTuningPanel.module.css`.

## Interface
- `CoreTuningPanelProps` — current `CoreVisualParams`, `fps` reading, and `onChange` callback [CoreTuningPanel.tsx:CoreTuningPanelProps]
- `CoreTuningPanel({ params, fps, onChange })` — the panel component [CoreTuningPanel.tsx:CoreTuningPanel]

## Invariants
- ["None"]

## Side Effects
- ["None"]

## Verification
- Run: `bun run test` (from workbench/)
