# Private Next TDD fixture repair 3 builder record

Date: 2026-10-09. Immutable fixture-only source preparation for the bounded
repair authorized after source review `4754d977`. No compile, test, behavioral
RED, formatter write, runtime, gate, Next completion, or T09 settlement is
claimed.

## Exact fresh source grant

> SOURCE RELEASE fresh fixture repair3 now. Root FULL-read accepted
> rejection4754d977 in private-next-tdd-repair2-independent-source-review-oct09.md.
> Read full original assignment/addenda/approved correction and repair2 packet;
> only two required second-read gates, as prior prep. Current source1f23ab0e now
> captured and root decoded/cmp in
> private-next-current-repair2-1f23ab0e-source-snapshot-oct09.json. Use that
> actual snapshot for exact diff; no originalc8 claim. Only next_test.go:
> install secondGate/once-backed releaseSecond for transient-read and retention
> fixtures; signal secondRead then wait gate/ctx before returning
> unavailable/oversize; initial Next + prior WM/initial exact ownership checked
> BEFORE releasing secondGate. Start failed Next (may block on wake), release
> secondGate, keep every existing third-gated failure/marker/ref/count/recovery
> assertion. Combined releaseAll must release second+third on ALL cleanup
> paths. Preserve every other assertion/current source byte except necessary
> bounded declarations/callback/cleanup. Native apply_patch only; no
> production/scaffold, formatter writes, compile/tests/gates/status/debt/Git
> mutations. Freeze AFTER final edit; NEW
> private-next-tdd-repair3-builder-oct09.md full grant/refs, four exact hashes,
> exact diff against actual1f23 snapshot and limitations. Report promptly. Root
> owns actual format probe/RED after independent source review.

## Reviewed packet and source identities

Read the complete original assignment, sequencing and notifier addenda,
projection-failure addendum, approved initial-unavailable correction and its
independent review, repair2 builder packet, and accepted repair2 source review.
The accepted review is
`private-next-tdd-repair2-independent-source-review-oct09.md`, SHA-256
`4754d97759c6140acb77b97bdab5e150c68f353b48317efaa769e357de79204d`. It
identifies the sole remaining race: the transient-read and retention fixtures
did not gate read two, allowing its failure/oversized candidate to publish
before the initial seeded Next and baseline checks completed. It authorizes
only adding those two read gates while retaining their third-read recovery
gates and current assertions.

The actual comparison snapshot is
`private-next-current-repair2-1f23ab0e-source-snapshot-oct09.json`, artifact
SHA-256
`c4029f3b9f3b8c1f3b6d8ce886199915413950e011039a4320f3b13cdbef2dc6`; its
embedded source SHA-256 is
`1f23ab0e55e30fe95a38fc67733a5d961209dd3874f82ce4966e444ccc8c7112`. The
source after this repair has SHA-256
`36dab5455a24521a6715f5ca8f9597a2cad28a070f7c5f09df56598c28386780`.
The unavailable original c8e825 preimage is not used and no comparison or
preservation claim against it is made.

## Exact bounded diff against the actual 1f23ab0e snapshot

The read-only unified diff from the decoded actual snapshot to the final source
was:

```diff
--- snapshot-1f23ab0e
+++ current
@@ -483,9 +483,15 @@
 		return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled"}}, UnreadableIDs: []string{}}, nil
 	})
 	secondRead := make(chan struct{}, 1)
+	secondGate := make(chan struct{})
+	releaseSecond := releaseRunObservationNextBarrier(t, secondGate)
 	thirdRead := make(chan struct{}, 1)
 	thirdGate := make(chan struct{})
 	releaseThird := releaseRunObservationNextBarrier(t, thirdGate)
+	releaseAll := func() {
+		releaseSecond()
+		releaseThird()
+	}
 	var traceCalls atomic.Int32
 	traces := runObservationManagerTraceFunc(func(ctx context.Context, _, _, _ string) (ports.RunTraceSnapshot, error) {
 		switch traceCalls.Add(1) {
@@ -493,6 +499,11 @@
 		case 2:
 			secondRead <- struct{}{}
+			select {
+			case <-secondGate:
+			case <-ctx.Done():
+				return ports.RunTraceSnapshot{}, ctx.Err()
+			}
 			return ports.RunTraceSnapshot{}, errors.New("controlled transient read failure")
 		case 3:
 			thirdRead <- struct{}{}
@@ -507,7 +518,7 @@
 		}
 	})
 	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-failure", itemID))
-	cleanupRunObservationManager(t, manager, releaseThird)
+	cleanupRunObservationManager(t, manager, releaseAll)
 	_, lease, err := manager.prepare(context.Background(), caseID, caller)
 	if err != nil || lease == nil {
 		t.Fatalf("Prepare failure cohort = lease %v, error %v", lease, err)
@@ -536,6 +547,7 @@
 	}()
 	waitRunObservationManagerSignal(t, secondRead, "controlled failed read")
+	releaseSecond()
 	failure := awaitRunObservationNext(t, failureResult, "failed-candidate Next")
 	if failure.err == nil || apperr.KindOf(failure.err) != apperr.KindUnavailable || !reflect.DeepEqual(failure.aggregate, runObservationNextAggregate{}) {
 		t.Fatalf("failed projection Next = (%#v, %v), want typed unavailable and zero aggregate", failure.aggregate, failure.err)
@@ -597,9 +609,15 @@
 				return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled"}}, UnreadableIDs: []string{}}, nil
 			})
 			secondRead := make(chan struct{}, 1)
+			secondGate := make(chan struct{})
+			releaseSecond := releaseRunObservationNextBarrier(t, secondGate)
 			thirdRead := make(chan struct{}, 1)
 			thirdGate := make(chan struct{})
 			releaseThird := releaseRunObservationNextBarrier(t, thirdGate)
+			releaseAll := func() {
+				releaseSecond()
+				releaseThird()
+			}
 			var traceCalls atomic.Int32
 			traces := runObservationManagerTraceFunc(func(ctx context.Context, _, _, _ string) (ports.RunTraceSnapshot, error) {
 				switch traceCalls.Add(1) {
@@ -607,6 +625,11 @@
 				case 2:
 					secondRead <- struct{}{}
+					select {
+					case <-secondGate:
+					case <-ctx.Done():
+						return ports.RunTraceSnapshot{}, ctx.Err()
+					}
 					large := make([]ports.RunTraceFrame, runObservationRetainedFrameLimit)
 					for index := range large {
 						id := "oversized-retained-event-" + strconv.Itoa(index) + strings.Repeat("x", 1100)
@@ -630,7 +653,7 @@
 				}
 			})
 			manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-retention", itemID))
-			cleanupRunObservationManager(t, manager, releaseThird)
+			cleanupRunObservationManager(t, manager, releaseAll)
 			_, lease, err := manager.prepare(context.Background(), caseID, caller)
 			if err != nil || lease == nil {
 				t.Fatalf("Prepare retention cohort = lease %v, error %v", lease, err)
@@ -659,6 +682,7 @@
 				}()
 				waitRunObservationManagerSignal(t, secondRead, "oversized retained candidate")
+				releaseSecond()
 				failed := awaitRunObservationNext(t, failedResult, "retention-unavailable Next")
 				if failed.err == nil || apperr.KindOf(failed.err) != apperr.KindUnavailable || !reflect.DeepEqual(failed.aggregate, runObservationNextAggregate{}) {
 					t.Fatalf("retention-unavailable Next = (%#v, %v), want typed failure and zero aggregate", failed.aggregate, failed.err)
```

The diff retains existing successful initial Next, baseline watermark snapshot,
exact initial entry/ref ownership checks before second-gate release; the failed
Next is started before waiting for the actual second-read signal. All existing
third-gate checks for unavailable result, zero aggregate, unchanged watermarks,
marker, exact refs and counted capacity remain before the fitting read is
released. Terminal retention still exits via its existing drain branch. Both
cleanup callbacks now release both read gates; each individual release remains
once-backed and each gated callback selects on worker context cancellation.

No other source file or assertion was changed. The snapshot diff was generated
read-only by decoding the root-saved xz+base64 payload in memory and comparing
it with the final file. No compiler, tests, formatter, gates, status/debt edits,
or Git mutation were performed. Root owns the actual format probe and expected
RED after independent source review. This preparation makes no runtime or
implementation claim.
