# Retained-version helper atomicity fixture repair — result

Date: 2026-10-07. TESTONLY source repair. No tests, compiler, formatter,
scanner, typecheck, Git, or runtime command was run.

## Exact preimage and resulting identities

Immediately before editing, the exact UTF-8 JSON `content` at
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/retained-test-5365-before-atomicity-repair-oct07.json`
decoded to 12,464 bytes, SHA-256
`5365c1ff47cdf0006868b102a1fdcdf9c77a12d51cee4ab06b04af536f494978`, and
matched the live test bytes exactly.

| File | Bytes | SHA-256 |
|---|---:|---|
| `run_observation_retained_version.go` | 5,836 | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` |
| `run_observation_retained_version_test.go` | 26,472 | `6ba0ab0ddbd80614d25c48d6485f1af70a75e8f670a72fac535c8ec128652d00` |
| `run_observation_retained_version_helper_atomicity_repair_assignment_oct07.md` | 6,541 | `abf191672db3e08e2c827068dfab97f4564cb0ae2a1f19a865370ef9f7494b20` |

The production helper remains the explicit rejected stub and encoder; its hash
is unchanged. Only its existing test file was modified. The exact unified diff
from the preserved preimage follows.

## Repair coverage

- Nonterminal refusal now proves the entire caller-owned prior state and its
  canonical image remain unchanged; the output matches the complete prior
  state except for the fixed availability marker and retains the same image
  length. Recovery checks the complete A/B/new ledger, exact source order,
  first ordinals, accepted timestamp, and next generation.
- Terminal refusal compares all retained values to the prior safe state plus
  the fixed terminal marker and searches both state and canonical image for
  candidate-only IDs, timestamps, metadata, status, and exit code.
- Exact-boundary tests measure the computed candidate images at 1 MiB and
  1 MiB + 1, then verify the accepted candidate's actual image is exactly
  1 MiB.
- Reorder, shorter window, and reappearance fixtures preserve source order and
  original ordinals; accepted candidate calls also prove prior images remain
  unchanged. Duplicate event IDs in one window are invalid, matching the
  trace-port contract; no winner policy is asserted.
- Prior/result/source optional pointers are checked nonnil and distinct before
  mutation. Generation and ordinal overflow reject without wrap or prior
  mutation. At maximum uint64 counters, all availability codes preserve
  canonical image length, including frame and ledger ordinals.

No source declarations, dependencies, public contracts, helper algorithm,
manager lifecycle, `Next`, or integration were changed. Material deviations:
none. The new assertions are unrun source-level expectations; no RED/GREEN or
runtime behavior is claimed. The fresh fixture is ready for independent guard
review against the full assignment and this receipt.

## Exact unified diff

```diff
--- 5365-preimage
+++ current
@@ -52,6 +52,34 @@
 	return state
 }
 
+func cloneRetainedTestState(input *runObservationRetainedState) *runObservationRetainedState {
+	if input == nil {
+		return nil
+	}
+	output := *input
+	if input.Frames != nil {
+		output.Frames = make([]retainedObservedFrame, len(input.Frames))
+		for i, frame := range input.Frames {
+			output.Frames[i] = frame
+			if frame.Frame.ExecutionStatus != nil {
+				status := *frame.Frame.ExecutionStatus
+				output.Frames[i].Frame.ExecutionStatus = &status
+			}
+			if frame.Frame.ExitCode != nil {
+				code := *frame.Frame.ExitCode
+				output.Frames[i].Frame.ExitCode = &code
+			}
+		}
+	}
+	if input.SeenByID != nil {
+		output.SeenByID = make(map[string]uint64, len(input.SeenByID))
+		for id, ordinal := range input.SeenByID {
+			output.SeenByID[id] = ordinal
+		}
+	}
+	return &output
+}
+
 func requireAcceptedRetainedCandidate(t *testing.T, result retainedCandidateResult) *runObservationRetainedState {
 	t.Helper()
 	if result.Outcome != retainedCandidateAccepted || result.State == nil {
@@ -65,11 +93,14 @@
 	snapshot := retainedTestSnapshot(key, retainedTestFrame("opaque-A", "2026-10-07T12:00:00Z"))
 	result := buildRunObservationRetainedCandidate(nil, key, snapshot, retainedTestTime)
 	state := requireAcceptedRetainedCandidate(t, result)
-	if state.AcceptedAt != retainedTestTime.Format(time.RFC3339Nano) || state.HighestOrdinal != 1 {
-		t.Fatalf("accepted timestamp/high-water = %q/%d", state.AcceptedAt, state.HighestOrdinal)
+	if state.AcceptedAt != retainedTestTime.Format(time.RFC3339Nano) || state.HighestOrdinal != 1 || state.Generation != 1 {
+		t.Fatalf("accepted timestamp/high-water/generation = %q/%d/%d", state.AcceptedAt, state.HighestOrdinal, state.Generation)
 	}
 	if len(state.Frames) != 1 || state.Frames[0].FirstOrdinal != 1 || state.Frames[0].Frame.EventID != "opaque-A" {
 		t.Fatalf("captured frame window = %#v", state.Frames)
+	}
+	if state.Frames[0].Frame.ExecutionStatus == nil || state.Frames[0].Frame.ExitCode == nil {
+		t.Fatal("retained optional frame pointers are nil")
 	}
 	if state.Frames[0].Frame.ExecutionStatus == snapshot.Frames[0].ExecutionStatus || state.Frames[0].Frame.ExitCode == snapshot.Frames[0].ExitCode {
 		t.Fatal("retained optional frame pointers alias the source snapshot")
@@ -81,17 +112,65 @@
 	}
 }
 
+func TestRunObservationRetainedCandidateClonesPriorAndSourcePointers(t *testing.T) {
+	key := retainedTestKey()
+	prior := requireAcceptedRetainedCandidate(t, buildRunObservationRetainedCandidate(nil, key,
+		retainedTestSnapshot(key, retainedTestFrame("opaque-A", "2026-10-07T12:00:00Z")), retainedTestTime))
+	source := retainedTestSnapshot(key, retainedTestFrame("opaque-A", "2026-10-07T12:01:00Z"))
+	result := buildRunObservationRetainedCandidate(prior, key, source, retainedTestTime.Add(time.Second))
+	next := requireAcceptedRetainedCandidate(t, result)
+	if len(prior.Frames) != 1 || len(next.Frames) != 1 || len(source.Frames) != 1 {
+		t.Fatalf("pointer fixture lengths prior/next/source = %d/%d/%d", len(prior.Frames), len(next.Frames), len(source.Frames))
+	}
+	priorFrame := prior.Frames[0].Frame
+	sourceFrame := source.Frames[0]
+	nextFrame := next.Frames[0].Frame
+	if priorFrame.ExecutionStatus == nil || priorFrame.ExitCode == nil || sourceFrame.ExecutionStatus == nil || sourceFrame.ExitCode == nil || nextFrame.ExecutionStatus == nil || nextFrame.ExitCode == nil {
+		t.Fatal("prior, source, and result optional pointers must all be nonnil before alias checks")
+	}
+	if nextFrame.ExecutionStatus == priorFrame.ExecutionStatus || nextFrame.ExitCode == priorFrame.ExitCode ||
+		nextFrame.ExecutionStatus == sourceFrame.ExecutionStatus || nextFrame.ExitCode == sourceFrame.ExitCode {
+		t.Fatal("result optional frame pointers alias prior state or source snapshot")
+	}
+	*nextFrame.ExecutionStatus = "result-only-status"
+	*nextFrame.ExitCode = 73
+	if *priorFrame.ExecutionStatus != "completed" || *priorFrame.ExitCode != 0 ||
+		*sourceFrame.ExecutionStatus != "completed" || *sourceFrame.ExitCode != 0 {
+		t.Fatal("mutating result frame pointers changed prior or source values")
+	}
+}
+
 func TestRunObservationRetainedCandidateKeepsFirstOrdinalsAcrossReorderAndShorterWindow(t *testing.T) {
 	key := retainedTestKey()
 	first := buildRunObservationRetainedCandidate(nil, key, retainedTestSnapshot(key,
 		retainedTestFrame("opaque-A", "2026-10-07T12:00:00Z"),
 		retainedTestFrame("opaque-B", "2026-10-07T12:00:01Z")), retainedTestTime)
 	prior := requireAcceptedRetainedCandidate(t, first)
+	priorBefore := cloneRetainedTestState(prior)
+	priorImage, err := marshalRunObservationRetainedImage(prior)
+	if err != nil {
+		t.Fatal(err)
+	}
 	second := buildRunObservationRetainedCandidate(prior, key, retainedTestSnapshot(key,
 		retainedTestFrame("opaque-B", "2026-10-07T12:00:01Z"),
 		retainedTestFrame("opaque-A", "2026-10-07T12:05:00Z"),
 		retainedTestFrame("opaque-C", "2026-10-07T12:00:02Z")), retainedTestTime.Add(time.Second))
 	reordered := requireAcceptedRetainedCandidate(t, second)
+	priorAfterImage, err := marshalRunObservationRetainedImage(prior)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if !reflect.DeepEqual(prior, priorBefore) || !reflect.DeepEqual(priorImage, priorAfterImage) {
+		t.Fatal("accepted reorder mutated the prior published state")
+	}
+	if len(reordered.Frames) != 3 {
+		t.Fatalf("reordered source window contains %d frames, want 3", len(reordered.Frames))
+	}
+	reorderedBefore := cloneRetainedTestState(reordered)
+	reorderedImage, err := marshalRunObservationRetainedImage(reordered)
+	if err != nil {
+		t.Fatal(err)
+	}
 	wantIDs := []string{"opaque-B", "opaque-A", "opaque-C"}
 	wantOrdinals := []uint64{2, 1, 3}
 	for i, frame := range reordered.Frames {
@@ -99,20 +178,71 @@
 			t.Fatalf("frame[%d] = %q/%d, want %q/%d", i, frame.Frame.EventID, frame.FirstOrdinal, wantIDs[i], wantOrdinals[i])
 		}
 	}
-	if reordered.HighestOrdinal != 3 || reordered.SeenByID["opaque-A"] != 1 || reordered.SeenByID["opaque-B"] != 2 || reordered.SeenByID["opaque-C"] != 3 {
-		t.Fatalf("ledger/high-water = %#v/%d", reordered.SeenByID, reordered.HighestOrdinal)
+	if reordered.HighestOrdinal != 3 || reordered.Generation != 2 || !reflect.DeepEqual(reordered.SeenByID, map[string]uint64{"opaque-A": 1, "opaque-B": 2, "opaque-C": 3}) {
+		t.Fatalf("ledger/high-water/generation = %#v/%d/%d", reordered.SeenByID, reordered.HighestOrdinal, reordered.Generation)
 	}
 	shorter := buildRunObservationRetainedCandidate(reordered, key, retainedTestSnapshot(key,
 		retainedTestFrame("opaque-C", "2026-10-07T12:00:02Z")), retainedTestTime.Add(2*time.Second))
 	short := requireAcceptedRetainedCandidate(t, shorter)
-	if len(short.Frames) != 1 || short.Frames[0].FirstOrdinal != 3 || len(short.SeenByID) != 3 || short.HighestOrdinal != 3 {
+	reorderedAfterImage, err := marshalRunObservationRetainedImage(reordered)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if !reflect.DeepEqual(reordered, reorderedBefore) || !reflect.DeepEqual(reorderedImage, reorderedAfterImage) {
+		t.Fatal("accepted shorter window mutated the prior published state")
+	}
+	shortBefore := cloneRetainedTestState(short)
+	shortImage, err := marshalRunObservationRetainedImage(short)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if len(short.Frames) != 1 || short.Frames[0].FirstOrdinal != 3 || len(short.SeenByID) != 3 || short.HighestOrdinal != 3 || short.Generation != 3 {
 		t.Fatalf("shorter source window lost ledger/high-water: %#v", short)
 	}
+	reappeared := buildRunObservationRetainedCandidate(short, key, retainedTestSnapshot(key,
+		retainedTestFrame("opaque-A", "2026-10-07T12:06:00Z"),
+		retainedTestFrame("opaque-C", "2026-10-07T12:00:02Z")), retainedTestTime.Add(3*time.Second))
+	reappearedState := requireAcceptedRetainedCandidate(t, reappeared)
+	if len(reappearedState.Frames) != 2 || reappearedState.Frames[0].Frame.EventID != "opaque-A" || reappearedState.Frames[0].FirstOrdinal != 1 ||
+		reappearedState.Frames[1].Frame.EventID != "opaque-C" || reappearedState.Frames[1].FirstOrdinal != 3 ||
+		reappearedState.HighestOrdinal != 3 || reappearedState.Generation != 4 || !reflect.DeepEqual(reappearedState.SeenByID, map[string]uint64{"opaque-A": 1, "opaque-B": 2, "opaque-C": 3}) {
+		t.Fatalf("disappeared ID did not reappear in source order with its first ordinal: %#v", reappearedState)
+	}
+	shortAfterImage, err := marshalRunObservationRetainedImage(short)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if !reflect.DeepEqual(short, shortBefore) || !reflect.DeepEqual(shortImage, shortAfterImage) {
+		t.Fatal("accepted reappearance mutated the prior published state")
+	}
+}
+
+func TestRunObservationRetainedCandidateRejectsDuplicateIDsInOneWindow(t *testing.T) {
+	key := retainedTestKey()
+	prior := retainedTestState(key, "opaque-existing")
+	before, err := marshalRunObservationRetainedImage(prior)
+	if err != nil {
+		t.Fatal(err)
+	}
+	duplicate := buildRunObservationRetainedCandidate(prior, key, retainedTestSnapshot(key,
+		retainedTestFrame("duplicate-id", "2026-10-07T12:00:00Z"),
+		retainedTestFrame("duplicate-id", "2026-10-07T12:00:01Z")), retainedTestTime.Add(time.Second))
+	if duplicate.Outcome != retainedCandidateInvalid || duplicate.State != nil {
+		t.Fatalf("duplicate IDs in one trace window = %v/%#v, want invalid/nil without selecting a winner", duplicate.Outcome, duplicate.State)
+	}
+	after, err := marshalRunObservationRetainedImage(prior)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if !reflect.DeepEqual(prior, retainedTestState(key, "opaque-existing")) || !reflect.DeepEqual(before, after) {
+		t.Fatal("invalid duplicate window mutated the prior state or canonical image")
+	}
 }
 
 func TestRunObservationRetainedCandidateRejectsBudgetAtomicallyAndRecovers(t *testing.T) {
 	key := retainedTestKey()
 	prior := retainedTestState(key, "opaque-A", "opaque-B")
+	priorBefore := cloneRetainedTestState(prior)
 	before, err := marshalRunObservationRetainedImage(prior)
 	if err != nil {
 		t.Fatal(err)
@@ -123,24 +253,38 @@
 	if result.Outcome != retainedCandidateRejected || result.State == nil || result.State.Availability != retainedRetentionUnavailable {
 		t.Fatalf("over-budget outcome/state = %v/%#v", result.Outcome, result.State)
 	}
-	if len(result.State.SeenByID) != len(prior.SeenByID) || result.State.HighestOrdinal != prior.HighestOrdinal || !reflect.DeepEqual(result.State.Frames, prior.Frames) {
-		t.Fatal("over-budget candidate partially changed the retained version/ledger")
+	priorAfter, err := marshalRunObservationRetainedImage(prior)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if !reflect.DeepEqual(prior, priorBefore) || !reflect.DeepEqual(before, priorAfter) {
+		t.Fatal("over-budget rejection mutated the caller-owned prior state or canonical image")
+	}
+	wantRefusal := cloneRetainedTestState(priorBefore)
+	wantRefusal.Availability = retainedRetentionUnavailable
+	if !reflect.DeepEqual(result.State, wantRefusal) {
+		t.Fatalf("nonterminal refusal changed prior safe state beyond its fixed marker: got=%#v want=%#v", result.State, wantRefusal)
 	}
 	after, err := marshalRunObservationRetainedImage(result.State)
 	if err != nil {
 		t.Fatal(err)
 	}
-	if len(after) > runObservationRetainedImageMaxBytes {
-		t.Fatalf("fixed-width refusal marker grew retained image to %d bytes", len(after))
+	wantAfter, err := marshalRunObservationRetainedImage(wantRefusal)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if len(after) > runObservationRetainedImageMaxBytes || len(after) != len(before) || !reflect.DeepEqual(after, wantAfter) {
+		t.Fatalf("fixed-width refusal image changed size or non-marker bytes: before=%d after=%d", len(before), len(after))
 	}
 	if _, acceptedCandidate := result.State.SeenByID[oversizedID]; acceptedCandidate {
 		t.Fatal("rejected candidate ID entered the ledger")
 	}
-	if !reflect.DeepEqual(prior.SeenByID, retainedTestState(key, "opaque-A", "opaque-B").SeenByID) {
-		t.Fatal("caller-owned prior ledger was mutated")
-	}
-	if len(before) == 0 {
-		t.Fatal("fixture did not produce a canonical prior image")
+	if len(before) == 0 || result.State.Generation != prior.Generation || result.State.AcceptedAt != prior.AcceptedAt ||
+		result.State.Execution != prior.Execution || result.State.Settlement != prior.Settlement ||
+		result.State.ObservationState != prior.ObservationState || result.State.TotalFrameCount != prior.TotalFrameCount ||
+		!reflect.DeepEqual(result.State.Frames, prior.Frames) || !reflect.DeepEqual(result.State.SeenByID, prior.SeenByID) ||
+		result.State.HighestOrdinal != prior.HighestOrdinal {
+		t.Fatal("nonterminal refusal changed prior generation, metadata, frame window, or lifetime ledger")
 	}
 
 	recovery := buildRunObservationRetainedCandidate(result.State, key, retainedTestSnapshot(key,
@@ -151,23 +295,72 @@
 	if recovered.Availability != retainedCurrent || recovered.HighestOrdinal != 3 || recovered.SeenByID["new-after-recovery"] != 3 {
 		t.Fatalf("recovery did not preserve full ledger atomically: %#v", recovered)
 	}
+	wantLedger := map[string]uint64{"opaque-A": 1, "opaque-B": 2, "new-after-recovery": 3}
+	if !reflect.DeepEqual(recovered.SeenByID, wantLedger) || recovered.Generation != prior.Generation+1 ||
+		recovered.AcceptedAt != retainedTestTime.Add(2*time.Second).Format(time.RFC3339Nano) ||
+		len(recovered.Frames) != 3 || recovered.Frames[0].Frame.EventID != "opaque-A" || recovered.Frames[0].FirstOrdinal != 1 ||
+		recovered.Frames[1].Frame.EventID != "opaque-B" || recovered.Frames[1].FirstOrdinal != 2 ||
+		recovered.Frames[2].Frame.EventID != "new-after-recovery" || recovered.Frames[2].FirstOrdinal != 3 {
+		t.Fatalf("recovery ledger/window/generation/time = %#v/%#v/%d/%q", recovered.SeenByID, recovered.Frames, recovered.Generation, recovered.AcceptedAt)
+	}
 }
 
 func TestRunObservationRetainedCandidateTerminalRefusalKeepsOnlySafePriorState(t *testing.T) {
 	key := retainedTestKey()
 	prior := retainedTestState(key, "safe-old-id")
+	priorBefore := cloneRetainedTestState(prior)
+	priorImage, err := marshalRunObservationRetainedImage(prior)
+	if err != nil {
+		t.Fatal(err)
+	}
 	candidateID := "candidate-secret-" + strings.Repeat("i", runObservationRetainedImageMaxBytes)
-	over := retainedTestSnapshot(key, retainedTestFrame(candidateID, "2026-10-07T12:00:03Z"))
-	over.Execution = "completed"
+	candidateStatus := "candidate-only-status"
+	candidateExitCode := int64(91)
+	candidateTimestamp := "2026-10-07T12:00:03Z"
+	overFrame := retainedTestFrame(candidateID, candidateTimestamp)
+	overFrame.ExecutionStatus = &candidateStatus
+	overFrame.ExitCode = &candidateExitCode
+	over := retainedTestSnapshot(key, overFrame)
+	over.Execution = "candidate-only-execution"
+	over.Settlement = "candidate-only-settlement"
+	over.TotalFrameCount = 2
 	result := buildRunObservationRetainedCandidate(prior, key, over, retainedTestTime.Add(time.Second))
 	if result.Outcome != retainedCandidateRejected || result.State == nil || result.State.Availability != retainedTerminalRetentionFailure {
 		t.Fatalf("terminal over-budget state = %v/%#v", result.Outcome, result.State)
 	}
-	if _, leaked := result.State.SeenByID[candidateID]; leaked || result.State.Execution == over.Execution || result.State.AcceptedAt == retainedTestTime.Add(time.Second).Format(time.RFC3339Nano) {
-		t.Fatalf("terminal candidate data leaked into safe old state: %#v", result.State)
-	}
-	if result.State.SeenByID["safe-old-id"] != 1 || result.State.HighestOrdinal != 1 {
-		t.Fatalf("prior safe state was not retained: %#v", result.State)
+	if !reflect.DeepEqual(prior, priorBefore) {
+		t.Fatal("terminal refusal mutated the caller-owned prior state")
+	}
+	wantTerminal := cloneRetainedTestState(priorBefore)
+	wantTerminal.Availability = retainedTerminalRetentionFailure
+	if !reflect.DeepEqual(result.State, wantTerminal) {
+		t.Fatalf("terminal refusal did not retain exact prior safe state plus fixed marker: got=%#v want=%#v", result.State, wantTerminal)
+	}
+	terminalImage, err := marshalRunObservationRetainedImage(result.State)
+	if err != nil {
+		t.Fatal(err)
+	}
+	wantImage, err := marshalRunObservationRetainedImage(wantTerminal)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if len(terminalImage) != len(priorImage) || !reflect.DeepEqual(terminalImage, wantImage) {
+		t.Fatalf("terminal safe image changed size or prior safe bytes: prior=%d terminal=%d", len(priorImage), len(terminalImage))
+	}
+	for _, candidateValue := range []string{candidateID, candidateTimestamp, over.Execution, over.Settlement, candidateStatus} {
+		if strings.Contains(string(terminalImage), candidateValue) {
+			t.Fatalf("terminal image retained candidate-only value %q", candidateValue)
+		}
+	}
+	for _, frame := range result.State.Frames {
+		if frame.Frame.EventID == candidateID || (frame.Frame.ExecutionStatus != nil && *frame.Frame.ExecutionStatus == candidateStatus) ||
+			(frame.Frame.ExitCode != nil && *frame.Frame.ExitCode == candidateExitCode) {
+			t.Fatalf("terminal state retained candidate frame data: %#v", frame)
+		}
+	}
+	if _, leaked := result.State.SeenByID[candidateID]; leaked || result.State.Generation != prior.Generation ||
+		result.State.AcceptedAt != prior.AcceptedAt || result.State.SeenByID["safe-old-id"] != 1 || result.State.HighestOrdinal != 1 {
+		t.Fatalf("terminal candidate ID/generation/time/prior ledger mismatch: %#v", result.State)
 	}
 }
 
@@ -180,6 +373,8 @@
 	state := retainedTestState(retainedTestKey(), "opaque-A")
 	state.Generation = maxUint
 	state.HighestOrdinal = maxUint
+	state.SeenByID["opaque-A"] = maxUint
+	state.Frames[0].FirstOrdinal = maxUint
 	base, err := marshalRunObservationRetainedImage(state)
 	if err != nil {
 		t.Fatal(err)
@@ -200,6 +395,16 @@
 		if decoded["generation"] != max || decoded["highest_ordinal"] != max || decoded["availability"] != strconv.Itoa(int(code)) {
 			t.Fatalf("control image fields for code %d = %#v", code, decoded)
 		}
+		frames, framesOK := decoded["frames"].([]any)
+		ledger, ledgerOK := decoded["ledger"].([]any)
+		if !framesOK || len(frames) != 1 || !ledgerOK || len(ledger) != 1 {
+			t.Fatalf("max-counter image frame/ledger shape = %#v/%#v", decoded["frames"], decoded["ledger"])
+		}
+		frame, frameOK := frames[0].(map[string]any)
+		identity, identityOK := ledger[0].(map[string]any)
+		if !frameOK || !identityOK || frame["first_ordinal"] != max || identity["first_ordinal"] != max {
+			t.Fatalf("max-counter image ordinal fields = %#v/%#v", frame, identity)
+		}
 	}
 
 	// Grow only a metadata identity field until its canonical JSON image is
@@ -216,12 +421,35 @@
 		t.Fatal("base image is not smaller than exact boundary")
 	}
 	key.caseID += strings.Repeat("x", padding)
+	exactExpectedState := retainedTestState(key)
+	exactExpectedImage, err := marshalRunObservationRetainedImage(exactExpectedState)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if len(exactExpectedImage) != runObservationRetainedImageMaxBytes {
+		t.Fatalf("computed exact candidate image length = %d, want %d", len(exactExpectedImage), runObservationRetainedImageMaxBytes)
+	}
 	exact := retainedTestSnapshot(key)
 	accepted := buildRunObservationRetainedCandidate(nil, key, exact, retainedTestTime)
 	if accepted.Outcome != retainedCandidateAccepted || accepted.State == nil {
 		t.Fatalf("exact image boundary outcome = %v", accepted.Outcome)
 	}
+	acceptedImage, err := marshalRunObservationRetainedImage(accepted.State)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if len(acceptedImage) != runObservationRetainedImageMaxBytes || !reflect.DeepEqual(acceptedImage, exactExpectedImage) {
+		t.Fatalf("accepted exact candidate image length = %d, want %d", len(acceptedImage), runObservationRetainedImageMaxBytes)
+	}
 	key.caseID += "x"
+	overExpectedState := retainedTestState(key)
+	overExpectedImage, err := marshalRunObservationRetainedImage(overExpectedState)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if len(overExpectedImage) != runObservationRetainedImageMaxBytes+1 {
+		t.Fatalf("computed rejected candidate image length = %d, want %d", len(overExpectedImage), runObservationRetainedImageMaxBytes+1)
+	}
 	over := retainedTestSnapshot(key)
 	rejected := buildRunObservationRetainedCandidate(nil, key, over, retainedTestTime)
 	if rejected.Outcome != retainedCandidateRejected || rejected.State != nil {
@@ -252,9 +480,61 @@
 	maxState.SeenByID["already-seen"] = maxUint
 	maxState.Frames[0].FirstOrdinal = maxUint
 	maxState.Generation = 2
+	maxStateBefore := cloneRetainedTestState(maxState)
+	maxStateImage, err := marshalRunObservationRetainedImage(maxState)
+	if err != nil {
+		t.Fatal(err)
+	}
 	result := buildRunObservationRetainedCandidate(maxState, key, retainedTestSnapshot(key,
 		retainedTestFrame("unseen-after-max", "2026-10-07T12:00:01Z")), retainedTestTime.Add(time.Second))
 	if result.Outcome != retainedCandidateOrdinalOverflow || result.State == nil || result.State.HighestOrdinal != maxUint || result.State.SeenByID["unseen-after-max"] != 0 {
 		t.Fatalf("ordinal overflow was not rejected atomically: %v/%#v", result.Outcome, result.State)
 	}
-}
+	maxStateAfterImage, err := marshalRunObservationRetainedImage(maxState)
+	if err != nil {
+		t.Fatal(err)
+	}
+	wantOrdinalOverflow := cloneRetainedTestState(maxStateBefore)
+	wantOrdinalOverflow.Availability = result.State.Availability
+	if !reflect.DeepEqual(maxState, maxStateBefore) || !reflect.DeepEqual(maxStateImage, maxStateAfterImage) ||
+		!reflect.DeepEqual(result.State, wantOrdinalOverflow) {
+		t.Fatal("ordinal overflow wrapped or partially mutated prior state/ledger")
+	}
+	resultImage, err := marshalRunObservationRetainedImage(result.State)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if len(resultImage) != len(maxStateImage) {
+		t.Fatalf("ordinal overflow marker changed fixed-width image length %d -> %d", len(maxStateImage), len(resultImage))
+	}
+
+	maxGenerationState := retainedTestState(key, "generation-anchor")
+	maxGenerationState.Generation = maxUint
+	maxGenerationBefore := cloneRetainedTestState(maxGenerationState)
+	maxGenerationImage, err := marshalRunObservationRetainedImage(maxGenerationState)
+	if err != nil {
+		t.Fatal(err)
+	}
+	generationResult := buildRunObservationRetainedCandidate(maxGenerationState, key, retainedTestSnapshot(key,
+		retainedTestFrame("new-after-max-generation", "2026-10-07T12:00:02Z")), retainedTestTime.Add(time.Second))
+	if generationResult.Outcome != retainedCandidateGenerationOverflow || generationResult.State == nil || generationResult.State.Generation != maxUint {
+		t.Fatalf("generation overflow result = %v/%#v, want non-wrapping safe prior state", generationResult.Outcome, generationResult.State)
+	}
+	maxGenerationAfterImage, err := marshalRunObservationRetainedImage(maxGenerationState)
+	if err != nil {
+		t.Fatal(err)
+	}
+	wantGenerationOverflow := cloneRetainedTestState(maxGenerationBefore)
+	wantGenerationOverflow.Availability = generationResult.State.Availability
+	if !reflect.DeepEqual(maxGenerationState, maxGenerationBefore) || !reflect.DeepEqual(maxGenerationImage, maxGenerationAfterImage) ||
+		!reflect.DeepEqual(generationResult.State, wantGenerationOverflow) || generationResult.State.SeenByID["new-after-max-generation"] != 0 {
+		t.Fatal("generation overflow wrapped or partially mutated prior state/ledger")
+	}
+	generationResultImage, err := marshalRunObservationRetainedImage(generationResult.State)
+	if err != nil {
+		t.Fatal(err)
+	}
+	if len(generationResultImage) != len(maxGenerationImage) {
+		t.Fatalf("generation overflow marker changed fixed-width image length %d -> %d", len(maxGenerationImage), len(generationResultImage))
+	}
+}
```
