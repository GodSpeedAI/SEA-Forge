# Surfaces

## Overview
The spatial surfaces of the Workbench: route-level compositions that frame the existing operate pages inside the `Surface` primitive from `src/spatial/primitives` and wire focus/time state. Every surface is a thin framing module — the wrapped page owns its governed reads and writes; no domain behavior is fabricated here.

## Subdirectories & Boundaries
- artifacts: [./artifacts/README.md] - Artifacts index composed as a spatial surface (`ArtifactsSurface`).
- case: [./case/README.md] - Case-horizon capability with spatial framing and focus wiring (`CaseSurface`).
- case-design: [./case-design/README.md] - New-work authoring over the case-creation workbench (`CaseDesignSurface`).
- causal: [./causal/README.md] - Evidence index framed as meaningful connections (`CausalSurface`).
- execution: [./execution/README.md] - Governed delegation work over the delegation workbench (`ExecutionSurface`).
- home: [./home/README.md] - The canonical CORE-centered orientation state (`HomeSurface`).
- judgment: [./judgment/README.md] - Bounded human judgment over pending approvals (`JudgmentSurface`).
- semantic-beat: [./semantic-beat/README.md] - The Thoth workspace composed spatially (`SemanticBeatSurface`).
- temporal: [./temporal/README.md] - Historical/developmental inspection over the operations monitor (`TemporalSurface`).
