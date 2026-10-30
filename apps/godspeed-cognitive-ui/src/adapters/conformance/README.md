# Casework Port Conformance

## Overview
Shared behavioral assertions for `CaseworkPort` implementations (`src/adapters/conformance/caseworkPortConformance.ts`). The same scenario runs against the local contract adapter (unit tests) and the real Go stack (the live ladder), so both adapters are held to one contract. A scenario supplies the port, a seeded case, and callbacks; `runCaseworkPortConformance` then drives reads, template preflight, a mutation, the event stream, temporal history, artifact resolution and refusal semantics.

## Interface
- `CaseworkPortConformanceScenario` — scenario inputs: `port`, `caseId`, `actor`, `template`, `artifact` (ref resolution, expected provenance, `digestMatchesContent`), `resume: 'replay-retained' | 'future-only'`, timeouts, `acceptedIntent`, `captureConsequentialState`, `stateBoundaryDescription` [caseworkPortConformance.ts:CaseworkPortConformanceScenario]
- `runCaseworkPortConformance(s)` — the common assertion suite (the only entry point) [caseworkPortConformance.ts:runCaseworkPortConformance]

## Invariants
Asserted against any conforming adapter (all cited from [caseworkPortConformance.ts:runCaseworkPortConformance]):
- A snapshot names the requested case, has a cursor and projected objects.
- Trajectory points are case-scoped, and `base_cursor`/`head_cursor` match the first/last point; a retained cursor resolves via `getSnapshotAt` to its exact snapshot.
- A valid intent is accepted, advances the projection cursor, and a post-dispatch `snapshot`/`settlement_recorded` event with an advanced cursor arrives on the subscribed stream.
- Resume semantics: `replay-retained` replays the exact accepted mutation cursor with chronologically advancing cursors; `future-only` never replays events already present at subscription time.
- New activity appends trajectory history and never rewrites a retained snapshot (byte-identical `getSnapshotAt`).
- Artifacts resolve to non-empty content with a well-formed `sha256:` digest, schema-shaped provenance (`case_id`, `plan_item_id`, `invocation_id`, `run_id`), and — when `digestMatchesContent` — a digest matching the actual returned bytes.
- A stale intent (old `client_cursor`) surfaces the typed `STALE_PROJECTION` refusal, changes no consequential state, and appends no trajectory revision.

## Side Effects
- Network/async calls against the supplied `port` (reads, `dispatchIntent`, `subscribeEvents`) [caseworkPortConformance.ts:runCaseworkPortConformance]
- SHA-256 digests computed via `crypto.subtle` for artifact-integrity checks [caseworkPortConformance.ts:sha256]

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; the conformance suite is exercised through `src/adapters/local/*.test.ts` and the live harness)
