# Cognitive UI: functional-completion phase report (2026-09-23)

**Scope.** This phase made the cognitive environment functionally complete and proved it through an affordance-dependency E2E ladder that drives the real rendered UI.

**What it runs against.** Everything runs against a **local contract-conformant adapter** behind the interface-contract port. The Go system front end was deliberately **not** connected, so none of this is backend-integration proof.

**Commits.** Nothing is committed.

Companion documents in this directory:
- `01-SCENE-INVENTORY.md`: the mock/spec → scene inventory and the resolution of "four representations".
- `02-JOURNEY-CUBE.md`: the ladder, depends_on/settles/unlocks, the Journey Cube, and the variation and recovery matrix.
- `03-CONTRACT-PORTS-AND-SEAMS.md`: ports, what the local adapter simulates, and proposed contract extensions and missing seams.

Evidence (results.json, results.md, screenshots) is in `.agents/evidence/godspeed-casework-cognitive-environment/ui-journeys/latest/`.

## 1. "Four representations", resolved

The phrase comes from spec `interaction_model.dynamic_surfaces`. It means four **world representations** of the same source objects:
1. orbital/orientation
2. causal
3. comparison
4. temporal

It is not four artifact types. Judgment, execution-inspect and case design are modes or surfaces derived from these. Artifacts are a separate progressive-disclosure axis. See 01.

## 2. What changed architecturally

**Contract port and projection.** `src/ports/contract.ts` uses the interface-contract types verbatim, plus spec-04 shapes and optional `x` extensions. `src/ports/project.ts` is the only reader of contract shapes.

**The UI no longer knows any fixture ids.**
- `layout.ts` is data-driven: `spatial_layout` supplies slots, and per-object causal representations drive the causal view. A unit test asserts that no fixture ids appear in the file.
- The old fixture model, aliases, the Go adapter and the fixture-live adapter were removed.
- An audit shows fixture ids only in the local adapter's data and the scripted agent's script.

**Local contract adapter** (`src/adapters/local/`):
- Contract data at rest (JSON snapshots and trajectory, template world, `ArtifactPayload`s).
- Authority checks: duplicate, stale, unavailable, agent-denied, role, justification.
- Execution as contract events. Settlement arrives only in snapshots.

**One intent path** (`src/app/intents.ts`) for human and agent actors. **Narration port** plus a scripted agent (`src/narrative/`).

**Artifacts** (`src/artifacts/`):
- A payload parser, a `resolveArtifact` service, and 9 lazy source renderers, each its own chunk, behind an error boundary.
- Renderers: diff, text, markdown, table, chart, JSON tree, graph, trace (expected vs observed), timeline.

**New representation operators:**
- Comparison over two snapshots, with identity-preserving merge, per-object change notes and "N changes inside" summaries.
- Time marks and compare.
- Case-design drafts and compare.
- Center chips: Causal view, History, Compare with earlier, Design case.
- An artifact pill on objects: minimal, then summary, then source.

**E2E harness** (`e2e/`): `agent-browser` with real pointer input, and a dependency-aware runner that marks journeys BLOCKED.

## 2b. Transition graph (scene → scene, with the return path)

```
Home (Core, orbital)
  ├─ click region / case ─────────────▶ Focused case (orbital)                    Esc / Core anchor / h ◀─┘
Focused case
  ├─ ▤ pill → excerpt → click ────────▶ Artifact viewer (right dock)              close / Esc → exact ViewSnapshot
  ├─ "Causal view" chip ──────────────▶ Causal representation                     chip / Esc → orbital
  ├─ "Compare with earlier" / marks ──▶ Comparison (A↔B)                          Exit compare → prior revision + surface
  ├─ "History" chip / t / [ ] ────────▶ Temporal (read-only past)                 Return to live / Esc
  │     └─ Mark ×2 → Compare A↔B ─────▶ Comparison ──Exit──▶ back to the historical position
  ├─ question in composer ────────────▶ Narrated beats (focus/arrange/artifacts/time/compare)
  │     └─ any interaction pauses; "continue" resumes from the checkpoint; Esc ends
  ├─ "Design case" chip ──────────────▶ Case Design (template world, local draft, compare)   ✕ / Esc → the same focus
  ├─ hover object → action chip ──────▶ Judgment (backend-offered choices)        ✕ / Esc; closes itself when accepted
  │     └─ accepted ──────────────────▶ Execution pill → Execution inspect (workbench)       Esc / Cases rail
  └─ settlement snapshot ─────────────▶ World reorganised (settled, quiet)        h → Home
```

Every transition changes the camera and representation over the same DOM objects; no route changes. The Core canvas and node are never recreated.

## 3. Product defects the ladder found and fixed

1. Scene pointer capture swallowed clicks on every in-world control (chips, pills).
2. Excerpts anchored to an invisible object when an artifact was bound to two objects.
3. The dock header pushed Pin and Close off-screen.
4. The persistent Core node was unmounted during focus and case design; it is now never recreated.
5. Label hit-boxes stretched over neighbouring objects, and satellite labels stole clicks from their parent.
6. Hover was lost between a sphere and its action chips.
7. The judgment panel hid its choices after a refusal, so no fresh attempt was possible.
8. The execution run object was not recognised; the execution evidence list included unrelated evidence.
9. The workbench chrome blocked clicks on the world.
10. The agent was still called after the adapter was lost.
11. The diff renderer had invalid CSS class names, so added and removed lines were unstyled.
12. Design compare paired the wrong version, and change notes were uninformative.

## 4. Results

Final clean run, 2026-09-23, dev server at http://127.0.0.1:4178, headless Chrome with software GL at about 5 fps:

| Journey | Result | Steps | Time |
|---|---|---|---|
| J0 Orientation | PASS | 6/6 | 72 s |
| J1 Focus | PASS | 6/6 | 110 s |
| J2 Artifact inspection | PASS | 7/7 | 174 s |
| J3 Representation switching | PASS | 6/6 | 136 s |
| J4 Time + comparison | PASS | 8/8 | 235 s |
| J5 Narrated beats (+ RECOV-003) | PASS | 6/6 | 285 s |
| J6 Causal reasoning | PASS | 6/6 | 165 s |
| J7 Case Design | PASS | 8/8 | 184 s |
| J8 Consequential action | PASS | 8/8 | 182 s |
| J9 Integrated (no deep links) | PASS | 9/9 | 509 s |
| RECOVERY | PASS | 8/8 | 515 s |

- **Ladder total:** 11/11 journeys, 78/78 steps, 0 page errors. Results are in `results.json` and `results.md`, with 26 screenshots at settlement points, under `.agents/evidence/godspeed-casework-cognitive-environment/ui-journeys/latest/`.
- **Variations** VAR-001 to VAR-007 are exercised UI-side in J0, J2, J3, J4, J5 and J8. The backend portions of VAR-004, VAR-005 and VAR-007 are deferred (see 02).
- **Unit tests:** 185 pass, 0 fail, 633 assertions, 13 files. They cover projection, adapter authority and event order, reducer, view/compare, layout (including a no-fixture-ids check), design, player and conductor, and renderer helpers.
- **GATE_UI** (`just casework-ui-check`) is green: frozen install, typecheck (src + e2e), build, tests.
- **Lazy renderers:** the build emits one chunk per source renderer. J0 asserts that none load at boot; J2 asserts that only the opened kinds load (VAR-006).
- **Core persistence:** one canvas and one Core node carry a marker from J0 through J9 and RECOVERY. The marker is never lost.
- **No fixture-specific UI:** a grep of `src/{ui,scene,layout,model,app,ports,artifacts}` finds no fixture ids, and `layout.test.ts` asserts it for layout.

How to rerun:
- `just casework-ui-up`, then `cd apps/godspeed-cognitive-ui && bun run e2e` (about 45 minutes here).
- `bun run e2e -- --only J4` runs one journey, treating dependencies outside the selection as settled.

## 5. Still contract-fixture-only / deferred to Go integration

- The world, history, artifacts, authority, execution and settlement all come from the local adapter. A Go `HttpCaseworkAdapter` implementing `CaseworkPort` (REST + SSE) is the next integration step.
- Deferred and not proven:
  - RECOV-001 (Go restart during an active lease).
  - RECOV-002 (missed event → resync).
  - Real SEA Forge authority, Gauntlet execution and RealityTrace evidence.
  - The spec's real integration trace.
- Proposed contract extensions the Go side must supply (03 §Seams):
  - S1: relationships, artifact descriptors, presentation classes and causal representations.
  - S2: settlement in the snapshot, and the template link.
  - S3: narration x-directives.
- Missing operations:
  - S4: submitting a template proposal. The UI disables Submit and says why.
  - S5: persisted pins. Pins are local.
- Narration is scripted (no LLM). Execution pacing is simulated.

## 6. Known functional limitations

- Case design proposals cover reordering stages and required evidence only. Roles and goal are read-only. Drafts are not persisted across reloads.
- Comparison summarises nested changes on the visible facet ("N changes inside") rather than laying out deeper levels.
- History labels come from snapshot labels; there is no range scrubbing between checkpoints.
- The execution-inspect rail has disabled items (Search, Inbox, Agents, Governance, Settings), marked "Not available in this build".
- In a comparison, objects that exist only in A are shown as ghosts and are selectable but not focusable.

## 7. Visual fidelity deliberately deferred

- Case-design panel styling (version buttons, stage rows, footer).
- Satellite labels that overlap their parent's subtitle.
- Excerpt title wrapping with long kind badges.
- Compare-bar styling.
- Core shader realism.
- The 1.16 MB bundle (three.js is not split; the renderers are).

## 8. Next smallest settlement

Implement `HttpCaseworkAdapter` against the Go system front end for **read-only** paths: `getSnapshotAt`, `queryTemporalTrajectory`, `resolveArtifact`, and SSE `snapshot` events. Then run J0–J4 unchanged against it. Those journeys use no consequential path, so they prove the port boundary before authority and execution are wired.
