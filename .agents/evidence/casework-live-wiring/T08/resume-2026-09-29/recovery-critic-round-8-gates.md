# T08 independent recovery review — round 8 gates

**The bounded source units are approved; T08 remains unconfirmed.** Focused UI tests,
typecheck, the escalated full Bun suite, and all 11 local browser journeys pass. A controlled
production-mode Vite build excludes local adapter and narrator assets. The canonical UI recipe
still needs rerunning after its build script pins `NODE_ENV=production`; its current normal
sandbox run encounters the explicit-loopback test's `EPERM`. F14 native reconnect/retention-one
proof and shared real-kernel conformance remain blocked on operator gateway-launch approval.

## Source review

Reviewed the original assignments/results in `trajectory-endpoints-retry-fix.md`,
`ui-type-gate-fix.md`, `native-test-count-fix.md`, `production-import-guard-fix.md`, and
`sse-loopback-test-fix.md`, and the final TypeScript source diffs for the trajectory adapter,
trajectory tests, native EventSource adapter/tests, intents, live UI tests, shared adapter tests,
and `main.tsx`/`journeys.test.tsx`.

The F13 response validator enforces the documented canonical required fields and nonnegative
integer counts (`completed <= total`), requires a nonempty checkpoint series with base/head
matching its first/last cursor, and rejects malformed/cross-case payloads as typed invalid
refusals. Tests cover missing/wrongly typed fields, invalid counts/cursor bounds, and valid
optional actor/stage cases. Stale refresh uses the active case identity and typed actor, then
preserves the refusal without replaying the intent. Native retry backoff grows on open/drop
cycles, cursor advancement follows validation/scope/deduplication, resync control does not
suppress a same-cursor snapshot, and the test now counts actual native EventSource instances
through the retry timer. The shared conformance suite invokes the same runner for local and live
adapters. The final `main.tsx` keeps local adapter/narrator imports behind the direct DEV guard;
production narration remains null until T09 provides grounded Ask wiring. The 503 tooth retains
its real HTTP path, ephemeral port, reconnect/resume assertions, and cleanup, with an explicit
loopback hostname. No remaining source defect was found in the bounded repair units.

## Independent focused verification

Runs were serialized. Before Bun/typecheck/build commands, available RAM ranged from 3.1–3.5
GiB; no concurrent Go/Rust build ran.

From `apps/godspeed-cognitive-ui`:

```text
bun test src/adapters/http/httpTrajectory.test.ts src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts src/app/intents.test.ts src/app/live.test.ts src/adapters/local/localAdapter.test.ts src/adapters/http/httpCaseworkAdapter.test.ts
```

**PASS — 43 tests, 0 failures, 144 assertions.**

```text
bun run typecheck
```

**PASS — exit 0.**

Full suite, first in the normal sandbox:

```text
bun test
```

**FAIL — 254 passed, 1 failed, 1262 assertions.** The only failure was
`src/ui/journeys.test.tsx:352`: `Bun.serve` could not bind `127.0.0.1` with `EPERM`.
The test did not skip, mock, or weaken the listener.

The same full command was then run directly with the approved `require_escalated` tool context
(not nested under `just`): **PASS — 255 tests, 0 failures, 1268 assertions.** This is distinct
from the normal-sandbox failure and proves the test works when the network bind is allowed.

Canonical command, run in the normal sandbox with the required runtime-directory override:

```text
XDG_RUNTIME_DIR=/tmp just casework-ui-check
```

Frozen install and typecheck passed, and Vite completed its build; the recipe then failed at the
same explicit-loopback test tooth (254/1, 1262 assertions). This invocation inherited
`NODE_ENV=development`, so its Vite build was not a valid production-isolation result. The
installed Vite config defines `import.meta.env.DEV` as `!isProduction`, where `isProduction`
depends on `NODE_ENV === 'production'`.

## Production asset verification

The inherited shell environment was measured directly as `NODE_ENV=development`. Therefore the
following diagnostic builds do **not** count as production checks even though Vite printed
“building client environment for production”:

```text
VITE_CASEWORK_SOURCE=local ./node_modules/.bin/vite build --outDir /tmp/T08-prod-local-20260930-round8
VITE_CASEWORK_SOURCE=invalid ./node_modules/.bin/vite build --outDir /tmp/T08-prod-invalid-20260930-round8
```

Both emitted local adapter/narrator chunks because the inherited environment selected Vite's
development definition. The canonical recipe likewise emitted such chunks under this inherited
environment. These outputs are retained as diagnostics, not reported as production leakage.

Controlled causal check:

```text
NODE_ENV=production VITE_CASEWORK_SOURCE=local ./node_modules/.bin/vite build --outDir /tmp/T08-prod-nodeprod-local-20260930-round8
```

**PASS — exit 0, 109 modules transformed.** The output contains `httpCaseworkAdapter` and no
`localAdapter` or `localAgent` asset. Scanning every emitted JS asset for
`LocalContractAdapter|NORTHSTAR_CASE_ID|northstarData|createLocalAgent|localAgent` found no
matches. This validates the direct DEV guard when Vite is actually configured for production.
The package build script must pin `NODE_ENV=production`, after which the canonical recipe and
explicit default/local/invalid production variants still need fresh independent checks.

## Local browser regression ladder

Command, run against the local UI only:

```text
bun run e2e --out ../../.agents/evidence/casework-live-wiring/T08/resume-2026-09-29/local-ladder-round8
```

**PASS — 11/11 journeys:** J0 6/6, J1 6/6, J2 7/7, J3 6/6, J4 8/8, J5 6/6, J6 6/6,
J7 8/8, J8 8/8, J9 9/9, and RECOVERY 8/8. Results and screenshots are under
`local-ladder-round8/`. `just casework-ui-up` reported that a pre-existing listener was reused;
I did not start or stop it. I verified its `/src/main.tsx` response contained the exact final
workspace source and direct DEV guard, and closed only the isolated browser session used for
that check. The ladder itself closed its per-journey browser sessions.

## Bind diagnosis and remaining live gates

An isolated normal-sandbox Bun diagnostic using the test's exact listener options
(`hostname: "127.0.0.1"`, `port: 0`, HTTP 503) failed at `listen` with `EPERM`. The same
diagnostic with the approved escalated execution context started on `127.0.0.1:41941`, fetched
its actual response, returned status `503`, and exited 0. The normal `just` failure is therefore
the sandbox's loopback bind restriction; it is not a reason to replace the real listener tooth.

No gateway was launched. F14 still needs a real native EventSource integration proof with
retention-one resync/reconnect against the approved live kernel, and the identical shared
conformance runner must pass against that live adapter. The operator approval for gateway launch
is still pending. Production binary/local/invalid bundles and the canonical UI recipe also need
their final post-`NODE_ENV`-pin rerun. T08 is not settled until those gates complete.

No product source, status, or commit was changed by this critic.
