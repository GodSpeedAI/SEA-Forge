# Unit5A cancellation independent runtime review — NOT APPROVED

Date: 2026-10-05

## Decision

The source and fixture review found no blocker in the assigned cancellation repair, and the focused five-test race selection passed. The assigned prerequisite is **not approved** because the full SFWP race gate and full-module race gate both failed response-limit mutation-recovery coverage. These failures must be resolved before approval. This review did not retry either gate or make code/test/status/Git/debt changes.

## Frozen inputs and exact diff

- `apps/godspeed-casework-go/internal/adapters/sfwp/client.go`: SHA-256 `e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe`.
- `apps/godspeed-casework-go/internal/adapters/sfwp/client_cancellation_test.go`: SHA-256 `be8ad34bfe93306ede3fe1590b906c4ce2c4e9764c0db89af2f65e6cd7d0f91c`.
- Frozen pre-repair fixture snapshot: SHA-256 `c4f1bd73f0f580b25d2367acaf5b331779d7bec078f08a17295cfdbe7314f43b`.

The exact worktree diff against the tracked baseline is limited to two files. `client.go` changes only `conn.call`: it registers a context `AfterFunc` that closes this checked-out `net.Conn`, joins a started callback before releasing the connection, marks the connection dead while still owning `cn.mu`, and wraps interrupted I/O as unavailable with `ctx.Err()` preserved. The fixture changes only the response-race peer's second read, from `readCancellationRequest` to direct `bufio.Reader.ReadString`, relying on the four-second read deadline armed by the initial request read. The source diff was inspected directly; there were no other Go source or fixture changes in scope.

## Source and fixture audit

I read the original test-first assignment, its critique supplement, runtime-root assignment, root next-unit clarifications, fresh `cancellation-peer-read-repair-oct05.md`, and `cancellation-runtime-source-readiness-oct05.md`; I independently opened the actual source and all of `client_cancellation_test.go`. The prior `cancellation-runtime-independent-gates-oct05.md` was treated as historical only and did not supply evidence for this review.

- Callback scope is the checked-out connection only. The callback does not touch `conn.dead` or pool state. The owner waits for callback completion and sets `dead` before unlocking `cn.mu`; `roundTrip` cannot release the connection until `call` returns.
- A cancellation-interrupted I/O error is typed unavailable and retains `context.Canceled` in its chain. A valid response remains returnable if cancellation races response receipt, while callback execution still retires the connection.
- Pool release discards a dead connection. Existing read-only retry and correlated-mutation recovery require `ctx.Err() == nil`; Ask's ambiguous failure remains no-resend/no-status. The reviewed code does not alter the existing response cap, positional pairing, or explicit busy retry behavior.
- The fixture synchronizes actual request receipt, applies a five-second operation timeout and one-second cancellation bound, checks `run_get`, Ask, and correlated mutation for one send/no recovery, observes peer EOF, preserves a second checked-out connection, discards a partial response connection, checks completed mutation response and same-peer reuse after later cancellation, and races cancellation against response/pool return with exact peer identity and bounded worker cleanup.
- The peer-read repair's deviation from an extra post-response observer is acceptable: the initial peer read arms its deadline before `raceStart`; direct `ReadString` then observes actual EOF or the next request line without attempting `SetReadDeadline` on a possibly closed pipe.

The accepted builder/source-readiness records disclose a prior missing adjacent preflight artifact and earlier hand-transcribed evidence copies. I did not use those historical copies or prior gate claims. This run captured fresh preflights and command outputs; each newly copied evidence file was byte-compared to its `/tmp` original. The canonical recipe's Go package output included `(cached)`; the separate fresh `-count=1` race gates below are the runtime evidence.

## Independent commands and results

Every Go command used `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0`, `GOCACHE=/tmp/sea-cancellation-oct05/gocache`, and `GOPATH=/tmp/sea-cancellation-oct05/gopath`. Each had a fresh actual-host preflight immediately before it. Preflights contain UTC time, MemTotal/MemAvailable/SwapTotal/SwapFree, and the full `ps -eo pid,comm` process list; no command-line arguments or process environments were collected. MemAvailable was 2,695,396 kB before focused, 2,697,300 kB before SFWP, 2,649,816 kB before canonical, and 2,679,160 kB before module. None showed a `go`, `cargo`, `rustc`, or `compile` process; other existing process names, including `.cline`, `bun`, and `node`, are retained in the comm-only captures.

1. Focused cancellation race, five named test functions, race detector, fresh execution:

   `cd apps/godspeed-casework-go && env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-cancellation-oct05/gocache GOPATH=/tmp/sea-cancellation-oct05/gopath go test -race -count=1 -parallel=1 ./internal/adapters/sfwp -run '^(TestManualCancellationInterruptsRunGetAskAndMutationWithoutRetry|TestCancelingOnePoolRequestLeavesConcurrentRequestUsable|TestCanceledPartialRunGetConnectionIsNotReused|TestCancelAfterCompletedMutationPreservesAndReusesItsConnection|TestRunGetCancellationRacesResponseAndPoolReturn)$'`

   **PASS**, package `ok ... 1.022s`, shell exit 0. Evidence: `rootcritic-focused-preflight.raw`, `rootcritic-focused.raw`, `rootcritic-focused.exit`.

2. Full SFWP package race, fresh execution:

   `cd apps/godspeed-casework-go && env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-cancellation-oct05/gocache GOPATH=/tmp/sea-cancellation-oct05/gopath go test -race -count=1 -parallel=1 ./internal/adapters/sfwp`

   **FAIL**, exit 1. `TestInspectOverLimitResponseRetriesOnceOnFreshConnection` failed after 3.11s and `TestOverLimitMutationAndRecoveryResponsesNeverResendMutation` failed after 3.74s; each returned `unavailable: deadline exceeded while the request was in flight`. Evidence: `rootcritic-sfwp-preflight.raw`, `rootcritic-sfwp.raw`, `rootcritic-sfwp.exit`.

3. Canonical root recipe:

   `env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-cancellation-oct05/gocache GOPATH=/tmp/sea-cancellation-oct05/gopath just casework-go-check`

   **PASS**, `casework-go-check: format, vet and tests green`, shell exit 0. Its SFWP test output was `(cached)`, so this does not override the fresh race failures. Evidence: `rootcritic-canonical-preflight.raw`, `rootcritic-canonical.raw`, `rootcritic-canonical.exit`.

4. Full Go module race, `-count=1`, serialized package parallelism:

   `cd apps/godspeed-casework-go && env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-cancellation-oct05/gocache GOPATH=/tmp/sea-cancellation-oct05/gopath go test -race -count=1 -parallel=1 ./...`

   **FAIL**, Go test exit 1 (the capture wrapper itself exited 0 after writing the test exit). `TestOverLimitMutationAndRecoveryResponsesNeverResendMutation` failed after 2.40s with the same deadline-exceeded result. Other listed packages passed. Evidence: `rootcritic-module-preflight.raw`, `rootcritic-module.raw`, `rootcritic-module.exit`.

The full SFWP and module processes were each polled to final process exit before any next compiler command. No test was rerun after failure, and no exit was inferred from missing output. A fresh preflight and exact raw/exit captures exist for all four gates. The 12 repository evidence copies were made from dynamic `cat` output using `apply_patch`; `cmp` succeeded for every original/copy pair. Copy SHA-256 values are the matching values recorded by the files and `/tmp` originals.

## Root evidence findings and timeout anchors

I read `cancellation-gate-evidence-root-findings-oct05.md`; its SHA-256 is `d4d7f143e517cbbf38622a489fbb274d22b8e92391dbfbad260c79c3d5f6e426`. Its ordering finding is historical: it reports the prior canonical retry began before the first invocation's captured exit, so those earlier gate claims were procedurally insufficient. My four commands above were sequential; I waited for each `exec_command` session's final process exit before taking the next preflight.

I compared the complete 10 prior archived raw files and all 10 paired exit files byte-for-byte to their `/tmp/sea-cancellation-oct05/` originals. All 20 pairs matched, including the archives that the root findings document lists only in its 10-row raw comparison. This confirms archival identity; it does not rehabilitate the prior overlapping execution. The new captures for this independent run are under their own `independent-cancellation-oct05/rootcritic-*` names and were separately compared to their new `/tmp/sea-cancellation-rootcritic-oct05-*` originals.

The fresh failure anchors are `response_limit_test.go:249-281` (`TestInspectOverLimitResponseRetriesOnceOnFreshConnection`) and `response_limit_test.go:284-330` (`TestOverLimitMutationAndRecoveryResponsesNeverResendMutation`), with assertion sites at `:273-280` and `:318-330`. Shared test configuration is `client_test.go:113-123`: `RequestTimeout` is 2 seconds, `RecoveryBudget` 3 seconds, and backoff base 20 ms. Production applies the per-round-trip timeout in `client.go:419-428`, then conditionally retries safe inspect or starts correlated recovery in `:432-449`; recovery applies its separate budget and requests status at `:465-476`. The captured failures are consistent with deadline expiration during those existing retry/recovery paths; I do not attribute them to this cancellation patch without further diagnosis.

## Next required action

Investigate why response-limit inspect retry and mutation correlation recovery exceed their configured operation deadline under the fresh race runs. Preserve their assertions and the cancellation fixture. After a scoped repair or substantiated diagnosis, obtain a new independent approval run with new immutable paths; this report does not approve broader Unit5A/T09 integration.
