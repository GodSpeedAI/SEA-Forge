# T08 recovery round 2 fixes

## Original assignment and boundaries

- Read the T08 plan/spec, current handoff protocol, and round-2 independent rejection before edits.
- Owned only `internal/server/trajectory.go`, `trajectory_test.go`, the native `EventSource` region in `httpCaseworkAdapter.ts`, the obsolete trajectory test in `httpCaseworkAdapter.test.ts`, the focused judgment setups in `app/intents.test.ts`, and a new native retry test file.
- Preserved the shared event stream contract: `/api/events` accepts `?last=` or `Last-Event-ID`, emits each retained cursor as the SSE ID, and the adapter filters other cases.
- No dependency, public contract, or security model changes. No Go/Bun compile, test, or build command was run because the compile token has not been granted.

## Changes

- Trajectory completion counts now include only `COMPLETED`; `REJECTED` remains in the total but is not counted as completed.
- Consequential event classification enumerates the known state-changing/lifecycle TraceKinds (plus the existing `case.submitted` event); informational and unknown kinds remain false.
- Native EventSource recovery now closes failed sources, retries with capped exponential backoff, resumes from the last received monotonic cursor, suppresses repeated cursors, preserves credentials, and cancels retry timers on disposal. It does not replay intents.
- Replaced the stale “trajectory unsupported” test with an authenticated endpoint call asserting a typed `not_found` response.
- Added native EventSource tests for error backoff/cap, successful reset, cursor resume, replay deduplication, and disposal.
- The two stale-refusal tests now invoke a real non-discretionary consequential action so their strong judgment outcome assertions have a judgment panel to settle.

## Material deviation

The first stale-refusal test previously moved to a historical revision before submission. The store deliberately prevents opening judgment from a historical view and clears judgment when time changes. To preserve that invariant and exercise the real user path, the test now opens judgment on the live head and verifies that refresh follows the new head while preserving focus. Historical views remain read-only.

## Verification state

- Prior independent-run failure remains immutable at `../bun-focused-round2.log` (17 passed, 3 failed, 70 assertions); it predates these fixes.
- `gofmt -d` on the two trajectory Go files and `git diff --check` both returned clean.
- Focused Bun command, run with 2.3 GiB available RAM:
  `bun test src/app/intents.test.ts src/app/live.test.ts src/adapters/http/httpCaseworkAdapter.test.ts src/adapters/http/httpTrajectory.test.ts src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts src/adapters/local/localAdapter.test.ts`
  **exit 0: 35 passed, 0 failed, 116 assertions across 6 files**, including the shared local port-conformance suite. Full output: `/tmp/t08-bun-focused-round2-fixes.log`.
- The first attempt piped the same passing test set to this evidence directory and exited 1 because shell redirection was denied (`Read-only file system`); Bun still reported 35 pass / 0 fail. The successful rerun above preserved its output in `/tmp`.
- No Go compile/test/vet/build, Bun typecheck, or build command was run. Focused trajectory Go tests and shared live conformance remain pending the parent’s Go token and integration run.
