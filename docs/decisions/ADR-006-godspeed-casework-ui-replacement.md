# ADR-006: Replace the SEA Forge and Gauntlet user interfaces

## Status

Accepted direction for the casework-environment specification. Removal occurs
only after the plan's parity and conformance gates pass.

## Date

2026-09-19

## Context

SEA Forge currently has a React renderer hosted by a Rust/Tauri desktop app.
Gauntlet is a separate Rust executor with its own TUI. The requested GodSpeed
environment uses a Go operational front end and a React cognitive interface.
The operator explicitly wants both existing user interfaces removed, rather
than merely labeled deprecated. The SEA Forge kernel/server and Gauntlet
executor remain the governed and execution substrate.

The current SEA Forge checkout contains no Go module. Its typed SFWP boundary
and Workbench are real; the exact Go-facing methods needed for leases, history,
and artifact persistence must be inventoried. The Gauntlet repository is a
sibling checkout. No GitHub casework adapter, CopilotKit integration, Open MCT
checkout, or OpenMontage checkout is assumed to exist here.

## Decision

The replacement Go application is built at `apps/godspeed-casework-go`; the
replacement React application is built at `apps/godspeed-cognitive-ui`.
React uses versioned HTTP JSON intents/snapshots and cursor-based server-sent
projection events through typed Go application APIs. Go coordinates work
through typed SEA Forge SFWP and Gauntlet adapters. SEA Forge alone grants
authority and accepts settlement; Gauntlet supplies execution observations.
Neither a Go lease nor a UI projection is case truth.
Read-only Gauntlet diagnostics may use a typed Go adapter; consequential
controls still cross SEA Forge authority.

Both existing interfaces run only during the migration period. The plan first
proves source-backed workflow parity, real governed integration, recovery,
and a cutover procedure. It then removes the SEA Forge Tauri/Rust GUI and
Gauntlet TUI code and supported build entry points. Any assertions protecting
the retained server/executor contract are ported before UI-specific tests go.
The post-removal build must pass all required gates and cold confirmation.
The old binaries are not shipped as fallback user interfaces.
The Rust-generated SFWP schema and its committed-schema drift assertion move
to the replacement client contract location before Workbench contract files
are removed; neither is discarded to make the old GUI deletion pass.

ADR-004 remains a record of the former Workbench stack; it is superseded for
the supported human interface only when the T14 removal gate settles. New
dependencies, SFWP methods, and persisted contracts still require the
repository's separate review and approval before implementation.

## Dated addendum — 2026-09-20

> **Superseded 2026-09-22 (operator decision):** R3F is no longer required and the donor list no longer applies. See the 2026-09-22 note below and `.agents/specs/cognitive-environment/GodSpeed_Cognitive_Environment_DESIGN.md` §0.1.

The implementation target is narrowed from renderer and donor optionality to a
real-time cognitive scene. React Three Fiber / Three.js / Drei is therefore the
selected and required primary scene renderer, while the UI core remains
renderer-independent behind a GodSpeed-owned scene adapter and DOM remains the
precision layer for documents, code, tables, and forms.

The selected mechanic donors are `jshor/tycho`, `pmndrs/racing-game`,
`rmarchet/blackhole-ts`, `frag2win/BLACK-HOLE`, and
`AmitDigga/threejs-galaxy-shader`; they must be inspected under `.tmp/donor/`
and frozen with revision, license, provenance, extraction, and target-seam
records. They supply spatial, game-loop, camera, Core/lensing, and restrained
particulate mechanics. They remain implementation inputs rather than product
dependencies, and donor ontology does not enter GodSpeed contracts.

This addendum does not alter the Go/application boundary, SEA Forge authority,
Gauntlet observation role, settlement requirements, or migration gates. The
renderer choice changes how the approved experience is realized; it does not
grant authority to the scene or local UI state.

## Dated addendum — 2026-09-22

> **Superseded 2026-09-22 (operator decision):** the Gargantua CORE renderer and the R3F host were removed. The UI direction, rendering approach and first milestone are defined in `.agents/specs/cognitive-environment/GodSpeed_Cognitive_Environment_DESIGN.md` §0.1, which overrides this addendum on rendering. The React + Go split, SFWP boundary, and authority rules in this ADR still stand.

The persistent CORE renderer is settled as a GodSpeed-owned **Three.js renderer
that owns its own WebGL context and canvas**, not an R3F scene graph. The
Gargantua donor implementation, previously extracted under the Workbench
(`workbench/apps/desktop/src/core/`), is relocated to
`apps/godspeed-cognitive-ui/src/host/gargantua/` and becomes the canonical CORE.
React Three Fiber / Three.js / Drei remains the required primary scene renderer
for objects, relationships, spatial projections, and interaction geometry, whose
canvas is composited with alpha **above** the CORE layer so CORE remains visible
through it.

`renderer_boundary` in spec v0.2.4 now states this composition explicitly. The
decision does not relax any authority, settlement, migration, or donor
boundary: CORE stays visual intent only, the UI core remains
renderer-independent behind the same GodSpeed-owned scene-adapter seam, and the
CORE renderer remains independently replaceable. The existing R3F Core
implementation (`src/host/scene/CoreObject.tsx`,
`src/host/scene/AccretionSwirl.tsx`) is retired once the relocated CORE renderer
is integrated and accepted; it is not maintained as a parallel or fallback CORE.

Donor provenance is recorded at
`apps/godspeed-cognitive-ui/src/host/gargantua/PROVENANCE.md`, including the
golden reference `sha256:9cfdc399…c93cd` and the byte-parity proof that the move
introduced no donor change. The move added no dependency: the app already
depended on `three@^0.186.0`.

T14 removal of the superseded interfaces is unchanged and remains gated on
replacement parity and real integration; the relocation of donor code into the
replacement app is not itself a removal step.

## Consequences

- The new Go and React applications need their own scoped instructions,
  canonical `just` gates, typed contract/version policy, and real integration
  tests.
- SEA Forge and Gauntlet keep their existing authority and execution gates.
- The T14 removal check must prove the old UI code and supported launch paths
  are absent from both repositories while retained engines still work.
- If parity, authority, history, or recovery fails, removal and final
  settlement are blocked. A hidden or disabled legacy UI does not satisfy
  the decision.

## Alternatives considered

Extending the Tauri host would retain the interface the operator wants
removed. Replacing Gauntlet's executor along with its TUI would expand scope
and discard useful execution substrate. Giving Go local case authority would
duplicate SEA Forge's governed boundary. These are rejected.
