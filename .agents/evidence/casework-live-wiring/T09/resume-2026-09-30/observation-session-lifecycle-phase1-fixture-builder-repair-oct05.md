# Observation session lifecycle Phase 1 fixture repair

Date: 2026-10-05. Source-only builder repair after the independent Phase 1
rejection. Production changes remain outside this assignment.

## Binding scope and identities

Followed `observation-session-lifecycle-root-proposal-oct05.md`,
`observation-session-lifecycle-phase1-independent-review-oct05.md`, and the
Phase 1 fixture assignment in
`observation-session-lifecycle-phase1-builder-oct05.md`. Owned only
`apps/godspeed-casework-go/internal/auth/session_observation_test.go` and this
new record. Before SHA-256, as recorded by the builder handoff and independent
review: `78cbe26fffc70837eb27ce597872ad9a1dfe7fe70cce745441ae10a721c2c6aa`.
After SHA-256: `5bbc19a4f1719defd79c564623603a64e2daa391a7db9ebe04821b94c34dd934`.
`gofmt -d` produced no output for the frozen file.

## Exact source delta

The unused local in `TestSessionStoreCurrentRacingDestroyNeverMissesRevocation`
changed from `clock, store := ...` to `_, store := ...`.

The prior `sync.WaitGroup` join was replaced with this bounded completion
pattern. Each worker signals only after completing its store operation and
writing any result; receiving both signals establishes the happens-before
edge before the test reads `got` and `found`:

```go
start := make(chan struct{})
completed := make(chan struct{}, 2)
var got CurrentSessionState
var found bool
go func() {
    <-start
    got, found = store.Current(session.ID)
    completed <- struct{}{}
}()
go func() {
    <-start
    store.Destroy(session.ID)
    completed <- struct{}{}
}()
close(start)
timer := time.NewTimer(5 * time.Second)
defer timer.Stop()
for worker := 0; worker < 2; worker++ {
    select {
    case <-completed:
    case <-timer.C:
        t.Fatal("racing Current and Destroy workers did not complete within five seconds")
    }
}
```

The pre-existing file also failed `gofmt -d`; the only formatter-equivalent
changes align the two test table declarations and four `name` fields exactly
as emitted by `gofmt` from the edited source. No assertions, race outcomes,
closure checks, deadlines, capacity checks, or coverage were removed or
weakened. Source comparison of the generated formatted copy yielded the same
SHA-256 as the frozen file.

## Deviations and verification boundary

No material deviation. No production declaration, other test, dependency,
schema, session identity, lifecycle behavior, compiler/test/scanner command,
Git operation, hook, status file, or debt ledger was changed or run. No test
was compiled or executed. This record requests fresh independent review only;
it does not claim assertion RED or test success.
