# T08 F13 endpoint and native retry test builder note

Date: 2026-09-29

## Original bounded assignment

> Fresh bounded T08 builder after independent critic rejected F13 endpoint semantics and a
> native-events regression setup. Read root AGENTS, `.agents` status, Graft, and current canonical
> sources. Edit only `src/adapters/http/httpCaseworkAdapter.ts`, `httpTrajectory.test.ts`, and
> `httpCaseworkAdapter.nativeEvents.test.ts` under `apps/godspeed-cognitive-ui`; do not run tests,
> build, vet, or compile until the root grants the token. Preserve the preceding trajectory shape
> field/count validation. Successful trajectories must have nonempty canonical points and endpoint
> cursors equal to the first/last point. Empty actor strings and omitted stage remain valid. Fix the
> representative fixture and cover empty points plus endpoint mismatch. In the native retry test,
> fire the scheduled timer before asserting the resumed URL; preserve backoff, dedupe, and resync
> assertions. Record original instructions/results/deviations and pending tests in a new T08
> evidence note. Do not alter contracts, dependencies, production stream logic, status, or commit.

## Basis

- The Go endpoint returns 404 for no retained revisions and sets `base_cursor`/`head_cursor` from
  the first/last retained revisions (`apps/godspeed-casework-go/internal/server/trajectory.go:67-81`).
- Shared port conformance checks nonempty configured retained history and equality between those
  endpoint cursors and the first/last point cursors (`src/adapters/conformance/caseworkPortConformance.ts:69-75`).
- Independent round-5 review recorded the accepting-empty/misaligned response defect, the
  inconsistent one-point positive fixture, the premature native resumed-URL assertion, and
  unrelated UI typecheck failures in
  `recovery-critic-round-5-implementation-and-gates.md`.

## Changes made

- `httpCaseworkAdapter.ts`: trajectory shape validation now rejects an empty `points` array and
  requires `base_cursor` and `head_cursor` to equal the first and last validated checkpoint
  cursors. Previous required-field, count-domain, empty actor, and optional stage behavior remains.
- `httpTrajectory.test.ts`: corrected one-point fixtures so both endpoint cursors equal that
  point; added typed-invalid cases for empty points and base/head cursor mismatches. Existing
  missing/wrong-type field matrix and permissive empty-actor/omitted-stage case remain.
- `httpCaseworkAdapter.nativeEvents.test.ts`: retry test now asserts no new source exists before
  the retry timer, fires that timer, and then checks the resumed `last=01M2` URL. Existing retry
  delay/cap, cursor dedupe, malformed/foreign-frame, disposal, and same-cursor resync assertions
  remain. Replaced the reported unsafe timer/fetch function casts with a guarded timer callback and
  a directly typed fetch stub; the timer monkey patch retains a double cast to the overloaded
  global `setTimeout` type.

## Verification, deviations, and remaining work

- No test, build, vet, or compile command was run. The exclusive compilation token is held by the
  T07 critic. All changes are pending focused and full UI verification by the authorized token
  owner.
- Round-5 `bun run typecheck` had additional failures outside this source scope:
  `httpCaseworkAdapter.test.ts:156`, `localAdapter.test.ts:151,168`, `app/intents.ts:82`, and
  `app/live.test.ts:29,63,96`. Root assigned those files to a separate UI type-gate builder;
  they are not edited here. The two reported native-events cast locations were within scope and
  were adjusted, but their typecheck result is unverified.
- F14 real native EventSource integration against a fresh gateway remains outstanding; fake source
  tests do not close that evidence gap. T08 remains unconfirmed.
- No contract, dependency, production stream behavior, status file, or commit was changed.

Pending verification after token release: focused trajectory/native-event tests, `bun run typecheck`,
full UI test/build gates, shared live conformance, and the independent F14 real native EventSource
resync/disconnect/reconnect proof.
