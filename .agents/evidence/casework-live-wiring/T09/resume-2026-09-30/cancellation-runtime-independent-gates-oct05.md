# T09 Unit5A cancellation runtime independent gates — APPROVED

Date: 2026-10-05

## Identity and decision

Independent source readiness is recorded in
`cancellation-runtime-source-readiness-oct05.md`. The verified frozen inputs
were:

- Fixture `client_cancellation_test.go`, SHA-256
  `be8ad34bfe93306ede3fe1590b906c4ce2c4e9764c0db89af2f65e6cd7d0f91c`.
- Production `client.go`, SHA-256
  `e0d3c12c1af75b35c041889a45db7da3978db26bb0226e38149b07ef0f885efe`.
- Prior frozen fixture snapshot SHA-256
  `c4f1bd73f0f580b25d2367acaf5b331779d7bec078f08a17295cfdbe7314f43b`.

The focused cancellation tests, full SFWP race package, canonical
`just casework-go-check`, and full-module race/count-one/parallel-one gate all
passed. I approve this client cancellation prerequisite and its fixture for the
assigned scope. This does not approve broader Unit5A/T09 integration or settle
the unrelated root Rust gate failure.

## Commands and evidence

All Go commands used
`GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0`, with
`GOCACHE=/tmp/sea-cancellation-oct05/gocache` and
`GOPATH=/tmp/sea-cancellation-oct05/gopath`. Host RAM/process preflight was
captured immediately before every compiler command. Each `/tmp` raw/exit and
preflight file below was copied unchanged into the repository evidence
directory; shell `cmp` verified every copy.

1. Focused cancellation race, all five fixture tests:

   `cd apps/godspeed-casework-go && env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-cancellation-oct05/gocache GOPATH=/tmp/sea-cancellation-oct05/gopath go test -race -count=1 -parallel=1 ./internal/adapters/sfwp -run '^(TestManualCancellationInterruptsRunGetAskAndMutationWithoutRetry|TestCancelingOnePoolRequestLeavesConcurrentRequestUsable|TestCanceledPartialRunGetConnectionIsNotReused|TestCancelAfterCompletedMutationPreservesAndReusesItsConnection|TestRunGetCancellationRacesResponseAndPoolReturn)$'`

   Exit 0, package reported `ok ... 1.019s`. Exact captures:
   `rootcritic-preflight-focused.raw/.exit`,
   `rootcritic-focused.raw/.exit`.

2. Full SFWP package race:

   `cd apps/godspeed-casework-go && env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-cancellation-oct05/gocache GOPATH=/tmp/sea-cancellation-oct05/gopath go test -race -count=1 -parallel=1 ./internal/adapters/sfwp`

   Exit 0, package reported `ok ... 17.126s`. Exact captures:
   `rootcritic-preflight-sfwp.raw/.exit`,
   `rootcritic-sfwp-race.raw/.exit`.

3. Canonical module check:

   `env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-cancellation-oct05/gocache GOPATH=/tmp/sea-cancellation-oct05/gopath just casework-go-check`

   Both invocations exited 0 and ended with
   `casework-go-check: format, vet and tests green`. First invocation downloaded
   uncached modules; its nested tool call returned before the session result was
   visible, so I began a second invocation. The first session later completed
   and its raw and exit 0 were captured. The second invocation is a redundant
   confirmation and used cached results where Go allowed. Exact captures:
   first `rootcritic-preflight-canonical.raw/.exit`,
   `rootcritic-canonical-attempt1.raw`, `rootcritic-canonical.exit`; retry
   `rootcritic-preflight-canonical-retry.raw/.exit`,
   `rootcritic-canonical-retry.raw/.exit`.

4. Full Go module race, fresh test execution with `-count=1` and serialized
   package parallelism:

   `cd apps/godspeed-casework-go && env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-cancellation-oct05/gocache GOPATH=/tmp/sea-cancellation-oct05/gopath go test -race -count=1 -parallel=1 ./...`

   Exit 0. Every module package passed; SFWP reported `ok ... 17.954s`, auth
   `33.426s`, and server `13.612s`. `-count=1` disables test-result reuse,
   including any golden-sensitive test caching. Exact captures:
   `rootcritic-preflight-module-race.raw/.exit`,
   `rootcritic-module-race.raw/.exit`.

## Failures and deviations retained

- The prior source review rejected the race fixture because its second helper
  could mistake `SetReadDeadline`'s `io.ErrClosedPipe` for failed EOF proof.
  Fresh fixture repair changes only the race peer's second read to a direct
  buffered `ReadString`; the initial request read had already armed the
  deadline before `raceStart`. It requires actual `Read` EOF and retains exact
  peer identity, dial count, decoded response, cleanup, and strict completed-call
  reuse assertions. The repair and rationale are in
  `cancellation-peer-read-repair-oct05.md`.
- Historical expected-RED evidence remains unchanged: the pre-repair
  cancellation fixture's five in-flight cancellation cases failed as expected;
  the first production builder run also failed those five cases because its
  returned error did not preserve `context.Canceled`. The final builder run on
  the production source reviewed here had only the invalid race-peer EOF
  assertion failure. The repaired fixture now passes all focused gates.
- The first canonical command was unnecessarily repeated because the wrapper
  did not initially return its session identifier/output before the first
  check. Both invocations completed successfully; both raw/exit sets are kept.
- The previously retained builder preflight
  `unit5a-oct05builder-preflight-3.raw` (16:41:38 UTC) showed 2.8 GiB available
  and a 1.4 GiB `.cline` process alongside several Node/Codex/Claude processes.
  The archived builder preflight sequence has no `preflight-2` artifact, so I
  cannot certify that every earlier builder compile had an adjacent host
  preflight. This review captured a fresh host preflight directly before each
  of its own compiler commands. Its first focused preflight recorded 3.0 GiB
  available; later preflights recorded 3.0, 3.0, 2.6, and 2.7 GiB available.
- Root separately reported a Rust clippy/test-expression push-gate failure.
  No Rust command was run by this reviewer; that failure is outside the
  Go-client prerequisite approved here.
- Initial hand-transcribed evidence files with the `unit5a-oct05critic-`
  prefix are not the raw authority. The exact `/tmp` captures were separately
  copied with `rootcritic-` names and verified with `cmp`; use those files for
  audit. The first canonical command has raw and exit 0; no exit was inferred.

## Token return

The sole Go compiler token is returned to root. No Go source or fixture changes
were made during independent gate execution. No source, tests, Git, status, or
debt files were edited by this reviewer; only new evidence artifacts were
added.
