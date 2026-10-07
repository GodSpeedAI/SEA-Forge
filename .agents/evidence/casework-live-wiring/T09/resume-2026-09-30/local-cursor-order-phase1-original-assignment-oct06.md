# Local ordinary cursor ordering — original Phase1 fixture assignment

Date: 2026-10-06. Builder: Luna live_cursor_v4_boundary_recon.
Root accepts revision4 6d7baafd after independent design review893ddd2e and full
source/proposal read. This release is one bounded TDD unit, not full UI/T09
completion. Prior invalid evidence anchors remain preserved under CW20.

## Write scope

- NEW `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorOrder.test.ts`.
- Existing `src/adapters/conformance/caseworkPortConformance.ts`: ONLY the
  subscription argument correction for future-only: use captured
  headCursorBeforeResume, retaining exact original snapshot.cursor for replay.
  Read whole affected file first. No assertion/timeout/replay weakening.

Do not edit localAdapter.ts, other source/tests, schemas, dependencies,
generated files, docs/status/debt or public interfaces. No algorithm/stub is
needed: the existing production local adapter is the honest pre-fix subject.

## Focused unit contract

Use controlled manual timers (existing HTTP nativeEvents test precedent), with
finally restoration and no sleeps/retry luck. Exercise the real adapter using
its existing API and well-formed intents/fixtures. No new production fixture
options or APIs; no global mock contamination. If a small test-only whitebox
helper is essential to queue an existing synchronous operation, bound it to
existing case/append/progress behavior, explain and verify its shape against
source, and retain public dispatch coverage for the real execution sequence.

Required independent test cases:

1. Omitted boundary: allocate/queue H+1 before subscribe(undefined), manually
   flush old delivery and assert no callback/error, then allocate genuine H+2
   after registration and assert its delivery/unsubscribe behavior.
2. Supplied revision H: queue H+1, subscribe(H), suppress that preexisting H+1,
   then deliver later H+2. Keep distinct from omitted-boundary case.
3. Public execution: controlled progression through progress, completion
   snapshot, settling progress, settlement snapshot and settlement side event.
   Preserve visible progress/settlement and assert every ordinary cursor unique
   and strictly increasing, side event after settlement snapshot, next snapshot
   after all previous ordinary events. Do not infer stream head from revision
   head alone or turn progress into retained history.
4. Immutable history: captured prior snapshots/trajectory stay DeepEqual after
   progress; progress/settlement side-event cursor cannot resolve as a retained
   snapshot. Subsequent snapshot adds exactly one matching revision/point.
5. Case-local isolation and disposal: independent cases advance independently;
   unsubscribe before queued drain prevents delivery. Existing healthy
   progress/settlement tests remain unchanged.

This unit intentionally covers ordering, subscription floors and history
compatibility. Full approved design's parser/invalid-input, queue64/error,
counter-ceiling/exhaustion/constructor validation require separate deterministic
fixtures before the complete algorithm can be accepted. Do not claim this
focused unit proves those other obligations or changes execution_observation.

Tests must compile against current source and fail for semantic reasons. Do
not rely on a missing future export or test expectation alone. Identify expected
positive RED assertions and any negative paths already satisfied by old code.

## Verification

Builder runs NO compiler, tests, scanner, Graft build, Git or network. Root
owns sole heavy push37401. Native apply_patch only. Read instructions/Graft and
nearby patterns first. Return new test/full conformance hashes and every
material deviation/whitebox assumption. A different independent source critic
receives this original plus full files; actual focused RED is held until fixture
approval, joined push and fresh resource/exclusive ownership checks. Production
ordering code and full algorithm remain HELD until root accepts actual RED.
