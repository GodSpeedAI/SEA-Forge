# Present-context Phase 1 focused RED — runtime/source boundary review

Date: 2026-10-06  
Verdict: **expected semantic RED against the deliberately unavailable stub; fixture assertions reached as listed below.** This records one focused run only. It is not algorithm, production, manager, caller, or public-readiness approval.

## Frozen source and command

- Source: `apps/godspeed-casework-go/internal/server/run_observation_present_context.go`, SHA-256 `565ad7a604bf68ab7667c901eaa23928d28476150fdfa82da9e170ba9e61ee86`.
- Fixture: `apps/godspeed-casework-go/internal/server/run_observation_present_context_test.go`, SHA-256 `fffa033505fa19b937da12963df91a0cdc2af30f39ecb50031fc2a17ec64a67b`.
- Working directory: `/home/sprime01/projects/sea-rs/apps/godspeed-casework-go`.
- Actual command: `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 go test -race -count=1 -parallel=1 -run '^TestCheckRunObservationPresentContext' -v ./internal/server`.
- Actual process session 13235 was joined. The capture wrapper completed with tool exit 0 and separately recorded Go test exit 1. The captured output shows package compilation and test execution, then `FAIL`; it contains no setup/build error or race report. No retry or additional gate was run.

## Resource preflight and immutable capture identity

The preflight was recorded at `2026-10-06T19:25:11Z`, immediately before launch. It measured `MemAvailable=2,203,516,928` bytes and `SwapFree=742,133,760` bytes, above the assigned 1,200 MiB RAM and 512 MiB swap floors. It recorded the frozen source/fixture hashes and `/tmp`/cache capacity.

Original captures in `/tmp`:

- `/tmp/sea-casework-20261006-present-context-red-01-preflight.raw`: SHA-256 `6b333c4bf819ef8cbb5787f7c97c8b0085026469df22b6436849c42a93db1b16`.
- `/tmp/sea-casework-20261006-present-context-red-01-run.raw`: SHA-256 `b6dd85e7ccebada186ca192817344a0789bd853a0b0c8bbd538037ce10582a85`.
- `/tmp/sea-casework-20261006-present-context-red-01-exit.raw`: SHA-256 `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22` (`exit=1`).

Repository archives `present-context-red-01-preflight.raw` and `present-context-red-01-exit.raw` compare byte-for-byte with their `/tmp` originals and have the same hashes. The first native archive `present-context-red-01-run.raw` is **not** byte-identical: its only difference is that the `nil_relay` and `empty_history` PASS lines under `RejectsUnavailableInputs` are transposed. Its SHA-256 is `0663942de34d8e705e8f3944778c84be5807c54bbbfc46864f8c5e60e83841e8`; preserve it as a transcription-error artifact, not as the actual run capture. The original `/tmp` raw remains the authority for output and ordering. Automatic review rejected a manually reconstructed “exact” replacement because it could misrepresent immutable evidence; no replacement archive was created. Root must compare the `/tmp` run capture directly if repository-local byte identity is required.

## Test boundaries reached

Seven focused top-level tests started:

- `RejectsUnavailableInputs` passed, including blank case, nil history, nil relay, and empty history. These checks match the stub's unconditional typed-unavailable zero result and do not prove input-specific validation.
- `RejectsEachNewestIdentityDefect` ran all 13 cases; each failed at the exact-history-call assertion because the stub makes no history call. The unavailable/zero-result assertions ran before that assertion. Relay-argument assertions were not reached.
- `UsesOnlyNewestRevisionWithoutFallback` failed at its exact-history-call assertion after unavailable/zero-result checks. No fallback behavior was executed by the stub.
- `RejectsRelayCursorGaps` ran all four cases; each failed at the history-call assertion. The subsequent relay-call assertion was not reached.
- `AcceptsAndCopiesEmptyAndPopulatedHorizons` ran both empty and populated success subtests; each failed immediately because the stub returned typed unavailable where success was expected. Exact output, input immutability, returned-set/source independence, and repeated-result assertions were not reached.
- `RechecksAdvancingHistoryAndDoesNotReuseOldParents` failed at the initial success assertion. Relay advancement, a later matching retained row, removed-parent handling, and result independence were not reached.
- `RejectsWrongCaseFromHistory` failed at the exact-history-call assertion after unavailable/zero-result checks. It does not require an unnecessary Relay read after invalid history.

There is no evidence here for the eventual algorithm's validation, ownership, no-fallback, relay-gap, progression, or copy properties. The failure is consistent with the frozen always-unavailable stub. This run releases no source implementation or caller work; any later production review and runtime gates require root assignment. No full package/full module/canonical gate, scanner, Graft build, Git operation, or external action was performed.
