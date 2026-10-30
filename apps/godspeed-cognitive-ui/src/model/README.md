# Model

## Overview
The UI's internal world model: state types, the reducer/store every surface reads, contract→internal projection helpers, presentation derivation, and view selection (live / historical / compare / design). Everything above `src/ports/contract.ts` sees only these shapes — never raw contract payloads.

## Interface
- `types.ts` — the whole internal state vocabulary: `WorldObject`, `WorldSnapshot`, `WorldHistory`, `Camera`, `Placement`, `ArtifactSpec`, `Beat`, `Narrative`, `UiState`, and the `Action` union [types.ts:WorldObject, types.ts:WorldHistory, types.ts:UiState, types.ts:Action]
- `store.ts` — `createStore(initial)` (external-store with `Listener` support), `reduce(s, a)` (the reducer), and selectors: `initialState`, `nowRevision`, `isPast`, `snapshotOf`, `focusOf`, `objectOf`, `expandedArtifact`, `cursorOf`, `designSnapshot`, `focusPath`, `emptyOverrides` [store.ts:createStore, store.ts:reduce, store.ts:nowRevision, store.ts:isPast]
- `view.ts` — view selection: `worldView(s)` (live/past/compare/design view of the current state), `diffObjects(a, b)`, `mergeForCompare(a, b)`, `sideLabel(h, cursor)`, `templateRoot(snap)`, `findArtifact(s, ref)` [view.ts:worldView, view.ts:diffObjects, view.ts:templateRoot]
- `derive.ts` — presentation derived from kernel fields (T09): `presentationOf(kind)`, `statusTone(status)`, `statusPhrase(status)`, `toneOf(o)`, `sentryExplanation(o, byId)` [derive.ts:presentationOf, derive.ts:toneOf]

## Invariants
- Snapshots append history and never rewrite it: revisions are keyed by cursor and ordered by `compareCursor` [store.ts:reduce, ports/project.ts:compareCursor]
- The past is read-only: `isPast` gates consequential interaction; historical views come from `history.snapshots[revision]` [store.ts:isPast, store.ts:snapshotOf]
- Authored `x` extensions (the local fixture) win over locally derived presentation wherever present; live snapshots carry no `x`, so presentation is derived from kernel fields [ports/project.ts, derive.ts]
- The reducer is the only place `UiState` is mutated; all components read through the store or selectors [store.ts:createStore]

## Side Effects
- ["None"] (pure state; the store only notifies `Listener` subscribers on dispatch [store.ts:createStore])

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; tests: `src/model/store.test.ts`, `derive.test.ts`, `view.test.ts`)
