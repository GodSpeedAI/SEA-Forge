# Corrected serialized independent approval: cancellation and fixtures

Date: 2026-10-05. **APPROVED for the assigned Unit5A client cancellation prerequisite, its repaired cancellation fixture, and the two bounded response-cap fixture edits only.** This is the final corrected approval record. Use it with `cancellation-independent-gate-copy-mapping-erratum-oct05.md` and `cancellation-serialized-independent-approval-oct05.md`. The first approval is immutable but contained an incorrect statement about the prior 12-file archive; this record and the erratum correct that mapping. The independently rerun gate evidence itself is unchanged and exact.

## Inputs and review

Reviewed the original cancellation test-first/runtime assignments, fixture critique supplement, root timeout/join clarifications, peer-read repair, response-cap repair assignment/builder record, and actual source. The final full-file SHA-256 identities are:

| File | SHA-256 |
| --- | --- |
| `client.go` | `e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe` |
| `client_cancellation_test.go` | `be8ad34bfe93306ede3fe1590b906c4ce2c4e9764c0db89af2f65e6cd7d0f91c` |
| `response_limit_test.go` | `dfb98f494880faf946ceaf3d0d10a139a20ed3260647112dd08dfc6372ccaa88` |

Source review confirms `conn.call` holds the checked-out connection mutex; its context callback only closes that network connection and signals done. The call owner stops or joins the callback, marks the connection dead, and only then returns through pool release/discard. Canceled I/O is typed unavailable and preserves `ctx.Err()`; the earlier request/context deadline and existing error classifications remain. A complete valid response can be returned through the race while that connection is retired. `roundTrip` discards failed connections and refuses retry or correlated outcome recovery when the caller context is canceled. Ask remains no-resend/no-status, mutations retain request-ID status recovery only with a live context, and explicit server-busy retry remains bounded. No API, dependency, protocol, identity, kernel operation, response cap, or retry-policy change is present.

The cancellation fixture has five synchronized cases: manual cancellation for run_get/Ask/correlated mutation, another checked-out pool connection surviving cancellation, no reuse of a partial-read connection, cancellation after successful completed mutation preserving same-connection reuse, and response/cancellation/pool-return race. The peer repair changes only the race peer's retirement read to direct `ReadString`, relying on the deadline armed by its initial request read; this requires actual read EOF rather than a `SetReadDeadline` error. The response-cap fixture diff is limited to the two assigned tests (10 insertions, 4 deletions): a pre-`New` local 1,024-byte cap, a 1,026-byte invalid-JSON line including LF, and the unchanged 154-byte valid recovery line. The dedicated fixture still tests exact 32 MiB acceptance and one-byte-over poisoning. Request/connection counts, returned outcomes, timeouts, recovery budgets, shared config and all behavioral assertions remain.

## Fresh gates and exact evidence

All four commands used the same environment limits and cache listed in `cancellation-serialized-independent-approval-oct05.md`. Before each command, a fresh actual-host preflight recorded date, `/proc/meminfo` RAM/swap fields and a full `ps -e -o comm=,rss=` scan for `go`, `cargo`, `rustc`, `compile`, and `bun`. Only Bun appeared, at roughly 24 MiB RSS. Each process was joined with its final exit before the next preflight. No command was retried.

1. Focused `go test -race -count=3 -parallel=1 ./internal/adapters/sfwp -run '^(TestInspectOverLimitResponseRetriesOnceOnFreshConnection|TestOverLimitMutationAndRecoveryResponsesNeverResendMutation|TestManualCancellationInterruptsRunGetAskAndMutationWithoutRetry|TestCancelingOnePoolRequestLeavesConcurrentRequestUsable|TestCanceledPartialRunGetConnectionIsNotReused|TestCancelAfterCompletedMutationPreservesAndReusesItsConnection|TestRunGetCancellationRacesResponseAndPoolReturn)$'` — exit 0; package 1.111s.
2. Full SFWP `go test -race -count=1 -parallel=1 ./internal/adapters/sfwp` — exit 0; package 9.222s.
3. Canonical `just casework-go-check` — exit 0; format, vet, and package tests green.
4. Fresh module `go test -race -count=1 -parallel=1 ./...` — exit 0; all packages pass.

For these commands, the actual originals are `/tmp/sea-cancellation-oct05/final-independent-gates-oct05/gate1-focused.raw/.exit`, `gate2-sfwp.raw/.exit`, `gate3-canonical.raw/.exit`, and `gate4-module.raw/.exit`. The adjacent `gateN-preflight.raw/.exit` files are the four actual-host preflights. All 16 files were copied with identical basenames to `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/final-cancellation-fixed-fixtures-oct05/` and individually `cmp`-verified.

Capture exit mtimes establish serialized ordering: gate 1 exited at 13:50:03.261 -0400 before gate 2's 13:50:18.658 preflight; gate 2 exited 13:50:44.607 before gate 3's 13:51:05.090 preflight; gate 3 exited 13:51:30.808 before gate 4's 13:51:44.822 preflight; gate 4 exited 13:53:12.138. Available RAM at the four preflights was 2,415,252 kB, 2,566,024 kB, 2,601,276 kB, and 2,595,728 kB respectively. Every command exit capture is 0.

## Corrected historical-copy audit and deviations

The mapping erratum establishes that all 12 files under the prior `independent-cancellation-oct05/` archive are byte-exact against their correct `/tmp/sea-cancellation-rootcritic-oct05-<basename-without-rootcritic->` originals. The first approval's 10-mismatch statement used a different `/tmp` run and is withdrawn by this corrected record. Separately, the 20 older `rootcritic` raw/exit artifacts match their `/tmp/sea-cancellation-oct05/rootcritic-*` originals. Those earlier gate runs remain historically insufficient because their canonical retry overlapped the original canonical command; the present approval rests on the four fresh serialized gates above. Older hand-transcribed `unit5a-oct05critic-*` artifacts and the process-only builder preflight with no host RAM value are historical limitations, not inputs to this approval. The later original builder focused failure was the race-peer EOF proof defect fixed by the one-read fixture repair; no code/source failure is inferred from it. The recorded intended baseline expected-RED occurred against the then-frozen baseline client/fixture before implementation; it is historical evidence, separate from the fresh green gates.

No source, test, status, debt, or Git changes were made during this independent approval run. The only additions are the exact gate captures and approval/erratum evidence. This approval does not represent broader T09 completion or release any observation, SSE, Rust, or UI work.
