# Interactive Explainers

This directory holds standalone HTML explainers for SEA Forge's governed execution lifecycle. Each page is a single file: no CDN, no external fonts, no build step, and no network access. Open it directly with `file://`.

Use these pages when a static diagram is not enough and you want to step through state transitions, evidence, and settlement outcomes.

| Explainer | Reader question | Type | Source anchors |
|---|---|---|---|
| [Governed Run Explorer](governed-run-explorer.html) | What actually happens between intent and settlement, and why does a clean exit not mean success? | Interactive stepper and scenario player | [`docs/workflows/cli-run-lifecycle.md`](../workflows/cli-run-lifecycle.md), [`docs/subsystems/authority-fabric.md`](../subsystems/authority-fabric.md), [`docs/subsystems/settlement-evidence.md`](../subsystems/settlement-evidence.md) |

These explainers are documentation, not runtime code. They may link to Markdown pages and source files, but they never call the kernel or mutate a cell.
