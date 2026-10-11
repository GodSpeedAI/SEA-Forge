# Fresh authorization retry fixture repair result

Date: 2026-10-07. Source-only test-fixture change. No tests, compiler,
formatter, scanner, typecheck, Git, or runtime command was run.

## Exact preserved preimage

The pre-edit test source is preserved by root's UTF-8 JSON wrapper at
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/manager-test-87be-before-auth-retry-oct07.json`.
Read-only decoding immediately before the edit yielded 54,907 bytes and SHA-256
`87be7ae28010132c3e5ddbaa9ad0469ed5891061b88a005f86d8d70b977124c0`; both
matched the live pre-edit test bytes exactly. This artifact is a source JSON
wrapper, not runtime evidence.

## Resulting source identities

- `run_observation_manager.go`: 5,488 bytes, SHA-256
  `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`
  (unchanged from assignment).
- `run_observation_manager_test.go`: 55,815 bytes, SHA-256
  `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.

## Exact diff summary

Against the decoded 87be test preimage, the source diff is limited to
`TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList` and its
sole-use import:

1. Remove the `strings` import.
2. Name the existing `case_1` fixture constant and add an atomic trace-call
   counter to the existing zero-result trace fake.
3. Preserve the stale-claim refusal check for typed unavailable error, zero
   event, nil lease, zero list calls, and zero perspective checks.
4. Replace the corrected-claim retry's hardcoded unwired-stub/error/zero-wrapper
   expectation with a successful nonnil lease and exact event expectation:
   `execution_observation`, cursor `cursor-1`, timestamp and observed-at
   `2026-10-06T12:00:00Z`, complete list state, `no_runs`, all six present zero
   counts, read budget `{Limit: 8, ReadsAttempted: 0, Exhausted: false}`, and a
   nonnil empty `Runs` slice.
5. Assert one perspective verification, one list call, and zero trace calls;
   detach the returned empty lease.

The separate refused-list DTO fixture and all lifecycle/cleanup fixtures were
left untouched. Production source remains the private always-unavailable stub;
no algorithm, `Next`, or callsite integration was added. No deviation from the
assigned semantic assertions was made. The necessary case constant and trace
counter support the requested case identity and no-trace expectations.

The exact unified diff against the decoded preimage follows:

```diff
--- 87be-preimage
+++ current
@@ -6,7 +6,6 @@
 	"reflect"
 	"runtime"
 	"strconv"
-	"strings"
 	"sync"
 	"sync/atomic"
 	"testing"
@@ -940,22 +939,24 @@
 }
 
 func TestRunObservationManagerAuthorizationRefusalReturnsNoLeaseBeforeList(t *testing.T) {
-	var listCalls atomic.Int32
+	const caseID = "case_1"
+	var listCalls, traceCalls atomic.Int32
 	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
 		listCalls.Add(1)
 		return ports.RunListResult{Runs: []ports.RunSummary{}, UnreadableIDs: []string{}}, nil
 	})
 	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
+		traceCalls.Add(1)
 		return ports.RunTraceSnapshot{}, nil
 	})
 	caller := runObservationManagerCaller("session-A")
 	perspective := &runObservationManagerPerspectiveFake{}
-	contextSource := runObservationManagerContext("case_1", "cursor-1", "item-1")
+	contextSource := runObservationManagerContext(caseID, "cursor-1", "item-1")
 	otherClaim := ports.ActorClaim{ActorID: "operator_other", Role: "operator"}
 	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(otherClaim), perspective, contextSource)
 	cleanupRunObservationManager(t, manager, nil)
 
-	if event, lease, err := manager.prepare(context.Background(), "case_1", caller); err == nil || apperr.KindOf(err) != apperr.KindUnavailable || lease != nil || !reflect.DeepEqual(event, contract.RunTraceObservationEvent{}) {
+	if event, lease, err := manager.prepare(context.Background(), caseID, caller); err == nil || apperr.KindOf(err) != apperr.KindUnavailable || lease != nil || !reflect.DeepEqual(event, contract.RunTraceObservationEvent{}) {
 		t.Fatalf("stale-claim Prepare() = (%+v, %v, %v), want authorization error/zero wrapper/nil lease", event, lease, err)
 	}
 	if listCalls.Load() != 0 || perspective.calls.Load() != 0 {
@@ -963,14 +964,35 @@
 	}
 
 	// Authorization refusal precedes reservation by contract. A corrected claim
-	// retries admission and reaches the explicit unwired stub without run.list.
+	// retries admission and receives the exact successful empty-list event.
 	manager.sessions = runObservationManagerCurrent(caller.Identity.Claim())
-	event, lease, err := manager.prepare(context.Background(), "case_1", caller)
-	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable || lease != nil || !reflect.DeepEqual(event, contract.RunTraceObservationEvent{}) || !strings.Contains(err.Error(), "unit 1 is not wired") {
-		t.Fatalf("Prepare after authorization retry = (%+v, %v, %v), want explicit unwired stub/zero wrapper/nil lease", event, lease, err)
-	}
-	if perspective.calls.Load() != 1 || listCalls.Load() != 0 {
-		t.Fatalf("authorized retry downstream calls: perspective=%d list=%d, want one perspective check and no list before stub", perspective.calls.Load(), listCalls.Load())
+	event, lease, err := manager.prepare(context.Background(), caseID, caller)
+	if err != nil || lease == nil {
+		t.Fatalf("Prepare after authorization retry = (%+v, %v, %v), want successful no-runs event and empty lease", event, lease, err)
+	}
+	stamp := "2026-10-06T12:00:00Z"
+	want := contract.RunTraceObservationEvent{
+		EventType: contract.RunTraceObservationEventType,
+		Cursor:    "cursor-1",
+		Timestamp: stamp,
+		Payload: contract.RunTraceObservation{
+			CaseID: caseID, ObservedAt: stamp,
+			RunListState: "complete", ObservationState: "no_runs",
+			ListedRunCount: managerCount(0), SelectedRunCount: managerCount(0),
+			ValidatedRunCount: managerCount(0), UnreadableRunCount: managerCount(0),
+			UnavailableRunCount: managerCount(0), OmittedRunCount: managerCount(0),
+			HydrationReadBudget: contract.RunTraceHydrationReadBudget{Limit: 8, ReadsAttempted: 0, Exhausted: false},
+			Runs:                []contract.RunTraceRunObservation{},
+		},
+	}
+	if !reflect.DeepEqual(event, want) {
+		t.Fatalf("Prepare after authorization retry = %+v, want exact successful no-runs event %+v", event, want)
+	}
+	if perspective.calls.Load() != 1 || listCalls.Load() != 1 || traceCalls.Load() != 0 {
+		t.Fatalf("authorized retry downstream calls: perspective=%d list=%d trace=%d, want one perspective check, one list, and no trace reads", perspective.calls.Load(), listCalls.Load(), traceCalls.Load())
+	}
+	if err := lease.detachAndDrain(context.Background()); err != nil {
+		t.Fatalf("Detach after authorization retry: %v", err)
 	}
 }
 
```

## Limits and handoff

The retry's future-success expectation is source-only and unrun. No RED/GREEN,
compilation, lifecycle behavior, integration, or T09 completion is claimed.
The bounded fixture is ready for review by a different guard critic.
