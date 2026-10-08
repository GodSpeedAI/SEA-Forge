# SEA Forge Workbench Design System

A reusable Open Design package for SEA Forge’s governed cognition and execution workbench. The system is dark, compact, evidence-first, keyboard-friendly, and explicit about state, authority, provenance, and lawful next actions.

## Product Overview

SEA Forge is a mission-control application and workbench for governed cognition and execution. Its primary surfaces cover Readiness, Thoth, Assets, Domain Models, Cases, Inbox and Approvals, Operations, Evidence, Memory, Capabilities, Specification Pipelines, Artifacts, Federation, Integrity, and Administration.

Within a case, the workbench supports Overview, Horizon, Timeline, Plan, Command Runs, Agent Tasks, Settlement, and Evidence. The interface keeps approval separate from execution, execution separate from settlement, agent termination separate from settlement, and settlement separate from capability.

## Source Context

The system is derived from the eight substantial SEA Forge design, UX, interaction, mapping, and frontend-contract documents preserved at the project root. `ui_kits/DESIGN-HANDOFF.md` records the Open Design handoff; `context-provenance.md` records the evidence inventory and derivation decisions.

## Reuse Workflow

1. Read `DESIGN.md` for the visual and interaction grammar.
2. Import `colors_and_type.css` before product-specific styles.
3. Review the semantic component references under `ui_kits/app/components/`.
4. Open `ui_kits/app/index.html` for the applied Readiness Console kit.
5. Read `SKILL.md` before generating a new SEA Forge surface.

When implementing in React/Tauri, replace static review data with typed source-backed view models. Preserve state-domain labels, protected-action results, evidence access, keyboard behavior, and the three depth levels.

## Package Contents

```text
DESIGN.md                     Canonical product and design rules
DESIGN-spec-mapping.md        Semantic traceability to SEA Forge views
colors_and_type.css           Reusable tokens and global type foundations
SKILL.md                      Agent-facing application instructions
context-provenance.md         Evidence inventory and derivation notes
ui_kits/DESIGN-HANDOFF.md     Open Design handoff notes
ui_kits/DESIGN-MANIFEST.json  Generated Open Design manifest (see caveat below)
ui_kits/app/index.html        Applied Readiness workbench
ui_kits/app/styles.css        Applied layout and component styling
ui_kits/app/components.js     Reusable HTML render functions
ui_kits/app/components/*.html Source-backed semantic component references
ui_kits/app/app.js            Local interactions and state transitions
ui_kits/app/README.md         UI-kit reuse guide
```

The substantial copied SEA Forge specifications remain in this directory as preserved source evidence.

Earlier revisions of this README referenced `preview/*.html` review cards, `preview/manifest.json`, `assets/README.md`, and a `context/` directory. Those files were not carried into this package; the component references under `ui_kits/app/components/` and `context-provenance.md` are the surviving equivalents.

`ui_kits/DESIGN-MANIFEST.json` is generated tooling output: it classifies the three component HTML snippets as `product-screen` entries. They are component references, not screens; do not derive routes from that classification.

## Assets, Fonts, and Build Artifacts

No source-approved logo, app icon, tray icon, avatar, wordmark, imagery, runtime icon, or font binary was present, and none were fabricated. `build/`, `fonts/`, and `assets/` are omitted because there are no evidence-backed files to place in them.

## Core Laws

- One governed focus and one dominant lawful action per screen.
- Always name the state domain; never show generic success.
- Keep execution, settlement, dialogue, termination, integrity, and capability separate.
- Show source, freshness, evidence, and disabled-action reasons within one action.
- Use compact rows, panels, tables, and drawers rather than card grids.
- Treat agent work as bounded operations, not casual chat.

## Runtime

The previews and UI kit are standalone HTML/CSS/JavaScript with no build step and no network dependencies. They are suitable for Open Design preview, static hosting, and extraction into a React/Tauri implementation.
