# Renderer chunks Phase 2 bounded failure diagnosis — 2026-10-06

## Scope and result

This diagnosis isolates only the two failures from the canonical `just casework-ui-check` run. No source, test, configuration, dependency, Git, status, debt, or generated files were changed. The canonical retry remains a failed gate and is not accepted.

Graft query: `graft ask "future-only subscription emits replayed duplicate cursors in local adapter conformance" --in apps/godspeed-cognitive-ui/ --source`. Graft returned `runCaseworkPortConformance` at `caseworkPortConformance.ts:L61-L183`; its source excerpt identifies the future-only assertion at L136. Graft reported approximately 21,909 tokens saved (~$0.02).

## Cursor conformance failure

Canonical raw `renderer-chunks-phase2-canonical-retry-oct06-run.raw` (SHA-256 `85ba0b92ccb4a69fda5cdb0d1750e3df5c43d656a5cf4fe4a49c9eb7ce6edaed`) reports `LocalContractAdapter > shared CaseworkPort behavioral conformance`, with `subscription head was 1.0000000008; received 1.0000000008,1.0000000009,1.0000000009`. The condition at [caseworkPortConformance.ts:136](../../../../../apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts) checks *every* event is strictly newer than the captured head. Thus cursor `...8` is the actual violating event; duplicate `...9` values are incidental to that failed condition.

Focused source explains how the local future-only fixture can violate that strict check without replay support. `subscribeEvents` ignores the supplied cursor and adds the callback to a listener set ([localAdapter.ts:168-172](../../../../../apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts)). An accepted `EXECUTE_ITEM` starts asynchronous progress timers after appending its accepted snapshot ([localAdapter.ts:268-271, 347-366](../../../../../apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts)); progress events read the current snapshot cursor at delivery time ([localAdapter.ts:391-399](../../../../../apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts)). `emit` schedules each current listener with `setTimeout` ([localAdapter.ts:433-435](../../../../../apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts)). Consequently an asynchronous execution-progress event can arrive after the resumed subscription is registered while still carrying a cursor equal to the head observed immediately beforehand. The future-only assertion currently treats that event as replay. This is a timing-sensitive interaction of existing local-adapter event delivery and the conformance check, and does not involve renderer chunk source.

An isolated default-sandbox rerun of only `shared CaseworkPort behavioral conformance` passed once (1 pass, 15 filtered, exit 0). This shows the canonical failure did not reproduce in that single isolated run; it does not establish determinism, prove flakiness, or establish a pass for the full gate. No additional retry was made.

## Listener test failure

Canonical raw identifies `journey: execute + execution pill + sentry > TOOTH: SSE drop flips the store to reconnecting via the live adapter error path, and an event resumes it`. The test binds `Bun.serve` to `127.0.0.1` at [journeys.test.tsx:352](../../../../../apps/godspeed-cognitive-ui/src/ui/journeys.test.tsx), which failed at `listen` with `EPERM` in the default sandbox. The root-authorized, bounded escalated repetition of this exact named test passed (1 pass, 24 filtered, 6 expectations, exit 0); its intended local 503 responses and reconnect warnings appeared. That demonstrates the test behavior succeeds when local listening is permitted; it does not replace or retroactively pass the canonical gate.

## Captures, preflights, and hashes

Each run was joined before reporting its exit. Raw captures are archived beside this record and were compared byte-for-byte with the `/tmp` source captures.

| Isolated command | Result | Raw capture SHA-256 |
|---|---:|---|
| `bun test src/adapters/local/localAdapter.test.ts -t 'shared CaseworkPort behavioral conformance'` (app cwd) | exit 0 | `fd86fba6e6b17618371a499d8839cc4af6858e9c9c3c2d246b5c31c5a106b3a4` |
| `bun test src/ui/journeys.test.tsx -t 'TOOTH: SSE drop flips the store to reconnecting via the live adapter error path, and an event resumes it'` (app cwd, default sandbox) | exit 1, expected sandbox `EPERM` | `83d76f0c597d2034fbcaad3ba93ad31def29e3d8198f73c2e8cd57bd85c34903` |
| Same exact listener test, bounded local-only escalated run | exit 0 | `7b1f99f79a7694e0109ede7df833812a16f9c8ae12566d666edadcd18bf81778` |

Default-sandbox preflight before the cursor run: available RAM 3,305,873,408 B; free swap 3,375,771,648 B; `/tmp` free 1,358,045,184 B. Before the default listener run: available RAM 2,518,765,568 B; free swap 3,983,413,248 B; `/tmp` free 1,358,016,512 B. Immediately before the escalated listener run, the same second preflight and hashes were freshly captured: available RAM 2,518,765,568 B; free swap 3,983,413,248 B; `/tmp` free 1,358,016,512 B. Bun was `1.4.0`; Just was `1.58.0`.

Frozen renderer sources remained unchanged:

| File | SHA-256 |
|---|---|
| `src/build/rendererChunkContract.ts` | `52cdf00b9650f0413c91e4375c0ce5b6527b63ad9f09219f2102383dca4dec64` |
| `vite.config.ts` | `95c287317aa4028a1976c145124f918d0d856a3bb0ca60802f31abd909e901ea` |
| `src/build/rendererChunkContract.test.ts` | `25b74d25318ce97659c91f3d734ca9b2cd794d0b4d90b1ec54c045b7ce022d08` |
| `package.json` | `85044b7b0619f390791f87d089a84463610814cc199f705662207f00337e2f54` |
| `bun.lock` | `9e3725c9ebd21566fb6499fe1df6f2fecf7230307b489dc6700ea768d7e39744` |
| `src/adapters/local/localAdapter.ts` | `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34` |
| `src/adapters/local/localAdapter.test.ts` | `07dbef23421395a251d6526febdab30b0ec51d0c87dc68f7a57d0de1c0fcc35b` |
| `src/adapters/conformance/caseworkPortConformance.ts` | `b2514a8ec08c077a9c250394a58ec68219ed1e8bef3f80c6f848a8fad39b3366` |
| `src/ui/journeys.test.tsx` | `c48391b88dd3e1649ebb83089d6e410cbd9b16c44330db9d2aac429bc696c441` |

The conformance and listener failures are separate from the build-only renderer chunk contract. The listener failure is attributable to the sandbox boundary in this run; the local conformance assertion has a plausible timing race and passed once in isolation, so its frequency remains unresolved. No code fix or gate acceptance is claimed. The full canonical gate must be adjudicated separately by the root operator.
