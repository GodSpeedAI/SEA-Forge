---
name: sea-forge-workbench-design
description: Apply the SEA Forge governed workbench design system to operational product surfaces, review cards, and prototypes.
user-invocable: true
---

# SEA Forge Workbench Design

Use this package when designing or reviewing SEA Forge interfaces.

## What is inside

- `DESIGN.md` — canonical product, visual, interaction, motion, voice, and anti-pattern rules.
- `colors_and_type.css` — semantic color, type, spacing, radius, depth, and motion tokens.
- `ui_kits/app/components/` — focused semantic component references (governed focus, readiness matrix, evidence drawer).
- `ui_kits/app/` — an interactive Readiness Console reference.
- Preserved source specifications in this directory.

## Source context

The package is grounded in the SEA Forge design, mockup brief, atomic breakdown, UX epic, view-flow specification, primary-path wireframes, semantic mapping, and frontend architecture contract. Read `context-provenance.md` before claiming that an asset or behavior is source-authoritative.

## When to use

Use this skill for SEA Forge workbench pages, governed operational components, evidence and provenance views, readiness and integrity states, cases and horizons, command runs, agent tasks, settlement review, capability inspection, and design-system QA.

Do not use it to create a marketing site, consumer chat interface, or generic analytics dashboard.

## How to use

1. Read `DESIGN.md`.
2. Read `DESIGN-spec-mapping.md` for screen semantics.
3. Read `context-provenance.md` before claiming a source asset or interaction is canonical.
4. Import `colors_and_type.css` before feature styles.
5. Identify the current governed focus and name every state by domain.
6. Keep execution and settlement separate; for agent tasks also separate dialogue and termination.
7. Expose the source, freshness, reason, evidence, and next lawful action.
8. Render exactly one dominant protected action.
9. Keep unavailable actions visible with a reason and repair path.
10. Add keyboard access, visible focus, reduced motion, and non-color state cues.

## Design-system highlights

- Dark, high-density, desktop-native governed workbench.
- Exactly one current governed focus and one dominant lawful action.
- Typed labels keep execution, settlement, dialogue, termination, integrity, and capability separate.
- Source, freshness, evidence, disabled reasons, and repair paths remain within one action.
- Compact rows, panels, matrices, drawers, and keyboard-first interaction.

Use semantic tokens from `colors_and_type.css`. Do not introduce raw colors, generic status names, extra pigments, radii above 8px, marketing display type, gradients, or decorative glow.

## Component contract

Every substantial operational component should be able to reveal typed identity, current state, state reason, source and freshness, evidence or provenance, next lawful action, and disabled-action reason.

Preferred primitives:

- Governed Focus Header
- Governed Status Pill
- Dual State Indicator
- Source Freshness Badge
- Protected Action Button
- Why State Panel
- Evidence Drawer
- Criterion Settlement Matrix

## Completion check

- The center is the current governed focus.
- Exactly one action is dominant.
- State domains remain distinct in text and structure.
- Essential small text does not use `--fg-tertiary`.
- The interface works by keyboard and at 200% zoom.
- No source asset, command, score, criterion, or evidence has been invented.
