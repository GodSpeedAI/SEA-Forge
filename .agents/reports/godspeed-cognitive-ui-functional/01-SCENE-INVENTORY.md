# Cognitive UI: scene inventory and representation mapping (2026-09-23)

This inventory draws on the following sources:
- The 13 mocks in `.agents/specs/cognitive-environment/mocks/`.
- The spec `godspeed.casework-cognitive-environment-spec.yaml` (`interaction_model.dynamic_surfaces`, `reference_journey`, `required_variations`, `required_recovery`).
- Contract spec 04 (`.agents/reports/interface-contracts/04-*`) and the TypeScript contract (`typescript/types.ts`, `client.ts`, `mock-adapter.ts`).
- The design doc (§0.1 and Appendix A).

## The "four representations" resolved

The phrase comes from spec `interaction_model.dynamic_surfaces`: *"The same world MUST smoothly rearrange into orbital/orientation, causal, comparison, and temporal representations without becoming separate application modules."*

It means four **world representations** (surfaces) over one set of source objects:

| # | Representation | Mock | Operator in the UI |
|---|---|---|---|
| 1 | Orbital / orientation | 01, 05, 06, 13 | `surface: 'orbital'` (default), with focus and semantic zoom |
| 2 | Causal | 09 | `surface: 'causal'`: expected → assumption → condition → missing → observed chain |
| 3 | Comparison | none (the spec and VAR-003 and VAR-004 require it) | `surface: 'compare'`: A/B over two snapshots of the same objects (historical A vs B, published vs proposed design) |
| 4 | Temporal | 08 | `revision` cursor plus the history strip: the world *becomes* its earlier version |

It does **not** mean four artifact kinds. Artifacts are a separate axis: progressive disclosure (minimal → summary → source, spec 04 §7), opened in the right-hand viewer.

Other surfaces are derived from these, not a fifth representation:
- **Judgment** (mock 10) is the orbital representation with a consequential object brought forward and the authority panel open.
- **Execution inspect** (mock 11) and **case design** (mock 12) are workbench modes over the same world and template world.

## Scene inventory (mock → state before this phase → work needed)

| Mock / scene | Existed | Missing functionality | Source data / contract | In ← / out → | Affordances | Verified by |
|---|---|---|---|---|---|---|
| 01/13 Home (Core) | yes | Data came from a fixture, not the port; layout was keyed by fixture ids | `CognitiveWorldSnapshot` via `CaseworkAdapter` | boot, `h`, Core anchor → focus | wake, wheel zoom, click object | J0 |
| 05/06 Focused case | yes | Id-keyed layout; the parts slots needed to be data (`spatial_layout`) | `visible_objects[].parent_id`, `spatial_layout` | click object ← / Esc, Core anchor → | focus, zoom (LOD), hover relations | J1 |
| 02/03/07 Excerpt → dock | partly | Not source-backed (no `resolveArtifact`); no lazy renderers; no search, select or provenance; pin was fake (POST to Go); fake tabs | `ArtifactPayload` (content_type, provenance, digest) | excerpt click ← / close, Esc → exact snapshot | expand, tabs, search, select line/row, pin (local), close | J2 |
| 04 Multi-artifact beat | yes | Artifacts were not port-resolved; "Open in …" links | narration `openArtifact` directives | beat ← / dismiss → | open each | J5 |
| 08 Temporal rewind | partly | Could not select two positions or compare; used fixture revision ids | `queryTemporalTrajectory`, `getHistoricalWorld` | `t`, time strip ← / Return to now → | step, scrub, mark A/B, compare | J4 |
| Comparison (no mock) | no | Everything | two `CognitiveWorldSnapshot`s of the same ids | Compare button ← / Exit compare → | select object (both sides), open artifact | J3, J4 |
| 09 Causal | partly | Hard-coded ids and titles; reachable only by narration | per-object `x.representations.causal` plus relationships | "Causal view" chip, narration ← / Esc → | select node, open evidence | J6 |
| 10 Judgment | yes | Choices and labels hard-coded by id; the human path only | object `actions[]` (`ActionDescriptor.consequential`) plus `dispatchIntent` | invoke action ← / decided, Esc → | approve, request changes, escalate, evidence links | J8 |
| 11 Execution inspect | yes | Rail items were decorative; used a fake run | `execution_progress` / `settlement_recorded` events | "Inspect run" ← / Esc → | view diff, logs; rail Home/Timeline/Artifacts | J8 |
| 12 Case design | yes | Buttons with no effect; fake template; no proposal/compare | template world via the same port (`case_id` of the template) | "Design this case" from a focus ← / Esc → back to the focus | select stage, reorder, toggle evidence, compare with published, save local draft | J7 |
| Execution / settlement | partly | Completion and settlement came from fixture timers keyed to `ns-release` | adapter events + `x.settlement` | accepted intent → settled world | pill → inspect | J8, J9 |
