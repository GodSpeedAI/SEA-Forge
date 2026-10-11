package server

import (
	"encoding/json"
	"reflect"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

var retainedTestTime = time.Date(2026, 10, 7, 12, 30, 45, 123456789, time.UTC)

func retainedTestKey() runObservationPollerKey {
	return runObservationPollerKey{caseID: "case-1", runID: "run-1", planItemID: "item-1"}
}

func retainedTestFrame(id, timestamp string) ports.RunTraceFrame {
	status := "completed"
	code := int64(0)
	return ports.RunTraceFrame{
		EventID: id, Kind: "command_finished", Timestamp: timestamp,
		ExecutionStatus: &status, ExitCode: &code,
	}
}

func retainedTestSnapshot(key runObservationPollerKey, frames ...ports.RunTraceFrame) ports.RunTraceSnapshot {
	owned := append([]ports.RunTraceFrame(nil), frames...)
	return ports.RunTraceSnapshot{
		CaseID: key.caseID, RunID: key.runID, PlanItemID: key.planItemID,
		Execution: "active", Settlement: "unsettled", Frames: owned,
		TotalFrameCount: len(owned),
	}
}

func retainedTestState(key runObservationPollerKey, ids ...string) *runObservationRetainedState {
	state := &runObservationRetainedState{
		Key: key, Generation: 1, AcceptedAt: retainedTestTime.Format(time.RFC3339Nano),
		Execution: "active", Settlement: "unsettled", ObservationState: "validated",
		TotalFrameCount: len(ids), SeenByID: make(map[string]uint64),
		Availability: retainedCurrent,
	}
	for _, id := range ids {
		state.HighestOrdinal++
		state.SeenByID[id] = state.HighestOrdinal
		state.Frames = append(state.Frames, retainedObservedFrame{
			Frame: retainedTestFrame(id, "2026-10-07T12:00:00Z"), FirstOrdinal: state.HighestOrdinal,
		})
	}
	return state
}

func cloneRetainedTestState(input *runObservationRetainedState) *runObservationRetainedState {
	if input == nil {
		return nil
	}
	output := *input
	if input.Frames != nil {
		output.Frames = make([]retainedObservedFrame, len(input.Frames))
		for i, frame := range input.Frames {
			output.Frames[i] = frame
			if frame.Frame.ExecutionStatus != nil {
				status := *frame.Frame.ExecutionStatus
				output.Frames[i].Frame.ExecutionStatus = &status
			}
			if frame.Frame.ExitCode != nil {
				code := *frame.Frame.ExitCode
				output.Frames[i].Frame.ExitCode = &code
			}
		}
	}
	if input.SeenByID != nil {
		output.SeenByID = make(map[string]uint64, len(input.SeenByID))
		for id, ordinal := range input.SeenByID {
			output.SeenByID[id] = ordinal
		}
	}
	return &output
}

func requireAcceptedRetainedCandidate(t *testing.T, result retainedCandidateResult) *runObservationRetainedState {
	t.Helper()
	if result.Outcome != retainedCandidateAccepted || result.State == nil {
		t.Fatalf("candidate outcome/state = %v/%#v, want accepted/non-nil", result.Outcome, result.State)
	}
	return result.State
}

func TestRunObservationRetainedCandidateCopiesPointersAndCapturesWindow(t *testing.T) {
	key := retainedTestKey()
	snapshot := retainedTestSnapshot(key, retainedTestFrame("opaque-A", "2026-10-07T12:00:00Z"))
	result := buildRunObservationRetainedCandidate(nil, key, snapshot, retainedTestTime)
	state := requireAcceptedRetainedCandidate(t, result)
	if state.AcceptedAt != retainedTestTime.Format(time.RFC3339Nano) || state.HighestOrdinal != 1 || state.Generation != 1 {
		t.Fatalf("accepted timestamp/high-water/generation = %q/%d/%d", state.AcceptedAt, state.HighestOrdinal, state.Generation)
	}
	if len(state.Frames) != 1 || state.Frames[0].FirstOrdinal != 1 || state.Frames[0].Frame.EventID != "opaque-A" {
		t.Fatalf("captured frame window = %#v", state.Frames)
	}
	if state.Frames[0].Frame.ExecutionStatus == nil || state.Frames[0].Frame.ExitCode == nil {
		t.Fatal("retained optional frame pointers are nil")
	}
	if state.Frames[0].Frame.ExecutionStatus == snapshot.Frames[0].ExecutionStatus || state.Frames[0].Frame.ExitCode == snapshot.Frames[0].ExitCode {
		t.Fatal("retained optional frame pointers alias the source snapshot")
	}
	*state.Frames[0].Frame.ExecutionStatus = "failed"
	*state.Frames[0].Frame.ExitCode = 9
	if *snapshot.Frames[0].ExecutionStatus != "completed" || *snapshot.Frames[0].ExitCode != 0 {
		t.Fatal("mutating retained frame changed source snapshot metadata")
	}
}

func TestRunObservationRetainedCandidateClonesPriorAndSourcePointers(t *testing.T) {
	key := retainedTestKey()
	prior := requireAcceptedRetainedCandidate(t, buildRunObservationRetainedCandidate(nil, key,
		retainedTestSnapshot(key, retainedTestFrame("opaque-A", "2026-10-07T12:00:00Z")), retainedTestTime))
	source := retainedTestSnapshot(key, retainedTestFrame("opaque-A", "2026-10-07T12:01:00Z"))
	result := buildRunObservationRetainedCandidate(prior, key, source, retainedTestTime.Add(time.Second))
	next := requireAcceptedRetainedCandidate(t, result)
	if len(prior.Frames) != 1 || len(next.Frames) != 1 || len(source.Frames) != 1 {
		t.Fatalf("pointer fixture lengths prior/next/source = %d/%d/%d", len(prior.Frames), len(next.Frames), len(source.Frames))
	}
	priorFrame := prior.Frames[0].Frame
	sourceFrame := source.Frames[0]
	nextFrame := next.Frames[0].Frame
	if priorFrame.ExecutionStatus == nil || priorFrame.ExitCode == nil || sourceFrame.ExecutionStatus == nil || sourceFrame.ExitCode == nil || nextFrame.ExecutionStatus == nil || nextFrame.ExitCode == nil {
		t.Fatal("prior, source, and result optional pointers must all be nonnil before alias checks")
	}
	if nextFrame.ExecutionStatus == priorFrame.ExecutionStatus || nextFrame.ExitCode == priorFrame.ExitCode ||
		nextFrame.ExecutionStatus == sourceFrame.ExecutionStatus || nextFrame.ExitCode == sourceFrame.ExitCode {
		t.Fatal("result optional frame pointers alias prior state or source snapshot")
	}
	*nextFrame.ExecutionStatus = "result-only-status"
	*nextFrame.ExitCode = 73
	if *priorFrame.ExecutionStatus != "completed" || *priorFrame.ExitCode != 0 ||
		*sourceFrame.ExecutionStatus != "completed" || *sourceFrame.ExitCode != 0 {
		t.Fatal("mutating result frame pointers changed prior or source values")
	}
}

func TestRunObservationRetainedCandidateKeepsFirstOrdinalsAcrossReorderAndShorterWindow(t *testing.T) {
	key := retainedTestKey()
	first := buildRunObservationRetainedCandidate(nil, key, retainedTestSnapshot(key,
		retainedTestFrame("opaque-A", "2026-10-07T12:00:00Z"),
		retainedTestFrame("opaque-B", "2026-10-07T12:00:01Z")), retainedTestTime)
	prior := requireAcceptedRetainedCandidate(t, first)
	priorBefore := cloneRetainedTestState(prior)
	priorImage, err := marshalRunObservationRetainedImage(prior)
	if err != nil {
		t.Fatal(err)
	}
	second := buildRunObservationRetainedCandidate(prior, key, retainedTestSnapshot(key,
		retainedTestFrame("opaque-B", "2026-10-07T12:00:01Z"),
		retainedTestFrame("opaque-A", "2026-10-07T12:05:00Z"),
		retainedTestFrame("opaque-C", "2026-10-07T12:00:02Z")), retainedTestTime.Add(time.Second))
	reordered := requireAcceptedRetainedCandidate(t, second)
	priorAfterImage, err := marshalRunObservationRetainedImage(prior)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(prior, priorBefore) || !reflect.DeepEqual(priorImage, priorAfterImage) {
		t.Fatal("accepted reorder mutated the prior published state")
	}
	if len(reordered.Frames) != 3 {
		t.Fatalf("reordered source window contains %d frames, want 3", len(reordered.Frames))
	}
	reorderedBefore := cloneRetainedTestState(reordered)
	reorderedImage, err := marshalRunObservationRetainedImage(reordered)
	if err != nil {
		t.Fatal(err)
	}
	wantIDs := []string{"opaque-B", "opaque-A", "opaque-C"}
	wantOrdinals := []uint64{2, 1, 3}
	for i, frame := range reordered.Frames {
		if frame.Frame.EventID != wantIDs[i] || frame.FirstOrdinal != wantOrdinals[i] {
			t.Fatalf("frame[%d] = %q/%d, want %q/%d", i, frame.Frame.EventID, frame.FirstOrdinal, wantIDs[i], wantOrdinals[i])
		}
	}
	if reordered.HighestOrdinal != 3 || reordered.Generation != 2 || !reflect.DeepEqual(reordered.SeenByID, map[string]uint64{"opaque-A": 1, "opaque-B": 2, "opaque-C": 3}) {
		t.Fatalf("ledger/high-water/generation = %#v/%d/%d", reordered.SeenByID, reordered.HighestOrdinal, reordered.Generation)
	}
	shorter := buildRunObservationRetainedCandidate(reordered, key, retainedTestSnapshot(key,
		retainedTestFrame("opaque-C", "2026-10-07T12:00:02Z")), retainedTestTime.Add(2*time.Second))
	short := requireAcceptedRetainedCandidate(t, shorter)
	reorderedAfterImage, err := marshalRunObservationRetainedImage(reordered)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(reordered, reorderedBefore) || !reflect.DeepEqual(reorderedImage, reorderedAfterImage) {
		t.Fatal("accepted shorter window mutated the prior published state")
	}
	shortBefore := cloneRetainedTestState(short)
	shortImage, err := marshalRunObservationRetainedImage(short)
	if err != nil {
		t.Fatal(err)
	}
	if len(short.Frames) != 1 || short.Frames[0].FirstOrdinal != 3 || len(short.SeenByID) != 3 || short.HighestOrdinal != 3 || short.Generation != 3 {
		t.Fatalf("shorter source window lost ledger/high-water: %#v", short)
	}
	reappeared := buildRunObservationRetainedCandidate(short, key, retainedTestSnapshot(key,
		retainedTestFrame("opaque-A", "2026-10-07T12:06:00Z"),
		retainedTestFrame("opaque-C", "2026-10-07T12:00:02Z")), retainedTestTime.Add(3*time.Second))
	reappearedState := requireAcceptedRetainedCandidate(t, reappeared)
	if len(reappearedState.Frames) != 2 || reappearedState.Frames[0].Frame.EventID != "opaque-A" || reappearedState.Frames[0].FirstOrdinal != 1 ||
		reappearedState.Frames[1].Frame.EventID != "opaque-C" || reappearedState.Frames[1].FirstOrdinal != 3 ||
		reappearedState.HighestOrdinal != 3 || reappearedState.Generation != 4 || !reflect.DeepEqual(reappearedState.SeenByID, map[string]uint64{"opaque-A": 1, "opaque-B": 2, "opaque-C": 3}) {
		t.Fatalf("disappeared ID did not reappear in source order with its first ordinal: %#v", reappearedState)
	}
	shortAfterImage, err := marshalRunObservationRetainedImage(short)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(short, shortBefore) || !reflect.DeepEqual(shortImage, shortAfterImage) {
		t.Fatal("accepted reappearance mutated the prior published state")
	}
}

func TestRunObservationRetainedCandidateRejectsDuplicateIDsInOneWindow(t *testing.T) {
	key := retainedTestKey()
	prior := retainedTestState(key, "opaque-existing")
	before, err := marshalRunObservationRetainedImage(prior)
	if err != nil {
		t.Fatal(err)
	}
	duplicate := buildRunObservationRetainedCandidate(prior, key, retainedTestSnapshot(key,
		retainedTestFrame("duplicate-id", "2026-10-07T12:00:00Z"),
		retainedTestFrame("duplicate-id", "2026-10-07T12:00:01Z")), retainedTestTime.Add(time.Second))
	if duplicate.Outcome != retainedCandidateInvalid || duplicate.State != nil {
		t.Fatalf("duplicate IDs in one trace window = %v/%#v, want invalid/nil without selecting a winner", duplicate.Outcome, duplicate.State)
	}
	after, err := marshalRunObservationRetainedImage(prior)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(prior, retainedTestState(key, "opaque-existing")) || !reflect.DeepEqual(before, after) {
		t.Fatal("invalid duplicate window mutated the prior state or canonical image")
	}
}

func TestRunObservationRetainedCandidateRejectsBudgetAtomicallyAndRecovers(t *testing.T) {
	key := retainedTestKey()
	prior := retainedTestState(key, "opaque-A", "opaque-B")
	priorBefore := cloneRetainedTestState(prior)
	before, err := marshalRunObservationRetainedImage(prior)
	if err != nil {
		t.Fatal(err)
	}
	oversizedID := "new-" + strings.Repeat("x", runObservationRetainedImageMaxBytes)
	over := retainedTestSnapshot(key, retainedTestFrame(oversizedID, "2026-10-07T12:00:03Z"))
	result := buildRunObservationRetainedCandidate(prior, key, over, retainedTestTime.Add(time.Second))
	if result.Outcome != retainedCandidateRejected || result.State == nil || result.State.Availability != retainedRetentionUnavailable {
		t.Fatalf("over-budget outcome/state = %v/%#v", result.Outcome, result.State)
	}
	priorAfter, err := marshalRunObservationRetainedImage(prior)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(prior, priorBefore) || !reflect.DeepEqual(before, priorAfter) {
		t.Fatal("over-budget rejection mutated the caller-owned prior state or canonical image")
	}
	wantRefusal := cloneRetainedTestState(priorBefore)
	wantRefusal.Availability = retainedRetentionUnavailable
	if !reflect.DeepEqual(result.State, wantRefusal) {
		t.Fatalf("nonterminal refusal changed prior safe state beyond its fixed marker: got=%#v want=%#v", result.State, wantRefusal)
	}
	after, err := marshalRunObservationRetainedImage(result.State)
	if err != nil {
		t.Fatal(err)
	}
	wantAfter, err := marshalRunObservationRetainedImage(wantRefusal)
	if err != nil {
		t.Fatal(err)
	}
	if len(after) > runObservationRetainedImageMaxBytes || len(after) != len(before) || !reflect.DeepEqual(after, wantAfter) {
		t.Fatalf("fixed-width refusal image changed size or non-marker bytes: before=%d after=%d", len(before), len(after))
	}
	if _, acceptedCandidate := result.State.SeenByID[oversizedID]; acceptedCandidate {
		t.Fatal("rejected candidate ID entered the ledger")
	}
	if len(before) == 0 || result.State.Generation != prior.Generation || result.State.AcceptedAt != prior.AcceptedAt ||
		result.State.Execution != prior.Execution || result.State.Settlement != prior.Settlement ||
		result.State.ObservationState != prior.ObservationState || result.State.TotalFrameCount != prior.TotalFrameCount ||
		!reflect.DeepEqual(result.State.Frames, prior.Frames) || !reflect.DeepEqual(result.State.SeenByID, prior.SeenByID) ||
		result.State.HighestOrdinal != prior.HighestOrdinal {
		t.Fatal("nonterminal refusal changed prior generation, metadata, frame window, or lifetime ledger")
	}

	recovery := buildRunObservationRetainedCandidate(result.State, key, retainedTestSnapshot(key,
		retainedTestFrame("opaque-A", "2026-10-07T12:00:00Z"),
		retainedTestFrame("opaque-B", "2026-10-07T12:00:01Z"),
		retainedTestFrame("new-after-recovery", "2026-10-07T12:00:04Z")), retainedTestTime.Add(2*time.Second))
	recovered := requireAcceptedRetainedCandidate(t, recovery)
	if recovered.Availability != retainedCurrent || recovered.HighestOrdinal != 3 || recovered.SeenByID["new-after-recovery"] != 3 {
		t.Fatalf("recovery did not preserve full ledger atomically: %#v", recovered)
	}
	wantLedger := map[string]uint64{"opaque-A": 1, "opaque-B": 2, "new-after-recovery": 3}
	if !reflect.DeepEqual(recovered.SeenByID, wantLedger) || recovered.Generation != prior.Generation+1 ||
		recovered.AcceptedAt != retainedTestTime.Add(2*time.Second).Format(time.RFC3339Nano) ||
		len(recovered.Frames) != 3 || recovered.Frames[0].Frame.EventID != "opaque-A" || recovered.Frames[0].FirstOrdinal != 1 ||
		recovered.Frames[1].Frame.EventID != "opaque-B" || recovered.Frames[1].FirstOrdinal != 2 ||
		recovered.Frames[2].Frame.EventID != "new-after-recovery" || recovered.Frames[2].FirstOrdinal != 3 {
		t.Fatalf("recovery ledger/window/generation/time = %#v/%#v/%d/%q", recovered.SeenByID, recovered.Frames, recovered.Generation, recovered.AcceptedAt)
	}
}

func TestRunObservationRetainedCandidateTerminalRefusalKeepsOnlySafePriorState(t *testing.T) {
	key := retainedTestKey()
	prior := retainedTestState(key, "safe-old-id")
	priorBefore := cloneRetainedTestState(prior)
	priorImage, err := marshalRunObservationRetainedImage(prior)
	if err != nil {
		t.Fatal(err)
	}
	candidateID := "candidate-secret-" + strings.Repeat("i", runObservationRetainedImageMaxBytes)
	candidateStatus := "timed_out"
	candidateExitCode := int64(91)
	candidateTimestamp := "2026-10-07T12:00:03Z"
	overFrame := retainedTestFrame(candidateID, candidateTimestamp)
	overFrame.ExecutionStatus = &candidateStatus
	overFrame.ExitCode = &candidateExitCode
	over := retainedTestSnapshot(key, overFrame)
	over.Execution = "failed"
	over.Settlement = "rejected"
	over.TotalFrameCount = 2
	result := buildRunObservationRetainedCandidate(prior, key, over, retainedTestTime.Add(time.Second))
	if result.Outcome != retainedCandidateRejected || result.State == nil || result.State.Availability != retainedTerminalRetentionFailure {
		t.Fatalf("terminal over-budget state = %v/%#v", result.Outcome, result.State)
	}
	if !reflect.DeepEqual(prior, priorBefore) {
		t.Fatal("terminal refusal mutated the caller-owned prior state")
	}
	wantTerminal := cloneRetainedTestState(priorBefore)
	wantTerminal.Availability = retainedTerminalRetentionFailure
	if !reflect.DeepEqual(result.State, wantTerminal) {
		t.Fatalf("terminal refusal did not retain exact prior safe state plus fixed marker: got=%#v want=%#v", result.State, wantTerminal)
	}
	terminalImage, err := marshalRunObservationRetainedImage(result.State)
	if err != nil {
		t.Fatal(err)
	}
	wantImage, err := marshalRunObservationRetainedImage(wantTerminal)
	if err != nil {
		t.Fatal(err)
	}
	if len(terminalImage) != len(priorImage) || !reflect.DeepEqual(terminalImage, wantImage) {
		t.Fatalf("terminal safe image changed size or prior safe bytes: prior=%d terminal=%d", len(priorImage), len(terminalImage))
	}
	for _, candidateValue := range []string{candidateID, candidateTimestamp, over.Execution, over.Settlement, candidateStatus} {
		if strings.Contains(string(terminalImage), candidateValue) {
			t.Fatalf("terminal image retained candidate-only value %q", candidateValue)
		}
	}
	for _, frame := range result.State.Frames {
		if frame.Frame.EventID == candidateID || (frame.Frame.ExecutionStatus != nil && *frame.Frame.ExecutionStatus == candidateStatus) ||
			(frame.Frame.ExitCode != nil && *frame.Frame.ExitCode == candidateExitCode) {
			t.Fatalf("terminal state retained candidate frame data: %#v", frame)
		}
	}
	if _, leaked := result.State.SeenByID[candidateID]; leaked || result.State.Generation != prior.Generation ||
		result.State.AcceptedAt != prior.AcceptedAt || result.State.SeenByID["safe-old-id"] != 1 || result.State.HighestOrdinal != 1 {
		t.Fatalf("terminal candidate ID/generation/time/prior ledger mismatch: %#v", result.State)
	}
}

func TestRunObservationRetainedImageFixedWidthControlsAndExactLimit(t *testing.T) {
	maxUint := ^uint64(0)
	max := fixedUint64Decimal(maxUint)
	if len(max) != 20 || max != "18446744073709551615" {
		t.Fatalf("max uint64 fixed-width decimal = %q (%d bytes)", max, len(max))
	}
	state := retainedTestState(retainedTestKey(), "opaque-A")
	state.Generation = maxUint
	state.HighestOrdinal = maxUint
	state.SeenByID["opaque-A"] = maxUint
	state.Frames[0].FirstOrdinal = maxUint
	base, err := marshalRunObservationRetainedImage(state)
	if err != nil {
		t.Fatal(err)
	}
	for code := retainedCurrent; code <= retainedStopScheduling; code++ {
		state.Availability = code
		marked, err := marshalRunObservationRetainedImage(state)
		if err != nil {
			t.Fatal(err)
		}
		if len(base) != len(marked) {
			t.Fatalf("control code %d changed fixed-width image size %d -> %d", code, len(base), len(marked))
		}
		var decoded map[string]any
		if err := json.Unmarshal(marked, &decoded); err != nil {
			t.Fatal(err)
		}
		if decoded["generation"] != max || decoded["highest_ordinal"] != max || decoded["availability"] != strconv.Itoa(int(code)) {
			t.Fatalf("control image fields for code %d = %#v", code, decoded)
		}
		frames, framesOK := decoded["frames"].([]any)
		ledger, ledgerOK := decoded["ledger"].([]any)
		if !framesOK || len(frames) != 1 || !ledgerOK || len(ledger) != 1 {
			t.Fatalf("max-counter image frame/ledger shape = %#v/%#v", decoded["frames"], decoded["ledger"])
		}
		frame, frameOK := frames[0].(map[string]any)
		identity, identityOK := ledger[0].(map[string]any)
		if !frameOK || !identityOK || frame["first_ordinal"] != max || identity["first_ordinal"] != max {
			t.Fatalf("max-counter image ordinal fields = %#v/%#v", frame, identity)
		}
	}

	// Grow only a metadata identity field until its canonical JSON image is
	// exactly the bound; one more byte must produce a refusal, not truncation.
	key := retainedTestKey()
	baseState := retainedTestState(key)
	baseState.AcceptedAt = retainedTestTime.Format(time.RFC3339Nano)
	baseImage, err := marshalRunObservationRetainedImage(baseState)
	if err != nil {
		t.Fatal(err)
	}
	padding := runObservationRetainedImageMaxBytes - len(baseImage)
	if padding <= 0 {
		t.Fatal("base image is not smaller than exact boundary")
	}
	key.caseID += strings.Repeat("x", padding)
	exactExpectedState := retainedTestState(key)
	exactExpectedImage, err := marshalRunObservationRetainedImage(exactExpectedState)
	if err != nil {
		t.Fatal(err)
	}
	if len(exactExpectedImage) != runObservationRetainedImageMaxBytes {
		t.Fatalf("computed exact candidate image length = %d, want %d", len(exactExpectedImage), runObservationRetainedImageMaxBytes)
	}
	exact := retainedTestSnapshot(key)
	accepted := buildRunObservationRetainedCandidate(nil, key, exact, retainedTestTime)
	if accepted.Outcome != retainedCandidateAccepted || accepted.State == nil {
		t.Fatalf("exact image boundary outcome = %v", accepted.Outcome)
	}
	acceptedImage, err := marshalRunObservationRetainedImage(accepted.State)
	if err != nil {
		t.Fatal(err)
	}
	if len(acceptedImage) != runObservationRetainedImageMaxBytes || !reflect.DeepEqual(acceptedImage, exactExpectedImage) {
		t.Fatalf("accepted exact candidate image length = %d, want %d", len(acceptedImage), runObservationRetainedImageMaxBytes)
	}
	key.caseID += "x"
	overExpectedState := retainedTestState(key)
	overExpectedImage, err := marshalRunObservationRetainedImage(overExpectedState)
	if err != nil {
		t.Fatal(err)
	}
	if len(overExpectedImage) != runObservationRetainedImageMaxBytes+1 {
		t.Fatalf("computed rejected candidate image length = %d, want %d", len(overExpectedImage), runObservationRetainedImageMaxBytes+1)
	}
	over := retainedTestSnapshot(key)
	rejected := buildRunObservationRetainedCandidate(nil, key, over, retainedTestTime)
	if rejected.Outcome != retainedCandidateRejected || rejected.State != nil {
		t.Fatalf("irreducible metadata over limit outcome/state = %v/%#v", rejected.Outcome, rejected.State)
	}
}

func TestRunObservationRetainedCandidateBoundsWindowAndOrdinalOverflow(t *testing.T) {
	key := retainedTestKey()
	maxUint := ^uint64(0)
	frames := make([]ports.RunTraceFrame, runObservationRetainedFrameLimit)
	for i := range frames {
		frames[i] = retainedTestFrame("event-"+strconv.Itoa(i), "2026-10-07T12:00:00Z")
	}
	window := buildRunObservationRetainedCandidate(nil, key, retainedTestSnapshot(key, frames...), retainedTestTime)
	state := requireAcceptedRetainedCandidate(t, window)
	if len(state.Frames) != runObservationRetainedFrameLimit || state.HighestOrdinal != runObservationRetainedFrameLimit {
		t.Fatalf("exact 1,024-frame window/high-water = %d/%d", len(state.Frames), state.HighestOrdinal)
	}
	tooMany := append(append([]ports.RunTraceFrame(nil), frames...), retainedTestFrame("event-over-limit", "2026-10-07T12:00:01Z"))
	rejectedWindow := buildRunObservationRetainedCandidate(nil, key, retainedTestSnapshot(key, tooMany...), retainedTestTime)
	if rejectedWindow.Outcome != retainedCandidateInvalid || rejectedWindow.State != nil {
		t.Fatalf("1,025-frame source window outcome/state = %v/%#v", rejectedWindow.Outcome, rejectedWindow.State)
	}

	maxState := retainedTestState(key, "already-seen")
	maxState.HighestOrdinal = maxUint
	maxState.SeenByID["already-seen"] = maxUint
	maxState.Frames[0].FirstOrdinal = maxUint
	maxState.Generation = 2
	maxStateBefore := cloneRetainedTestState(maxState)
	maxStateImage, err := marshalRunObservationRetainedImage(maxState)
	if err != nil {
		t.Fatal(err)
	}
	result := buildRunObservationRetainedCandidate(maxState, key, retainedTestSnapshot(key,
		retainedTestFrame("unseen-after-max", "2026-10-07T12:00:01Z")), retainedTestTime.Add(time.Second))
	if result.Outcome != retainedCandidateOrdinalOverflow || result.State == nil || result.State.HighestOrdinal != maxUint || result.State.SeenByID["unseen-after-max"] != 0 {
		t.Fatalf("ordinal overflow was not rejected atomically: %v/%#v", result.Outcome, result.State)
	}
	maxStateAfterImage, err := marshalRunObservationRetainedImage(maxState)
	if err != nil {
		t.Fatal(err)
	}
	wantOrdinalOverflow := cloneRetainedTestState(maxStateBefore)
	wantOrdinalOverflow.Availability = result.State.Availability
	if !reflect.DeepEqual(maxState, maxStateBefore) || !reflect.DeepEqual(maxStateImage, maxStateAfterImage) ||
		!reflect.DeepEqual(result.State, wantOrdinalOverflow) {
		t.Fatal("ordinal overflow wrapped or partially mutated prior state/ledger")
	}
	resultImage, err := marshalRunObservationRetainedImage(result.State)
	if err != nil {
		t.Fatal(err)
	}
	if len(resultImage) != len(maxStateImage) {
		t.Fatalf("ordinal overflow marker changed fixed-width image length %d -> %d", len(maxStateImage), len(resultImage))
	}

	maxGenerationState := retainedTestState(key, "generation-anchor")
	maxGenerationState.Generation = maxUint
	maxGenerationBefore := cloneRetainedTestState(maxGenerationState)
	maxGenerationImage, err := marshalRunObservationRetainedImage(maxGenerationState)
	if err != nil {
		t.Fatal(err)
	}
	generationResult := buildRunObservationRetainedCandidate(maxGenerationState, key, retainedTestSnapshot(key,
		retainedTestFrame("new-after-max-generation", "2026-10-07T12:00:02Z")), retainedTestTime.Add(time.Second))
	if generationResult.Outcome != retainedCandidateGenerationOverflow || generationResult.State == nil || generationResult.State.Generation != maxUint {
		t.Fatalf("generation overflow result = %v/%#v, want non-wrapping safe prior state", generationResult.Outcome, generationResult.State)
	}
	maxGenerationAfterImage, err := marshalRunObservationRetainedImage(maxGenerationState)
	if err != nil {
		t.Fatal(err)
	}
	wantGenerationOverflow := cloneRetainedTestState(maxGenerationBefore)
	wantGenerationOverflow.Availability = generationResult.State.Availability
	if !reflect.DeepEqual(maxGenerationState, maxGenerationBefore) || !reflect.DeepEqual(maxGenerationImage, maxGenerationAfterImage) ||
		!reflect.DeepEqual(generationResult.State, wantGenerationOverflow) || generationResult.State.SeenByID["new-after-max-generation"] != 0 {
		t.Fatal("generation overflow wrapped or partially mutated prior state/ledger")
	}
	generationResultImage, err := marshalRunObservationRetainedImage(generationResult.State)
	if err != nil {
		t.Fatal(err)
	}
	if len(generationResultImage) != len(maxGenerationImage) {
		t.Fatalf("generation overflow marker changed fixed-width image length %d -> %d", len(maxGenerationImage), len(generationResultImage))
	}
}
