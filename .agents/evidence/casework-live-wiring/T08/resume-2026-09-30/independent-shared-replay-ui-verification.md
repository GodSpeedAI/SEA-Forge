# Independent shared replay and UI verification

**Shared live replay passes. T08 remains unconfirmed because the canonical UI gate's
TypeScript stage fails.** The common replay correction is test-only; no production transport
or adapter code changed in this verification phase.

## Replay correction review and tests

The final diff in `src/adapters/conformance/caseworkPortConformance.ts` waits for the first
event with `cursor >= accepted.new_cursor`, fails unless that cursor equals the exact receipt
cursor, then validates every event received through that match as strictly advancing from
the requested snapshot cursor. An event beyond the target fails immediately. The future-only
local branch and other assertions are unchanged. This is behaviorally equivalent to waiting
for the exact cursor while also detecting a stream that passes over it.

The phase-one fixture tests in `src/adapters/local/localAdapter.test.ts` were unchanged
(SHA-256 `4b4e76d8a9bd5424327d265042d961baba8561d2afac1b2d2bcd48cb8d3a8aef`). They use the
real local adapter for dispatch and first subscription, and control only the second
replay subscription. The focused tests demonstrate ordered-prefix acceptance, missing-target
timeout, and out-of-order-prefix rejection.

Actual-host checks before the Bun commands showed 2,053,224 kB then 1,991,816 kB available,
with no Go/Cargo/Vite compiler active; unrelated Bun PID 13814 was left untouched.

- `bun test src/adapters/local/localAdapter.test.ts -t 'shared conformance'`: **exit 0**,
  3 passed, 0 failed. Raw log:
  `/tmp/t08-shared-replay-green-focused-20260930.log`
- `bun test src/adapters/local/localAdapter.test.ts`: **exit 0**, 16 passed, 0 failed,
  24 assertions. Raw log:
  `/tmp/t08-shared-replay-green-local-adapter-20260930.log`

## Fresh shared live conformance

Actual-host preflight at 2026-09-30 21:44:09 UTC showed 2,060,064 kB available, no Go/Cargo
compiler process, and port 4179 free. The runner used a new owned cell/gateway and pinned its
internal Go build to `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1` with the existing
writable Go cache.

From `apps/godspeed-cognitive-ui`:

```sh
CASEWORK_LIVE=1 bun run e2e/live-conformance.ts
```

The command completed with **exit 0**. The raw transcript records health and unauthenticated
refusal, operator login/session, template preflight, PROPOSE and case identity, target-specific
action offers, the shared CaseworkPort suite including exact accepted-cursor replay and
retained history/immutability assertions, stale refusal with durable no-write, durable
`item_activated`, and the separate R-SO session. The runner reported its owned gateway
shutdown and `all checks passed`. Successful runner cleanup removes its temporary run root;
the raw transcript is retained at
`/tmp/t08-shared-live-green-20260930/shared-live-conformance.log`.

## Canonical UI gate and independent test result

The actual-host preflight at 2026-09-30 21:44:49 UTC showed 2,007,268 kB available; port
4178 was free. `XDG_RUNTIME_DIR=/tmp just casework-ui-check` exited **2** at `tsc --noEmit`,
before its build and tests. Errors were:

- `e2e/live-conformance.ts:104`: Bun `TCPSocketListener` lacks required property `unix` from
  the declared `UnixSocketListener` type.
- `e2e/native-events-live.ts:84`: `XObject` has no `settlement` property. The current
  `objectStanding` includes that undefined field on both sides, so its equality comparison
  does not detect settlement changes; do not conceal this with a cast or contract widening.
- `e2e/native-events-live.ts:195,370`: both local `disconnected` bindings are unused.
- `e2e/native-events-live.ts:261`: TypeScript narrows the event-array length to literal `2`
  and reports a comparison with literal `4` as unintentional.
- `src/adapters/local/localAdapter.test.ts:213`: the proxy forwards four arguments to the
  concrete local adapter's three-argument `subscribeEvents` method.
- `src/adapters/local/localAdapter.test.ts:242`: fixture actor includes `display_name` and
  `kind`, which are outside the conformance scenario's declared actor shape.

The exact gate log is `/tmp/t08-ui-gate-20260930.log` (explicit `exit=2`). I then ran the
recipe's Bun test portion separately to collect an independent result:

```sh
bun test
```

**Exit 0: 258 passed, 0 failed, 1,271 assertions across 21 files.** Raw log:
`/tmp/t08-ui-bun-test-suite-20260930.log`. This does not make the failed typecheck green.

The prior local browser ladder remains reusable evidence from round 8: 11/11 journeys passed
in `recovery-critic-round-8-gates.md`. The app entry and browser ladder sources were not
changed in this verification; the changed live runner is separate from that local ladder.

## Canonical production bundle variants

The inherited shell had `NODE_ENV=development`. I first ran raw `vite build` diagnostics
without a production pin into `/tmp/t08-production-bundles-20260930/{default,local,invalid}`.
All three emitted local fixture chunks, including an 88.21 kB `localAdapter` chunk; these
are **not production evidence** because they used the inherited development environment.
Those diagnostic outputs and logs are retained unchanged. Before canonical builds, the
preexisting ignored `dist/` was copied to
`/tmp/t08-production-bundles-20260930/preexisting-dist/`.

The three normative variants then used the package script, whose command prints
`NODE_ENV=production vite build`, and the normal ignored `dist/` output was copied to a
unique `/tmp` path after each build:

```sh
env -u VITE_CASEWORK_SOURCE bun run build
VITE_CASEWORK_SOURCE=local bun run build
VITE_CASEWORK_SOURCE=invalid bun run build
```

All three builds exited **0**. The outputs under `package-default`, `package-local`, and
`package-invalid` are byte-identical; each contains the HTTP adapter chunk and no local
adapter, Northstar, fixture, or local narrator marker/chunk. Scanned markers:
`LocalContractAdapter`, `NORTHSTAR_CASE_ID`, `northstarData`, `createLocalAgent`,
`localAdapter`, `localAgent`, `evi-case-timeline`, and `case-northstar`. The bundle logs are
`package-default.log`, `package-local.log`, and `package-invalid.log` in the same `/tmp`
directory. These builds do not erase the unresolved TypeScript gate failure.

## Source hashes and remaining work

```text
b2514a8ec08c077a9c250394a58ec68219ed1e8bef3f80c6f848a8fad39b3366  apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts
4b4e76d8a9bd5424327d265042d961baba8561d2afac1b2d2bcd48cb8d3a8aef  apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.test.ts
52d881816346d3a71ba1e457efd72f65c46c021513837ea42fd1a0c93f4db6bf  apps/godspeed-cognitive-ui/e2e/live-conformance.ts
d66b19e6bc59cb79195eaad75d6afa9cac86e9d4cb5e4f9035c05b80abb8d4e3  apps/godspeed-cognitive-ui/src/main.tsx
85044b7b0619f390791f87d089a84463610814cc199f705662207f00337e2f54  apps/godspeed-cognitive-ui/package.json
```

Independent native EventSource race evidence from the unchanged native test remains recorded
in `store-retention-green-evidence/native-live-independent-count1.log` (exit 0, 35.546 s).
Prior independent Store/Go and Rust evidence is separately recorded in that directory and
the T07 authorization evidence; it is reused only for the unchanged source scope. Full T08
approval remains blocked until the type errors are repaired and the required canonical UI
gate passes.
