# Renderer chunk unit: checkpoint scope and evidence reconciliation

Date: 2026-10-07. Source-only bounded reconciliation. No source edits, tests,
typecheck, build, scanners, or Git commands were run.

## Current files and last approved identities

Read the original Phase 1 fixture assignment, fixture repair assignment, both
Phase 1 reviews, the focused RED record, the Phase 2 production assignment,
Phase 2 source review, gate/root-verification records, the latest private UI
cursor runtime review/citation correction, current sources, current status
files, and `workbench/AGENTS.md`/`.agents/AGENTS.md`.

The current three renderer-unit files exactly match the last independently
approved Phase 2 source identities:

| Path | Last approved SHA-256 | Current SHA-256 | Result |
|---|---|---|---|
| `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.ts` | `52cdf00b9650f0413c91e4375c0ce5b6527b63ad9f09219f2102383dca4dec64` | same | exact match |
| `apps/godspeed-cognitive-ui/src/build/rendererChunkContract.test.ts` | `25b74d25318ce97659c91f3d734ca9b2cd794d0b4d90b1ec54c045b7ce022d08` | same | exact match |
| `apps/godspeed-cognitive-ui/vite.config.ts` | `95c287317aa4028a1976c145124f918d0d856a3bb0ca60802f31abd909e901ea` | same | exact match |
| `apps/godspeed-cognitive-ui/package.json` | `85044b7b0619f390791f87d089a84463610814cc199f705662207f00337e2f54` | same | unchanged |

The phase2 source review approved the private bundle checker and build-only
Vite hook against those three hashes. The checker still validates nine exact
renderer module paths, unique placement, distinct dynamic-entry filenames,
and registry dynamic-import edges. It normalizes slashes and query/hash
suffixes, rejects path-boundary impostors, and does not mutate the bundle.
The Vite plugin runs only for `build`; no runtime renderer code, public
interface, dependency, package script, or lockfile is part of this unit.

## Original assignments and disposition history

* `renderer-chunks-original-assignment-oct06.md` (SHA-256
  `c8cd566639c191c6cde923acaee13fad889d1fca467ece081b04f5aa3636b142`)
  authorized two new source/fixture files with a throwing stub and synthetic
  bundle assertions. The first independent review rejected one gap: the
  same-basename impostor did not distinguish a path-segment-aware match from
  a raw `endsWith` match.
* `renderer-chunks-fixture-repair-original-assignment-oct06.md` (SHA-256
  `19dd797832d620a7b17a7461df28b98a9fba1b8a7f8f0c96c6806c4d440a26d6`)
  authorized only two `not-src` boundary negatives. The rereview approved the
  resulting helper/test hashes `c5d9e161…e0fd77` / `25b74d25…022d08`.
* `renderer-chunks-phase1-red-oct06.md` (SHA-256
  `fed218e0dafb441948ef2fddfe3470918347845690b4c13a8ae6e46043fc39e6`)
  records all 13 fixtures reaching their intended assertions against the
  throw-only stub; the focused RED was independently accepted. This is fixture
  evidence, not production behavior.
* `renderer-chunks-production-original-assignment-oct06.md` (SHA-256
  `8962db6257c7bee621e939978d7fc3549a7d9011225c8a467852b24fa56ced35`)
  authorized exactly the helper implementation and build-only Vite hook,
  with the fixture frozen. `renderer-chunks-phase2-independent-source-review-oct06.md`
  (SHA-256 `c1f91ebcce7c56c57f1c8a9a2f88cef6188cba5dabb9bd8d3287b24a2204288a`)
  approved the current three source identities.

No material source change from the authorized phase2 implementation exists in
the current files: the hashes still match exactly. The original assignments,
source approvals, focused RED, and build evidence remain separate immutable
records rather than being subsumed by the later private cursor checkpoint.

## Checkpoint scope distinction

The current status calls commit `f549bf0` the private UI cursor checkpoint
and identifies its source set as:

* `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts`
* `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorOrder.test.ts`
* `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorBounds.test.ts`
* `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.settlementAtomicity.test.ts`
* `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.hostileThrownValues.test.ts`
* `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts`

The renderer checker, its test, and `vite.config.ts` belong to the earlier
renderer-chunks assignment and were expressly excluded from that cursor
checkpoint per the operator's scope instruction. I did not inspect Git or
independently infer the commit path set; this path-set statement is bounded by
the root/current-status checkpoint record and operator instruction. Do not
describe the three renderer-unit files as newly checkpointed by `f549bf0`.

If a separately named renderer checkpoint is needed, its source path set is
exactly the three tabled files above; `package.json`/`bun.lock` are unchanged,
and `dist/` artifacts are generated outputs rather than source paths. Preserve
the existing cursor checkpoint's six paths as a distinct unit. No commit,
push, or Git comparison was performed here.

## Verification evidence and remaining provenance

The old Phase 2 gate record says the focused 13-test assertion, strict
typecheck, and production build passed. The build emitted the hook marker and
nine distinct renderer files, with hashes recorded in
`renderer-chunks-phase2-gates-oct06.md`; root separately reports that all
Phase 2 captures were byte-compared to contemporaneous `/tmp` originals.
However, the Phase 2 canonical `just casework-ui-check` attempt completed with
274 pass / 2 fail (a local-listener `EPERM` and the then-current cursor
conformance failure), and its root verification correctly did not accept that
gate as green.

The later private cursor runtime review records a successful `bun run build`
with the renderer marker, nine emitted renderer chunks, successful strict
typecheck, and a successful canonical `just casework-ui-check` (299 pass / 0
fail). That review's preflights hash the six cursor/conformance files, but do
not list the three renderer-unit paths. Current hashes now match the Phase 2
approved identities, so current source review is exact; the later run's
contemporaneous renderer-source identity is not independently recorded in its
preflight. The runtime review/citation correction is therefore valid evidence
for its reported UI gate and build result, but does not turn the excluded
renderer files into members of the `f549bf0` checkpoint or provide a new
renderer-specific current-hash preflight.

No renderer source approval is missing: Phase 2 review approved the current
exact source. No renderer-specific gate pass is inferred from the earlier
red canonical gate. Existing later canonical/build evidence is recorded with
the source-identity limitation above. If an operator wants a standalone
renderer checkpoint or renewed evidence tied to the current worktree, the
bounded next step is an explicit renderer-only preflight naming these three
hashes and then the existing focused assertion/typecheck/build/canonical
verification under serialized compiler ownership. Do not expand that task into
the cursor adapter or change the source absent a new assignment.

## Limits

This is a source/evidence-scope inventory, not a new source approval, runtime
replay, canonical-gate re-verification, browser claim, or checkpoint/remote
inspection. Existing source approval and captured gates are cited as records;
no evidence was fabricated or rewritten. No operator file was edited except
this new immutable recon record.

Graft saved ~12,299 tokens (<$0.01) this turn.
