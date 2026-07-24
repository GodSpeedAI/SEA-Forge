# SEA Forge Applied App Kit

This kit demonstrates the source-backed Readiness Overview in the shared SEA Forge application shell. It is a product UI reference, not a static marketing mockup and not a backend implementation.

## Source Basis

The layout and behavior come from `MOCKUP-BRIEF.md`, the primary-path Readiness wireframe, the atomic Readiness Console organism, the governed UX epic, and the frontend source/freshness and disabled-state contracts.

## Structure

- `index.html` — semantic shell and Readiness regions.
- `styles.css` — responsive layout and component styling bound to `../../colors_and_type.css`.
- `components.js` — reusable drawer-panel render functions.
- `components/` — focused semantic HTML references for the governed focus, readiness matrix, and evidence drawer.
- `app.js` — operation selection, protected-action feedback, evidence tabs, drawer state, navigation focus, and keyboard shortcuts.

## Components

- `components/GovernedFocusHeader.html` composes typed state, source freshness, explanation, and actions.
- `components/ReadinessConditionTable.html` renders compact source-backed conditions with non-color state labels.
- `components/EvidenceDrawer.html` provides the Why, Evidence, Provenance, and Record projections.

`components.js` keeps the standalone preview dependency-free. The HTML component files preserve the same contracts for React/Tauri implementation.

## Usage

1. Open `index.html` directly, or copy the components into a React surface.
2. Switch the intended operation and inspect the readiness state and case-creation reason.
3. Run checks and confirm the interface reports acceptance for processing before restoring authoritative state.
4. Open Why, Evidence, Provenance, and Record.
5. Verify keyboard shortcuts `R`, `I`, `B`, `E`, `C`, and `Escape`.
6. Resize through desktop, compact desktop, tablet, and phone layouts.

## Reuse

Import `../../colors_and_type.css` first. Preserve semantic state labels and component boundaries when moving the kit into React or Tauri. Replace the reference display data with typed, source-backed query models; do not turn the static demonstration into a fake backend record.

## Design Notes

- The Readiness surface is operation-sensitive, not one universal health score.
- Degradation remains usable and names its limitation.
- Source freshness is adjacent to the governed state.
- The right drawer provides depth without turning logs into the primary explanation.
- The phone layout supports inspection and corrective action rather than compressing desktop authoring.
- This is an App workbench with a compact Sidebar; it deliberately has no consumer-style ChatArea.
