# T09 Unit5A cancellation and response-cap fixture: serialized independent approval

Date: 2026-10-05. Verdict: **APPROVED for the assigned Unit5A client cancellation prerequisite and the two bounded response-cap fixtures only.** This does not approve observation/SSE integration, broader T09, or unrelated Rust/workbench gates.

This final evidence record was created after the source review, four fresh gates, copy/hash audit, and serialization audit. It is immutable; corrections belong in a separately named supplement.

## Frozen identities and source review

The final reviewed source identities are:

| File | SHA-256 |
| --- | --- |
| `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` | `e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/client_cancellation_test.go` | `be8ad34bfe93306ede3fe1590b906c4ce2c4e9764c0db89af2f65e6cd7d0f91c` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/response_limit_test.go` | `dfb98f494880faf946ceaf3d0d10a139a20ed3260647112dd08dfc6372ccaa88` |

The cancellation fixture hash is the assigned fresh peer-read repair (`cancellation-peer-read-repair-oct05.md`); its recorded predecessor snapshot hash is `c4f1bd73f0f580b25d2367acaf5b331779d7bec078f08a17295cfdbe7314f43b`. The response test hash is a file SHA-256, not a Git object ID; it exactly matches the builder result.

Source review of the actual `client.go` confirms:

* `conn.call` holds the connection mutex for the whole exchange (`client.go:191-193`). Its `context.AfterFunc` callback closes only that checked-out `net.Conn` and signals completion (`:197-201`); it does not touch `conn.dead` or pool bookkeeping.
* The owner stops the callback, or waits for a callback already running, then marks the connection dead before `call` unlocks (`:203-215`). Canceled I/O becomes typed unavailable wrapping `ctx.Err()` (`:211-214`). The call uses the earlier of the request timeout and context deadline, and retains existing timeout/transport classification (`:216-249`). A valid complete response remains returnable even if the cancellation callback raced and retired that connection.
* `roundTrip` discards failed connections before returning; caller cancellation blocks mutation outcome recovery and transport retry (`:428-454`). Only the original mutation uses request-status correlation when its context remains live (`:433-437`). `Request.IsTransportRetrySafe` excludes mutations and Ask (`frame.go:86-99`). `Client.Do` retains the single bounded explicit server-busy retry (`client.go:395-417`). Ask remains record-writing and uncorrelated; no status query or automatic resend is introduced.
* Pool capacity is released on discard and healthy connections return only after `call` has completed; dead connections are not put back into idle storage (`client.go:302-370`). `Client.Close` behavior is not used to cancel an individual read (`:157-170`). The response-line ceiling and parsing/discard behavior are unchanged (`:38-40,71-78,236-249`). No public API, protocol, retry policy, dependency, identity, or kernel operation changed in these inputs.

The repaired race fixture changes only the retired peer's post-response read to direct `bufio.Reader.ReadString`, using the deadline already set by its initial request read. This makes the retirement branch depend on an actual read result instead of `SetReadDeadline` returning `io.ErrClosedPipe`; same-peer reuse, exact peer identity, dial count, response decode, and bounded cleanup assertions remain. The three frozen test hashes above match the independently reviewed files.

The response-cap repair is exactly 10 insertions and 4 deletions in the two assigned tests. Each has a local 1,024-byte cap applied before `New`; invalid JSON is 1,025 `x` bytes plus LF (1,026 total), so line overflow is detected before JSON decoding. The valid recovery JSON is 153 bytes plus LF (154). Both request/connection count assertions and the real correlated mutation outcome remain. The separate fragmented fixture still accepts the exact 32 MiB default boundary and rejects one byte over while poisoning the connection (`response_limit_test.go:199-227`). Shared `testConfig` deadlines and recovery budget are unchanged (`client_test.go:113-123`).

## Fresh sequential gates

All commands used `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-cancellation-oct05/gocache GOPATH=/tmp/sea-cancellation-oct05/gopath`. Every compiler command had a fresh immediately preceding host preflight, including date, MemTotal/MemAvailable/SwapTotal/SwapFree, and a full process `comm`/RSS scan for `go`, `cargo`, `rustc`, `compile`, and `bun`. Each preflight found only `bun` among those comm names (about 24 MiB RSS); no compiler overlap was observed. Every gate session was joined with its final exit before the next preflight/command. No retries were needed.

| Order | Preflight UTC | Available RAM; swap free | Command exit time (`-0400`) | Command and result |
| --- | --- | --- | --- | --- |
| 1 | 2026-10-05 17:49:38 | 2,415,252 kB; 1,053,728 kB | 13:50:03.261 | `cd apps/godspeed-casework-go && env [limits above] go test -race -count=3 -parallel=1 ./internal/adapters/sfwp -run '^(TestInspectOverLimitResponseRetriesOnceOnFreshConnection|TestOverLimitMutationAndRecoveryResponsesNeverResendMutation|TestManualCancellationInterruptsRunGetAskAndMutationWithoutRetry|TestCancelingOnePoolRequestLeavesConcurrentRequestUsable|TestCanceledPartialRunGetConnectionIsNotReused|TestCancelAfterCompletedMutationPreservesAndReusesItsConnection|TestRunGetCancellationRacesResponseAndPoolReturn)$'` — exit 0; package `ok ... 1.111s`. |
| 2 | 2026-10-05 17:50:18 | 2,566,024 kB; 1,008,544 kB | 13:50:44.607 | `cd apps/godspeed-casework-go && env [limits above] go test -race -count=1 -parallel=1 ./internal/adapters/sfwp` — exit 0; package `ok ... 9.222s`. |
| 3 | 2026-10-05 17:51:05 | 2,601,276 kB; 948,780 kB | 13:51:30.808 | `env [limits above] just casework-go-check` — exit 0; format, vet and package tests green. |
| 4 | 2026-10-05 17:51:44 | 2,595,728 kB; 950,272 kB | 13:53:12.138 | `cd apps/godspeed-casework-go && env [limits above] go test -race -count=1 -parallel=1 ./...` — exit 0; all module packages pass, including SFWP `13.910s`, auth `34.456s`, and server `12.996s`. |

The exit times are the actual exit-capture file mtimes. In each case the exit precedes the next preflight: gate 1 exit 13:50:03.261 < gate 2 preflight 13:50:18.658; gate 2 exit 13:50:44.607 < gate 3 preflight 13:51:05.090; gate 3 exit 13:51:30.808 < gate 4 preflight 13:51:44.822. The final command exited at 13:53:12.138. Dates in the preflight outputs are UTC; capture mtimes use host `-0400` time.

### Exact captures

The 16 `/tmp` originals are under `/tmp/sea-cancellation-oct05/final-independent-gates-oct05/`. Each matching raw/exit file was copied to `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/final-cancellation-fixed-fixtures-oct05/` with the identical basename and `cmp` verified byte equality:

* `gate1-preflight.raw/.exit` and `gate1-focused.raw/.exit`
* `gate2-preflight.raw/.exit` and `gate2-sfwp.raw/.exit`
* `gate3-preflight.raw/.exit` and `gate3-canonical.raw/.exit`
* `gate4-preflight.raw/.exit` and `gate4-module.raw/.exit`

All four command exit files contain `0`. The command raw SHA-256s are: gate1 `9a6af8ce22050988226b5575470aa18f2ff51bd21e92edff0ac04602969e62a8`; gate2 `066c1bbe2483faa58e6d4a6618c1ab6ec4d317747b1b93afe8f916ff285b14ae`; gate3 `156bf521893f26764840d7dc0dac7c51c709cea67544b48e0c583353917c1555`; gate4 `71fe7d0d3d2ed9acf96c78338a1c79e97332d207eae90d85ca12a334466b7254`. All exit files have SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`. The four preflight raw files are individually retained and compare-equal; their repository copies have the corresponding `gateN-preflight` names.

## Original instructions and material history

The original test-first requirement is recorded in `cancellation-fixture-independent-review-oct05.md`: on the then-frozen baseline client hash `8cdfc52a68c07df84e1f88506163da1a8dab212004d45689a4c8ab8007b3b61a` and pre-repair fixture hash `c4f1bd73f0f580b25d2367acaf5b331779d7bec078f08a17295cfdbe7314f43b`, the five in-flight cancellation assertions compiled and failed as expected, with no compile/infrastructure failure. That review's full-package command was broader than a filtered test-name command, as it disclosed. The runtime builder's first focused attempt later failed the five cancellation assertions because cancellation errors did not preserve `context.Canceled`; its subsequent source was the `e0d3…` identity reviewed here. Its final focused attempt had one remaining race-peer EOF assertion failure, which was fixture proof, not evidence against connection retirement; the `be8ad…` direct-read repair addressed that precise issue. These earlier events remain history and are not counted as the fresh gates above.

The earlier independent gate set is not relied on for this approval: `cancellation-gate-evidence-root-findings-oct05.md` records that its canonical retry overlapped the first canonical command, so those otherwise-zero exits were procedurally insufficient. I independently compared all 10 historical `rootcritic` raw captures and all 10 matching exit captures against their `/tmp/sea-cancellation-oct05/` originals; all 20 matched. The separately copied 12-file historical independent-gate subdirectory has 10 byte mismatches and is non-authoritative. Those copies and their original artifacts remain untouched. The root findings also correct a prior builder-preflight statement: a `preflight-2` process listing exists but contains no host RAM measurement; no missing RAM value is inferred. This review's own four adjacent preflights each include host RAM and process comm/RSS evidence.

The response-cap fixtures originally failed under broader race testing at the 32 MiB response sizes. The assigned narrow test-only repair keeps default-boundary coverage in its dedicated fixture and lowers only these two retry/recovery fixtures to 1,024 bytes. No source/test timeout, cap, recovery behavior, request, connection, or response assertion was loosened. All four freshly run gates passed; no failure was retried or edited around.

## Approval boundary

I approve the `client.go` cancellation lifetime repair and the frozen cancellation fixture, including the direct-read peer repair, for the client prerequisite described by the original test-first/runtime assignments. I also approve the two bounded response-cap fixture changes. This does not authorize or approve broader observation cohort/poller/SSE work, full T09 completion, the separate Rust checkpoint, or any wider integration claim. No code, test, status, debt, or Git changes were made during this independent review; only the new capture directory and this final evidence record were added.

Graft was used to retrieve repository/source context before exact source inspection. Graft reported approximately 153,071 tokens saved this turn, worth about $0.12 at its displayed rates.
