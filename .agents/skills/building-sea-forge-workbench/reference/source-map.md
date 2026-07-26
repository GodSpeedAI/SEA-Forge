# Source map — `.agents/specs/frontend/`

Every file in the frontend specification package, its purpose, when to read
it, and its status. **Normative** files constrain implementation; **reference**
files inform it. The package is preserved source evidence — do not edit it
while implementing product code; record new findings here instead.

## Reading order for full-package orientation

README → context-provenance → DESIGN → DESIGN-spec-mapping → MOCKUP-BRIEF →
ux-epic → atomic-breakdown → view-flow → wireframes → frontend-architecture →
api-spec → api-method-catalog → build-plant-prompt → ui_kits.
Single-slice tasks should instead load only the rows relevant to the slice.

## File inventory

| File | Purpose | Read when | Status |
|---|---|---|---|
| `README.md` | Package overview, contents, core laws | Orienting in the package | Reference |
| `context-provenance.md` | Evidence inventory, derivation decisions, asset boundary | Before claiming any asset/behavior is source-authoritative | Normative for provenance |
| `DESIGN.md` | Canonical visual/interaction grammar, tokens, motion, voice, anti-patterns | Any visual or interaction work | **Normative (design)** |
| `DESIGN-spec-mapping.md` | Traceability from design system to SEA Forge views and page families | Mapping a screen to design rules | Reference |
| `MOCKUP-BRIEF.md` | Required primary-path screens, shell, failure variants, copy constraints | Building a primary-path screen | Normative (screen scope) |
| `sea-forge-governed-workbench-ux-epic-v0.1.md` | Actors, journeys, cross-cutting invariants, non-goals | Understanding who does what and why | Reference |
| `sea-forge-gui-atomic-design-breakdown-v0.1.md` | Foundations/atoms/molecules/organisms/templates/pages | Component tiering and naming | Normative (component inventory) |
| `sea-forge-gui-view-flow-transition-spec-v0.1.md` | Route architecture, guards G1–G9, state machines, transitions | Routing, guards, navigation flows | **Normative (routes/flows)** |
| `sea-forge-primary-path-screen-wireframe-interaction-spec-v0.1.md` | Region hierarchy, states, keyboard contracts, acceptance criteria per screen | Implementing a specific primary-path screen | **Normative (screen behavior)** |
| `sea-forge-workbench-frontend-architecture-component-contract-v0.1.md` | Frontend layers, state ownership, route modules, component interfaces, test architecture | Architecture and state-ownership questions | **Normative (frontend architecture)** |
| `sea-forge-workbench-api-spec-v0.1.md` | SFWP: framing, envelopes, preconditions, idempotency, events, errors | Any API/bridge work | **Normative (target semantics)** — repository grounding required; it is a target, not an inventory |
| `sea-forge-workbench-api-method-catalog-v0.1.yaml` | Full target method catalog with interaction class, authority surface, mutation, response | Mapping/naming any method | Normative (target catalog, `status: target-unmapped`) |
| `build-plant-prompt.md` | The prompt that produced the unified GUI spec; inspection sequences | Historical context only | Reference (superseded by this skill) |
| `colors_and_type.css` | Semantic token stylesheet (surfaces, fg, authority/execution/dialogue state colors, type, spacing, radius, depth, motion) | Token source of truth for theming | **Normative (tokens)** |
| `SKILL.md` | Design-system skill (`sea-forge-workbench-design`) | Design QA of produced surfaces | Normative (design application). **Not** the implementation skill; never overwrite or move it |
| `ui_kits/DESIGN-HANDOFF.md` | OpenDesign handoff notes | Understanding kit provenance | Reference |
| `ui_kits/DESIGN-MANIFEST.json` | Generated OpenDesign manifest | Almost never — see caveats | Generated; do not hand-edit |
| `ui_kits/app/README.md` | Kit reuse guide | Extracting kit patterns | Reference |
| `ui_kits/app/index.html` | Applied **Readiness** workbench and shared-shell reference (not a generic launcher) | Shell and Readiness visual fidelity | Reference (visual only) |
| `ui_kits/app/styles.css` | Applied shell, Readiness, and Operate-route styling bound to `../../colors_and_type.css` | Visual styling detail; see media-query caveat below | Reference (visual only) |
| `ui_kits/app/app.js` | Static Operate-route view scenarios plus prototype interactions | View hierarchy and interaction feel only | Reference — **never** copy its state management or display records into production |
| `ui_kits/app/components.js` | Dependency-free HTML render helpers for the static preview | Rarely | Reference (prototype plumbing) |
| `ui_kits/app/components/GovernedFocusHeader.html` | Semantic component reference | Building the governed focus header | Reference (component contract) |
| `ui_kits/app/components/ReadinessConditionTable.html` | Semantic component reference | Building the readiness matrix | Reference (component contract) |
| `ui_kits/app/components/EvidenceDrawer.html` | Semantic component reference | Building the evidence drawer | Reference (component contract) |

## Corrections applied 2026-07-24 (do not re-introduce)

These stale references existed in the package and were fixed in place; the
list is kept so future agents recognize old copies or diffs:

1. **`css.txt` renamed to `colors_and_type.css`.** The token stylesheet was
   delivered as `css.txt` while every document (README, SKILL, kit README,
   `ui_kits/app/index.html`'s `<link>`) referenced `colors_and_type.css`. The
   file was renamed to match the documented name; all references now resolve.
2. **`preview/` never existed.** README and SKILL referenced eight
   `preview/*.html` review cards and `preview/manifest.json`; none were
   carried into the package. References removed; the surviving equivalents
   are `ui_kits/app/components/*.html`.
3. **`context/provenance.md` → `context-provenance.md`** (README, SKILL ×2,
   DESIGN.md). The `context/` directory does not exist.
4. **`context/source-context.md` → `ui_kits/DESIGN-HANDOFF.md`** (README) —
   the actual handoff record.
5. **`assets/README.md`, `build/`, `fonts/` never existed.** README rewritten
   to state the asset boundary directly (no approved assets; none fabricated).

## Remaining known caveats (documented, not editable)

- **`ui_kits/DESIGN-MANIFEST.json` misclassifies components as screens.** Its
  `screens` array lists `EvidenceDrawer.html`, `GovernedFocusHeader.html`,
  and `ReadinessConditionTable.html` with `role: product-screen` and
  `index.html` as `launcher-overview`. In truth the three are component
  snippets and `index.html` is the applied **Readiness** reference surface.
  The manifest is generated (`schema: open-design.design-manifest.v1`), so it
  is documented here rather than hand-edited. **Never derive routes from it.**
- **Prototype JS is not architecture.** `app.js` mutates DOM state directly
  and simulates transitions locally; production state ownership is defined by
  the frontend architecture contract §6 (state classes) and this skill.
- **Operate-route CSS is trapped inside reduced-motion media.** In the
  checked-in `styles.css`, the route-variant block follows an unclosed
  `prefers-reduced-motion` rule. Treat its selectors as design intent, but
  verify their production projection with normal-motion computed styles;
  do not copy the accidental media-query boundary.
- **The API spec/catalog are self-declared `target-unmapped`.** Every method
  must be grounded per `api-and-event-contracts.md` before implementation.
- **No fonts ship with the package.** Inter and JetBrains Mono stacks resolve
  via local/system fallbacks; bundling real fonts is a packaging-milestone
  decision.
