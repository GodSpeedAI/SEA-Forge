# T08 independent recovery review — round 6 static review

**Status: F13 trajectory-shape source appears repaired; T08 remains unconfirmed.** This review covered the fresh `trajectory-endpoints-retry-fix.md` and `ui-type-gate-fix.md` assignments/results and the final seven listed TypeScript files. No Bun/Go/Cargo/browser command was run; the T07 critic holds the compile token. No source or status file was edited.

## F13 re-review

The response guard at `apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:102-107` now requires the requested case, nonempty base/head cursors, a nonempty points array, and base/head equality with the first/last checkpoint. `isTemporalCheckpoint` checks the required contract fields and optional `active_stage_id` shape at `:110-126`. The count constraints (`Number.isInteger`, nonnegative, completed ≤ total) go beyond the TypeScript interface's `number`, but the note correctly documents their source semantics: Go uses integer counters initialized at zero, counts only completed items, and increments total from visible work items/milestones (`apps/godspeed-casework-go/internal/server/trajectory.go:90-120`). Empty actor attribution and omitted optional stage remain accepted. The trajectory fixture now has equal endpoints, and regression cases cover empty points and mismatched endpoints (`httpTrajectory.test.ts:8-18,136-143`). No F13 source defect remains apparent by static inspection; independent runtime tests are still required.

## Remaining static issue — native retry test instance count

The new delayed-resume assertion in `httpCaseworkAdapter.nativeEvents.test.ts:202-205` has incorrect expected instance counts. `subscribeEvents` constructs one initial source. The loop at `:186-192` runs three timers, each constructing one replacement, so there are four instances when `resumed` is selected at `:194`. After `resumed.fail()`, the scheduled retry has not fired, so the count remains four. The test currently expects five before `timers.runNext()` and six after it. It should assert no growth relative to a captured pre-retry count and exactly one new instance after the timer; the expected concrete counts here are four and five. This must be corrected before the test can pass.

The direct typed fetch assignment in `httpCaseworkAdapter.nativeEvents.test.ts:79-82` is an unverified typecheck risk. The prior independent typecheck explicitly said Bun's `typeof fetch` carries a required `preconnect` property. The new assignment removes the cast but still assigns a plain async function without that property. Only the authorized typecheck can settle this; if it remains red, preserve Bun's required shape without weakening types.

## Other changed files

- `intents.ts:82` now passes actor ID and role from the already-typed outgoing intent to the stale refresh, preserving the same session/user identity and satisfying the port's `ActorRole` type.
- `live.test.ts` replaces unsafe partial-port casts with a full `CaseworkPort`; unused methods fail loudly if called.
- `httpCaseworkAdapter.test.ts` retains the `text/markdown` literal type; `localAdapter.test.ts` now calls its declared three-argument subscription method. These changes match existing APIs and do not weaken assertions.
- The two builder notes accurately state that source changes were limited to their assignments, with no contract, dependency, or production-stream changes. Their claimed `git diff --check` is builder evidence, not an independent check.

## Independent verification to run after token grant

For every compile-like command, first check `free -m` and active compiler processes; serialize runs. Then run the focused set from `apps/godspeed-cognitive-ui`:

```sh
bun run typecheck
bun test src/adapters/http/httpTrajectory.test.ts src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts src/app/intents.test.ts src/app/live.test.ts src/adapters/local/localAdapter.test.ts src/adapters/http/httpCaseworkAdapter.test.ts
```

If focused gates pass, run `just casework-ui-check` from the repository root (frozen install, typecheck, production build, full Bun suite). Independently build the default production bundle and explicit `VITE_CASEWORK_SOURCE=local` and invalid-source production variants into isolated `/tmp` directories; verify every production variant excludes the local adapter. Then run the local browser regression ladder (`bun run e2e` from the UI app) using the already-read `agent-browser` skill, preserving its evidence and known baseline floor. Do not start the live gateway or shared real-stack suite without the pending user approval.

F14 and the shared live proof remain open. Before T08 confirmation, require the approved fresh-stack shared port-conformance run and the planned real native `EventSource` test against a live server with retention set to one: make the request cursor older than the single retained revision, verify `resync_required` then the same-cursor authoritative snapshot, and exercise actual interruption/reconnect and retained replay. A fake source or fetch fallback does not establish this native-browser path. Changed Go gateway code also needs fresh Go vet/race gates once its compile slot is authorized.
