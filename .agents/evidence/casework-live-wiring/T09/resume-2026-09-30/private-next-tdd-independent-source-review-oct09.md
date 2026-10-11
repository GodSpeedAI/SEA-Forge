# Independent source review: private Next TDD fixtures

## Verdict

**REJECT source readiness.** The four-file change is within the released
TDD-scaffold boundary and does not implement lifecycle behavior, but the test
file contains static compile errors and several fixtures do not establish the
required behaviors. No compiler, tests, formatter, or runtime gate was run.

## Evidence reviewed

I read the full original `run-observation-private-next-assignment-oct09.md`,
sequencing addendum, notifier-proof addendum, and full
`private-next-tdd-builder-result-oct09.md`. The frozen source identities are:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `b45dc39bc7466160cec9bc441d2635d577ceae69f64f165f19dbca7f7605737d` |
| `run_observation_poller_worker.go` | `2e5ef4ee58b1c821b6b5a9fdee83a2bce38f504dae3001f08e2b73306f350766` |
| `run_observation_next.go` | `f362b461744b7b474f71a6d91c01f01eba76752c4c82761af5fa06ddb3db4015` |
| `run_observation_next_test.go` | `c8e8253539ed7d72f61b417f99e2849bcc711fb8401cd395c6c35094ba216261` |

The manager diff adds only lease field declarations. The worker diff adds the
approved notifier target/helper stubs; the new Next file contains the private
aggregate, projector/error types, and unavailable stub. Existing manager tests,
worker lifecycle, and `buildRunObservationRunDelta` are unchanged. This matches
the TDD-only release and notifier proof seam; no algorithm implementation is
claimed.

## Blocking findings

1. **The test file does not statically compile.** Four list callbacks declare
   `(ports.RunListResult, error)` but return only the result at
   `run_observation_next_test.go:314`, `:558`, `:600`, and `:654`. In addition,
   `runObservationManager.cohorts` is `map[*runObservationLease]struct{}`
   (`run_observation_manager.go:50`), but the tests use its single map value as
   a boolean at `run_observation_next_test.go:519`, `:629`, `:643`, `:696`,
   `:714`, and `:751`. Those checks need comma-ok membership results.

2. **The terminal hydration/no-replay fixture cannot establish pruning.**
   `TestRunObservationNextAcceptsTerminalCaptureWithoutReplayingHydratedFrames`
   (`run_observation_next_test.go:86–116`) supplies one run with 1,024 frames
   whose event IDs are short. The aggregate 1 MiB hydration cap is not reached
   by that fixture, so its assertion that Prepare pruned frames is unsupported
   and can fail before calling Next. The existing proven setup uses eight runs,
   1,024 frames each, and long IDs (`run_observation_manager_test.go:358–450`)
   and checks aggregate DTO pruning. Adapt that real shape to assert successful
   terminal Next does not replay frames removed by hydration.

3. **The all-or-nothing watermark test is vacuous.**
   `TestRunObservationNextDiscardsWholeMultiRunCandidateAfterOneProjectionFails`
   (`run_observation_next_test.go:388–428`) starts with terminal `completed`
   captures at high-water ordinal 1. Prepare seeds each watermark from that
   same accepted state, so the first candidate cannot advance it. The
   before/after equality assertion therefore cannot detect a partial commit.
   The test then retries immediately after an injected projector error without
   a later publication/wake. Make the candidate advance beyond its baseline and
   coordinate a genuine later fitting publication for recovery. Root must
   settle the projector-error disposition so that recovery does not rely on an
   unspecified wake or terminal-drain policy.

4. **The initial no-current recovery case is absent.** The tests cover a
   transient read failure only after Prepare accepted an initial current value
   (`run_observation_next_test.go:245–302`). They do not cover accepted list and
   attachment with no `retainedCurrent`, followed by a later fitting value.
   The builder result correctly says that such a lease has no scalar baseline
   and must use zero prior so its first genuine frames are delivered; the
   fixture must prove that case without manufacturing watermark state.

5. **Authorization sequencing is not exercised at the required boundaries.**
   The authorization/cursor variants at `run_observation_next_test.go:430–483`
   change authorization or cursor before calling Next. The cancellation,
   detach, and Stop cases pause projection, but no test makes the first auth
   check pass and the post-wake check fail, mutates present context between
   projection and final disclosure, or blocks an auth callback while detach
   claims the already-registered operation. The sequencing addendum requires
   pre-wait, post-wake/pre-capture, and final disclosure checks, with operation
   registration before callbacks. Add deterministic assertions for those
   boundaries and callback ownership.

6. **Failure cleanup can strand the transaction and notifier references.**
   Five projector-barrier tests close `projectRelease` only on their success
   path (`run_observation_next_test.go:164–221`, `:501–539`, `:572–592`,
   `:612–639`, and `:678–703`). On an earlier fatal assertion,
   `cleanupRunObservationManager` cancels workers and releases held reads but
   does not release these projectors; it waits only two seconds for Stop/drain
   (`run_observation_manager_test.go:178–245`). Use once-backed deferred
   cleanup to release each barrier. The notifier ownership test also lacks a
   deferred once-backed `sendPollerNotifierTargets` release after registering
   real targets (`run_observation_next_test.go:737–771`), so a failing assertion
   can leave the drain waiting for its reference.

7. **Aggregate gap ordering has no oracle.** The required transaction sorts
   both runs and window gaps by exact run-ID bytes. The tests assert run order,
   but the only `WindowGaps` assertion checks the empty aggregate at line 67.
   Add a multi-run case with actual gaps and assert exact run-ID byte ordering.

## Scope and limits

The positive scope findings above do not overcome the blockers. This is a
fixture/scaffold rejection only; it is not a claim that production Next is
implemented or incorrect. Preserve the existing assertions and pure projector,
repair these fixtures in a fresh source-preparation pass, then request a new
independent review before root's expected-RED gate. No files were changed by
this critic.
