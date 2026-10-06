# Physical run.get admission — independent runtime verification

Date: 2026-10-06  
Verdict: **APPROVE the bounded physical-admission production unit for its assigned runtime gates.**  
This approval is limited to the implementation and proof below. It does not settle T09 or claim live/browser/CI/publication success.

## Reviewed source identity

Source approval is recorded in `physical-admission-production-independent-rereview-oct06.md`. The final hashes remained unchanged before and after each accepted gate:

| Path | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` | `ad3d599224527beda603a320b1b86faa82b80c75480d914cd805fa1e4de3a857` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_get_admission.go` | `af040128d0d0288a2495d63bdb959a14edbf4ca8d5a2d8e29954193bb4e5fc6f` |
| `apps/godspeed-casework-go/cmd/godspeed-casework/main.go` | `dcc92dd73a951d3053ad907d66bda6caaf03f39b22fa33029ac81598349ddeed` |
| frozen limiter fixture | `dae0406d503e16546b24c507b02db63489dd8b06cd381b34fd045326d1983d58` |
| frozen client fixture | `becd4266dca4a28e7641f5eaab1dafe5dc932e04723895cf5e795e7671524103` |

The implementation conforms to both original assignments and the root compatibility adjudication: a fixed two-record/one-second limiter, no-spin blocking wait rules, a new permit for every physical `run_get` retry, finish time at the last attempted payload/LF write before response reading, callback join and pool cleanup before permit release, distinct retry failure phases, one shared owner at the sole production client-construction site, and coverage of trace and artifact-provenance reads. The final permit-gated pre-write context check preserves nil-permit Ask/mutation/other-verb behavior. The separate source rereview records the exact code anchors and empty-ID/timer/finish-fallback review.

## Accepted commands and evidence

Each accepted command ran serially under the assigned caps:

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1
```

Every preflight exceeded the minimum 1200 MiB available RAM / 512 MiB free swap, found the assigned Go cache, and recorded the five hashes above. Process snapshots showed only `codex`, `bash`, and `ps` in this execution namespace; this is not a claim about host-global processes. Every accepted preflight/raw/exit file was copied with native `apply_patch` and `cmp` verified byte equality against its original `/tmp` capture.

1. **Focused nine-group race test — exit 0.** Session `70862` joined. Command: `go test -race -count=1 -parallel=1 -timeout=90s -v ./internal/adapters/sfwp -run '^(TestRunGetLimiterContract|TestClientUsesAdmissionOnlyForPhysicalRunGetAttempts|TestClientAdmissionCountsAllFourRunGetRetryAttempts|TestClientDoesNotRouteAskOrMutationThroughRunGetAdmission|TestMutationTransportFailureIsNotResentThroughAdmission|TestClientAdmissionMarksPayloadAndLFWriteBoundary|TestClientAdmissionFinishesAtFinalAttemptedWriteOnPayloadAndLFFailure|TestClientWaitsForBlockedPayloadAndLFAttemptBeforeFinish|TestClientAdmissionReleaseFollowsCancellationAndConnectionRetirement)$'`. All nine top-level functions reached and passed. The same-run busy/no-spin subtest passed, including cancellation and real permit-release/reacquire. The healthy pool-at-release assertion passed with live=1/idle=1 and the expected returned connection; the canceled path passed with live=0/idle=0 after connection retirement. Partial payload, LF write failure, blocked payload/LF, all four retry attempts, Ask/mutation exclusion, and cancellation callback ordering passed. Captures: `physical-admission-focused-repeat-oct06.{preflight.raw,preflight.exit,raw,exit}`.
2. **Full SFWP package race — exit 0.** Session `14903` joined; output: `ok .../internal/adapters/sfwp 21.025s`. Command: `go test -race -count=1 -parallel=1 -timeout=180s ./internal/adapters/sfwp`. Captures: `physical-admission-sfwp-race-oct06.{preflight.raw,preflight.exit,raw,exit}`.
3. **Full Go module race — exit 0.** Session `38277` joined. Command: `go test -race -count=1 -parallel=1 -timeout=300s ./...`. Every module package with tests passed, including `sfwp`, `auth`, and `server`; command output contained no race report or failure. Captures: `physical-admission-module-race-oct06.{preflight.raw,preflight.exit,raw,exit}`.
4. **Canonical Go gate — exit 0.** Session `79397` joined. Command: `just casework-go-check`. Output: `casework-go-check: format, vet and tests green`; the adapter tests ran, other already-valid packages were reported cached. Captures: `physical-admission-casework-go-check-oct06.{preflight.raw,preflight.exit,raw,exit}`.

## Preserved first attempt and limits

The first focused attempt also joined exit 0 and reached the same passing test groups, but its immediately preceding preflight was not durably captured. Its raw/exit were preserved byte-exact in `physical-admission-focused-initial-oct06.raw` and `.exit`, with the evidence gap recorded in `physical-admission-focused-initial-capture-gap-oct06.md`. That attempt is not counted as gate evidence; root authorized and the review executed the exact repeat above with complete byte-checked captures.

No compiler/scanner/Graft build was run beyond the four assigned commands and the separate read-only `gofmt -d` source check. These gates establish the scoped adapter behavior under the frozen fixtures and Go module suite. They do not establish manager-level logical cohort budgeting, end-to-end T09 hydration, SSE/UI, live browser behavior, settlement, CI, or publication. The separate limiter tests are controlled concurrency evidence, not proof of every possible scheduler interleaving or production load profile.
