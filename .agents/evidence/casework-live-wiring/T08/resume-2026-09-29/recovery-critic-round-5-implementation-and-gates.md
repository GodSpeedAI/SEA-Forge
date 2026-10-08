# T08 independent review — round 5 implementation and focused gates

**F13: REJECT. T08: remains unconfirmed.** This round reviewed the trajectory-shape builder's final two source files and independently ran the authorized focused Bun command and UI typecheck. No source or status files were edited. No gateway was launched; no Go or Rust compilation was attempted.

## F13 residual shape defects

The new `isTemporalCheckpoint` validator at `apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:108-126` now validates all required checkpoint field types, permits empty unknown actor strings and absent optional stage ID, and enforces nonnegative integer counts with completed not greater than total. Those count constraints match the Go producer's integer counters and derivation in `apps/godspeed-casework-go/internal/server/trajectory.go:90-120`.

The response-level validator still accepts an empty points array and does not establish that `base_cursor` and `head_cursor` equal the first and last point cursors. The shared port assertions require those endpoint equalities (`src/adapters/conformance/caseworkPortConformance.ts:70-75`), and the Go producer only returns a successful response for nonempty retained history, setting base/head from its first/last revision (`internal/server/trajectory.go:66-82`). The new positive fixture `VALID_TRAJECTORY` itself has `head_cursor: '01BBB'` while its only point cursor is `'01AAA'` (`src/adapters/http/httpTrajectory.test.ts:10-17`), so it blesses a structurally inconsistent trajectory. Add nonempty/end-point validation and correct the fixture; cover empty points and mismatched base/head as typed-invalid regressions before approving F13.

## Independent verification

Before the focused test command, `free -m` reported 3815 MiB available; `ps` showed only the Codex process. Ran from `apps/godspeed-cognitive-ui`:

```text
bun test src/adapters/http/httpTrajectory.test.ts src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts src/app/intents.test.ts src/app/live.test.ts src/adapters/local/localAdapter.test.ts
```

Result: **31 passed, 1 failed, 109 assertions, exit 1.** The F13 tests passed, but the valid fixture's inconsistent endpoint cursors above were accepted. The failure is in the newly added native-event test `httpCaseworkAdapter.nativeEvents.test.ts:201`: after `resumed.fail()`, it checks the latest EventSource URL for `last=01M2` before firing the scheduled retry timer. No new EventSource has been constructed yet, so the latest instance still has the original `last=01M0` URL. The earlier recovery test runs the timer before checking the new URL (`:106-112`). Add `timers.runNext()` before the resumed-instance URL assertion, preserving the delay/cap assertions. This is an actionable test defect; it does not establish a stream implementation defect.

Before typecheck, `free -m` reported 3816 MiB available; `ps` showed only the Codex process. Ran from `apps/godspeed-cognitive-ui`:

```text
bun run typecheck
```

Result: **exit 1.** TypeScript reported errors in `httpCaseworkAdapter.nativeEvents.test.ts:10,79` (unsafe function casts), `httpCaseworkAdapter.test.ts:156` (content-type literal widening), `localAdapter.test.ts:151,168` (argument count), `app/intents.ts:82` (string not assignable to `ActorRole`), and `app/live.test.ts:29,63,96` (partial port not assignable to `CaseworkPort`). No error pointed at the new F13 validator or trajectory-shape matrix itself, but the UI gate is not green. Preserve and reconcile all errors rather than attributing them to baseline without evidence.

I did not run full `bun test`: root stopped the broader command after the focused suite exposed a known source-test failure and assigned fresh bounded builders. I did not run `bun e2e`, production builds, Go gates, or any live conformance, because the root asked me to release the exclusive compile token to the T07 critic after recording these results.

## Native live evidence remains outstanding

The F14 evidence gap from round 4 remains: fake EventSource tests plus a separate Go server resync test do not prove the production native EventSource path against the fresh gateway. Require a real native EventSource integration that forces the requested cursor before a single retained revision, then checks `resync_required` followed by that same-cursor snapshot, plus real interruption/reconnect and retained replay. No gateway was launched because approval for that operation remains pending.
