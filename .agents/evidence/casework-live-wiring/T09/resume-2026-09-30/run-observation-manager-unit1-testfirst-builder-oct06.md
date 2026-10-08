# Run observation manager unit 1 — test-first builder record

Date: 2026-10-06. **TESTFIRST files only.** This record does not claim an
expected RED, source approval, manager implementation, caller integration,
public streaming, or T09 settlement.

## Original instructions and evidence order

The exact bounded assignment is preserved before the source files in
`run-observation-manager-unit1-original-assignment-oct06.md`, SHA-256
`de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`. The two
new Go files were absent from the repository inventory before writing. They
are new files; no previous evidence or source was overwritten.

## Private test seam

The unexported seam is deliberately limited to the server package:

```go
type runObservationListReader interface {
    RunsListForCase(context.Context, ports.CaseRef) (ports.RunListResult, error)
}
type runObservationSessionCurrent interface {
    Current(string) (auth.CurrentSessionState, bool)
}
type runObservationAuthorize func(
    context.Context, runObservationCaller, string,
    runObservationSessionCurrent, PerspectiveVerifier,
) error
type runObservationGuard func(string) (runObservationPresentContext, error)

func newRunObservationManager(
    runObservationListReader, ports.RunTracePort,
    runObservationSessionCurrent, PerspectiveVerifier,
    runObservationAuthorize, runObservationGuard, func() time.Time,
) *runObservationManager
func (*runObservationManager) prepare(
    context.Context, string, *requestIdentity,
) (contract.RunTraceObservationEvent, *runObservationLease, error)
func (*runObservationManager) stopAndDrain(context.Context) error
func (*runObservationLease) detachAndDrain(context.Context) error
```

The actual return shapes are grounded in the existing ports and guards:
`RunsListForCase` returns `(ports.RunListResult, error)`;
`RunTracePort.ReadRunTrace` returns `(ports.RunTraceSnapshot, error)`;
`SessionStore.Current` returns `(auth.CurrentSessionState, bool)`;
`PerspectiveVerifier.VerifyPerspective` returns `error`; and the injected
present-context guard returns `(runObservationPresentContext, error)` from the
existing `checkRunObservationPresentContext` contract. `runObservationCaller`
copies only source, session ID, and `ports.ActorClaim`; its type has no bearer,
CSRF, or mutable session pointer field.

An optional private `afterAttach(caseID, runID)` callback field is a
deterministic barrier for the shared-initializer fixture. If used by a later
implementation, it must run after releasing the manager lock. It adds no
exported symbol or runtime wiring. The current stub does not invoke it.

## Files and test cases

- `apps/godspeed-casework-go/internal/server/run_observation_manager.go` is
  only a compile seam. Its `prepare`, `stopAndDrain`, and `detachAndDrain` return typed
  unavailable with safe fixed messages; it has no state maps, worker, caller,
  or production route. The only behavior before that deliberate unwired
  response is caller snapshot validation plus injected auth and the existing
  present-context guard. No full manager algorithm was implemented.
- `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go`
  uses channel-controlled fake list/trace ports and a fixed clock. Its fixtures
  cover:
  1. exact empty initial event wrapper, real current cursor, zero-present counts,
     nonnil `Runs`, and lease result;
  2. stale present cursor refusal before list/read and a valid parent control
     that must reach list/trace;
  3. parent outside the captured Horizon rejected without trace disclosure;
  4. returned run identity mismatch withheld from the DTO;
  5. caller snapshot omits bearer/session credential material;
  6. 16 preparing cohort reservations before list, then the same 16 active
     leases remain counted until detach;
  7. 16 initializing pollers across two eight-run cohorts, with a third cohort
     capacity-limited and no 17th `ReadRunTrace` start;
  8. failed list Prepare returns zero wrapper/nil lease and a retry can be
     admitted;
  9. cancellation after one successful attachment and while a second actual
     read is held: rollback must wait for that call to return before a retry
     can reinitialize the keys;
  10. a shared initializer survives initiator cancellation after the second
      lease ref is observed through the private barrier, returns to the
      surviving watcher, and starts one trace read;
  11. timed-out stop keeps admission closed while a held read remains owned,
      then joins after actual read return; repeated stop and repeated detach are
      required to be idempotent.

All inter-worker ordering is controlled with channels; timeouts are fail-safe
test bounds, not sleeps used to establish ordering. No new test config, public
API, test-only production method, or production caller was added. The selector,
hydration helper, present-context guard, shared physical admission, cooldown,
retry, cancellation callback, and client retirement source remain unchanged.

## Material limits and deviations

This source/fixture set is a compile scaffold for independent source review,
not the full algorithm. It does not implement cohort/poller state maps, attach
dispositions, shared worker ownership, precise stop timeout/lease-drain
behavior, initial DTO construction, auth policy, later `Next`, ID ledgers,
retention budgeting, polling cadence, or SSE. The shared-initializer and
drain/rollback tests describe future behavior through the private `Prepare` /
lease seam; because the stub does not execute those transitions, their
expected failure remains to be demonstrated by the separately authorized
critic. The optional barrier is the only test synchronization hook beyond
injected ports and guards.

The plan's 16 shared poller-entry bound is already accepted; the 16 cohort /
128-attachment and combined 1 MiB retained-value bounds follow the new root
private design decision, which supersedes only draft extra-operator-approval
language for routine private caps. None of those private choices authorizes
public V4, schema, cursor/frontier, kernel writer, route, SSE, or identity
changes. The distinct 32 MiB Go inbound response-line cap is already
implemented; the kernel 64 MiB whole-journal cap and post-decode 1,024-frame
DTO retention are different bounds. CW-23 incomplete-prefix/false-empty debt
remains open and is not converted into a continuity guarantee here.

The original manager blueprint used capitalized illustrative pseudocode names;
the implementation seam and all test calls use the private lower-case names
above. No Go export was added. Root requested and confirmed this correction
before source freeze.

The first source review must inspect these frozen files and this record before
the root separately grants the sole compiler owner permission for an actual
focused RED. No test, typecheck, compile, scanner, Git, or gate was run here.
`gofmt -d` was a read-only formatting check and returned no diff.

## Source hashes at handoff

```text
original assignment: de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916
run_observation_manager.go: ee7afab1b22c011cf79b602f415336d459fda346aad1bf0c942a4464326ff878
run_observation_manager_test.go: 2b2d1fbcd3bfb7e7c27254673ca46b2deb30d9edd4ac862fa464e2a9b18334b6
```
