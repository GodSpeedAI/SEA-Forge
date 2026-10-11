# T09 Unit5A cancellation fixture independent review — source APPROVE, expected RED observed

Date: 2026-10-05

## Identity and scope

Source review was completed before compilation. The fixture
`apps/godspeed-casework-go/internal/adapters/sfwp/client_cancellation_test.go`
matches SHA-256 `c4f1bd73f0f580b25d2367acaf5b331779d7bec078f08a17295cfdbe7314f43b`.
Production `client.go` matches the required frozen SHA-256
`8cdfc52a68c07df84e1f88506163da1a8dab212004d45689a4c8ab8007b3b61a`.
No fixture, production, status, or Git files were edited by this reviewer.
The pre-existing staged/worktree changes were preserved.

The source review considered the original test-first assignment, critique
supplement, root clarifications, and prior independent rejection records. The
previous defects are repaired in this fixture: the completed mutation starts
before peer lookup; the gated concurrent peer has a cleanup release; and the
race peer observes closure by reading after any response-write outcome.

The race accepts a valid decoded response followed by either exact-peer reuse
or retirement, and accepts cancellation even if the peer response write
completed. Retirement must be evidenced by EOF on the original peer and a
distinct fresh peer/dial; reuse requires the exact original peer and one dial.
The separate completed-mutation case requires `Do` to return a valid correlated
mutation response before cancellation and then requires exact same-peer reuse.
Request-receipt synchronization, the five-second request timeout, one-second
manual-cancel bound, no-retry checks, other-connection isolation, partial-response
retirement, and bounded worker cleanup are present.

## Verification

Host preflight command:
`date -u; free -h; ps -eo pid,comm,rss,%mem --sort=-rss | head -n 12`,
with the specified Go limits recorded. It exited 0; actual host availability was
2.2 GiB. Exact output and exit are archived as `unit5a-oct05-preflight-1.raw`
and `unit5a-oct05-preflight-1.exit`.

Focused package command, run from `apps/godspeed-casework-go`:
`env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-cancellation-oct05/gocache GOPATH=/tmp/sea-cancellation-oct05/gopath go test -race -count=1 -parallel=1 ./internal/adapters/sfwp`.
Exit 1 after 23.173s. The five manual cancellation assertions fail because the in-flight
calls do not return within one second (three run_get/Ask/mutation subcases, pool
isolation, and partial response). No compile/infrastructure failure occurred.
The remaining tests in the package, including response/cancellation race and strict
post-completion mutation reuse, did not report failure. This is the intended
compiling assertion RED for missing in-flight cancellation; it is not production
approval.

Exact test output and exit are archived as `unit5a-oct05-focused-race.raw` and
`unit5a-oct05-focused-race.exit`. Original raw files remain at
`/tmp/sea-cancellation-oct05/`.

## Decision

APPROVE the frozen tests-only fixture as source-reviewable and record the expected
RED. This does not approve any production repair, broader T09 work, or other tests.
Compiler token is returned to root. No deviations beyond the package-level command
running the full `sfwp` package rather than filtering to new test names.
