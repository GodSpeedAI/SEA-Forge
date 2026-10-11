# T08 independent recovery review — round 7 gates

**T08 remains REJECTED.** The F13 trajectory shape and prior stale-refresh/native-retry fixes passed the focused UI tests and typecheck. The production bundle check fails: default, explicit local, and invalid source builds all include the local adapter. Full Bun tests and the canonical UI recipe also stop at a sandbox loopback permission failure. No gateway was launched and no Go/Rust command was run. The compile token is released to root for the newly assigned source corrections.

## Independent review scope

Reviewed the final fresh builder notes `trajectory-endpoints-retry-fix.md`, `ui-type-gate-fix.md`, and `native-test-count-fix.md`, plus the seven changed TypeScript files. The native-event test count correction now correctly captures four existing EventSource instances and asserts no instance is added until the retry timer fires. Its Bun `fetch` mock preserves the original `preconnect` property. The F13 validator now requires complete canonical checkpoint fields, valid count semantics, nonempty points, and base/head cursors matching first/last points. The related test fixtures and invalid cases match those rules. Stale refresh uses the typed outgoing actor; the test type fixes preserve the assertions. No remaining static source defect was found in these seven files.

## Independent UI verification

Before each Bun/typecheck/build command, `free -m` showed 3744–4102 MiB available and `ps` showed only the Codex process. Runs were serialized.

Focused command, from `apps/godspeed-cognitive-ui`:

```text
bun test src/adapters/http/httpTrajectory.test.ts src/adapters/http/httpCaseworkAdapter.nativeEvents.test.ts src/app/intents.test.ts src/app/live.test.ts src/adapters/local/localAdapter.test.ts src/adapters/http/httpCaseworkAdapter.test.ts
```

**PASS — 43 tests, 0 failures, 144 assertions.**

Typecheck, from the same directory:

```text
bun run typecheck
```

**PASS — exit 0.**

Full suite, first in the workspace sandbox and retried through the approved escalation path:

```text
bun test
```

Both attempts ended **254 passed, 1 failed, 1262 assertions, exit 1**. The remaining failure is the existing local HTTP test tooth at `src/ui/journeys.test.tsx:349-362`: `Bun.serve({ port: 0, ... })` failed with `EPERM: operation not permitted, listen` at line 352. The escalated retry also failed at the same loopback bind. The previous recorded T09 UI gate did pass this tooth (`.agents/evidence/casework-live-wiring/T09/gates/ui-gates.log`); current failure is an environment bind restriction. A fresh builder is changing it to bind explicitly to `127.0.0.1`, after which the full suite must be independently rerun.

Canonical UI gate:

```text
XDG_RUNTIME_DIR=/tmp just casework-ui-check
```

**FAIL — exit 1.** Frozen Bun install completed with no changes; `tsc --noEmit` passed; Vite production build completed; the full test suite then hit the same `Bun.serve` loopback EPERM above. The first attempt without `XDG_RUNTIME_DIR=/tmp` stopped before recipe execution because `/run/user/1000/just` is read-only.

## Production bundle findings (blocking)

The default production build from `just casework-ui-check` emitted:

```text
dist/assets/localAdapter-Cy-YJ1r8.js
```

The default bundle contains `LocalContractAdapter`, `NORTHSTAR_CASE_ID`, or `northstarData` markers in that file and the main JS asset. Thus the required production-only HTTP adapter bundle check fails even though the build itself exits 0.

Both isolated production variants also built successfully but emitted the same local adapter chunk:

```text
VITE_CASEWORK_SOURCE=local ./node_modules/.bin/vite build --outDir /tmp/T08-prod-local-20260930
VITE_CASEWORK_SOURCE=invalid ./node_modules/.bin/vite build --outDir /tmp/T08-prod-invalid-20260930
```

`rg --files` and `rg -l 'LocalContractAdapter|NORTHSTAR_CASE_ID|northstarData'` confirmed the local adapter asset and markers in default, local, and invalid build outputs. This violates T08's production HTTP-only requirement that the local adapter be tree-shaken out; previous F-1 recovery work did not close the issue. A fresh builder now owns the production selector/asset boundary and the loopback-only test fixture.

## Local ladder and live evidence limits

The `agent-browser` skill was read. `XDG_RUNTIME_DIR=/tmp just casework-ui-up` initially failed to bind loopback in the sandbox; the authorized escalated retry started the UI dev server at `http://127.0.0.1:4178`. The ladder was deferred because the root assigned fresh builders to modify source after the current bundle failure. `just casework-ui-down` reported the saved PID stale, so the dev process was no longer running. Rerun the local ladder using the `agent-browser`-backed `bun run e2e` after the fresh source gate, writing evidence into a new T08 directory.

F14 native EventSource live retention-one resync/reconnect evidence and the shared real-kernel conformance suite remain outstanding. The operator approval for launching a gateway is still pending. Do not infer either proof from fake EventSource tests or the local adapter suite. Changed Go source still needs its separate fresh Go gates after approval and compile-token scheduling.

## Disposition

Static review approves the repaired trajectory-shape, stale refresh, native retry implementation/test design, and type-only test updates. Independent focused tests and typecheck pass. T08 cannot be approved until the production assets exclude local fixture data, the fresh loopback test passes, the local browser ladder is rerun, and the approved shared live/native EventSource evidence is collected. No product source, status, or commit was changed by this critic.
