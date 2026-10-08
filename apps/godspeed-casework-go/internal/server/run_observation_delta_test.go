package server

import (
	"reflect"
	"strconv"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
)

func cloneRunObservationDeltaLedger(input map[string]uint64) map[string]uint64 {
	if input == nil {
		return nil
	}
	output := make(map[string]uint64, len(input))
	for id, ordinal := range input {
		output[id] = ordinal
	}
	return output
}

func TestRunObservationDeltaUsesCapturedOrdinalRangeAndSourceOrder(t *testing.T) {
	key := retainedTestKey()
	state := retainedTestState(key, "opaque-P", "opaque-z", "opaque-a")
	state.Generation = 9
	state.AcceptedAt = retainedTestTime.Format(time.RFC3339Nano)
	state.Execution = "completed"
	state.Settlement = "accepted"
	state.ObservationState = "validated"
	state.TotalFrameCount = 7
	state.Frames[1], state.Frames[2] = state.Frames[2], state.Frames[1]
	state.SeenByID["opaque-gap-z"] = 5
	state.SeenByID["opaque-gap-a"] = 4
	state.HighestOrdinal = 5

	ledger := cloneRunObservationDeltaLedger(state.SeenByID)
	ledger["opaque-later"] = 6
	stateBefore := cloneRetainedTestState(state)
	ledgerBefore := cloneRunObservationDeltaLedger(ledger)
	prior := runObservationWatermark{highestObservedOrdinal: 1}

	got, gap, candidate, err := buildRunObservationRunDelta(state, ledger, prior)
	if err != nil {
		t.Fatalf("build delta: %v", err)
	}
	if got.caseID != key.caseID || got.runID != key.runID || got.planItemID != key.planItemID || got.observedAt != state.AcceptedAt {
		t.Fatalf("captured identity/time = %#v", got)
	}
	if got.execution != contract.RunExecutionStanding("completed") ||
		got.settlement != contract.RunSettlementStanding("accepted") ||
		got.observationState != contract.RunTraceRunObservationState("validated") {
		t.Fatalf("captured standings = %#v", got)
	}
	if got.window != (runObservationDeltaWindow{
		generation: 9, totalFrameCount: 7, retainedFrameCount: 3, omittedFrameCount: 4, truncated: true,
	}) {
		t.Fatalf("captured window = %#v", got.window)
	}
	if got.frames == nil || len(got.frames) != 2 || got.frames[0].EventID != "opaque-a" || got.frames[1].EventID != "opaque-z" {
		t.Fatalf("delta frames did not preserve captured source order or P boundary: %#v", got.frames)
	}
	if got.frames[0].Timestamp != state.Frames[1].Frame.Timestamp || got.frames[1].Timestamp != state.Frames[2].Frame.Timestamp {
		t.Fatalf("delta frame payload was not copied from captured source order: %#v", got.frames)
	}
	if gap == nil || gap.runID != key.runID || gap.previousOrdinal != 1 || gap.currentOrdinal != 5 ||
		!reflect.DeepEqual(gap.evictedObservedEventIDs, []string{"opaque-gap-a", "opaque-gap-z"}) ||
		gap.evictedObservedEventCount != 2 || gap.currentWindow != got.window ||
		!gap.unobservedLossPossible || !gap.unknownMissedFrameCount {
		t.Fatalf("captured ordinal gap = %#v", gap)
	}
	if got.frames[0].EventID == "opaque-P" || candidate.highestObservedOrdinal != 5 ||
		candidate.execution != got.execution || candidate.settlement != got.settlement ||
		candidate.observationState != got.observationState || candidate.window != got.window {
		t.Fatalf("watermark or P-boundary result = %#v / %#v", got, candidate)
	}
	if !reflect.DeepEqual(state, stateBefore) || !reflect.DeepEqual(ledger, ledgerBefore) || prior.highestObservedOrdinal != 1 {
		t.Fatal("projection mutated its captured state, copied ledger, or prior watermark")
	}
}

func TestRunObservationDeltaIgnoresLaterWindowAndLedgerAboveCapturedHighWater(t *testing.T) {
	key := retainedTestKey()
	captured := retainedTestState(key, "opaque-before", "opaque-at-capture")
	captured.Generation = 4
	captured.AcceptedAt = retainedTestTime.Format(time.RFC3339Nano)
	captured.TotalFrameCount = 3
	captured.HighestOrdinal = 2

	later := cloneRetainedTestState(captured)
	later.Generation = 5
	later.AcceptedAt = retainedTestTime.Add(time.Minute).Format(time.RFC3339Nano)
	later.TotalFrameCount = 4
	later.HighestOrdinal = 3
	later.SeenByID["opaque-after-capture"] = 3
	later.Frames = append(later.Frames, retainedObservedFrame{
		Frame: retainedTestFrame("opaque-after-capture", "2026-10-08T12:00:00Z"), FirstOrdinal: 3,
	})

	ledger := cloneRunObservationDeltaLedger(captured.SeenByID)
	ledger["opaque-after-capture"] = 3
	got, gap, candidate, err := buildRunObservationRunDelta(captured, ledger, runObservationWatermark{highestObservedOrdinal: 1})
	if err != nil {
		t.Fatalf("build captured delta: %v", err)
	}
	if gap != nil || len(got.frames) != 1 || got.frames[0].EventID != "opaque-at-capture" {
		t.Fatalf("later ledger/window affected captured projection: delta=%#v gap=%#v", got, gap)
	}
	if got.observedAt != captured.AcceptedAt || got.window.generation != captured.Generation ||
		candidate.highestObservedOrdinal != captured.HighestOrdinal || got.window.totalFrameCount != captured.TotalFrameCount {
		t.Fatalf("projection did not preserve captured boundary: %#v / %#v", got, candidate)
	}
	if later.Generation == got.window.generation || later.AcceptedAt == got.observedAt || later.Frames[len(later.Frames)-1].Frame.EventID == got.frames[0].EventID {
		t.Fatal("later publication fixture did not differ from captured state")
	}
}

func TestRunObservationDeltaAdvancesWindowAndStandingsWithoutNewFrames(t *testing.T) {
	key := retainedTestKey()
	base := retainedTestState(key, "opaque-one", "opaque-two")
	base.Generation = 1
	base.TotalFrameCount = 3
	base.HighestOrdinal = 2
	prior := runObservationWatermark{
		highestObservedOrdinal: 2,
		execution:              contract.RunExecutionStanding("active"),
		settlement:             contract.RunSettlementStanding("unsettled"),
		observationState:       contract.RunTraceRunObservationState("validated"),
		window: runObservationDeltaWindow{
			generation: 1, totalFrameCount: 3, retainedFrameCount: 1, omittedFrameCount: 2, truncated: true,
		},
	}

	grown := cloneRetainedTestState(base)
	grown.Generation = 2
	grown.AcceptedAt = retainedTestTime.Add(time.Second).Format(time.RFC3339Nano)
	grown.Execution = "completed"
	grown.Settlement = "accepted"
	grown.TotalFrameCount = 3
	grown.HighestOrdinal = 2
	grownWindow := runObservationDeltaWindow{
		generation: 2, totalFrameCount: 3, retainedFrameCount: 2, omittedFrameCount: 1, truncated: true,
	}
	first, firstGap, grownWatermark, err := buildRunObservationRunDelta(
		grown, cloneRunObservationDeltaLedger(grown.SeenByID), prior,
	)
	if err != nil {
		t.Fatalf("build standing/window-only delta: %v", err)
	}
	if first.frames == nil || len(first.frames) != 0 || firstGap != nil || first.window != grownWindow {
		t.Fatalf("window growth with unchanged H = %#v / %#v", first, firstGap)
	}
	if first.observedAt != grown.AcceptedAt || first.execution != contract.RunExecutionStanding("completed") ||
		first.settlement != contract.RunSettlementStanding("accepted") || grownWatermark.highestObservedOrdinal != 2 ||
		grownWatermark.window != grownWindow || grownWatermark.execution != first.execution || grownWatermark.settlement != first.settlement {
		t.Fatalf("standing/window watermark did not advance: %#v / %#v", first, grownWatermark)
	}

	shrunk := cloneRetainedTestState(grown)
	shrunk.Generation = 3
	shrunk.AcceptedAt = retainedTestTime.Add(2 * time.Second).Format(time.RFC3339Nano)
	shrunk.TotalFrameCount = 5
	shrunk.Frames = []retainedObservedFrame{shrunk.Frames[1]}
	shrunk.HighestOrdinal = 2
	shrunkWindow := runObservationDeltaWindow{
		generation: 3, totalFrameCount: 5, retainedFrameCount: 1, omittedFrameCount: 4, truncated: true,
	}
	second, secondGap, shrunkWatermark, err := buildRunObservationRunDelta(
		shrunk, cloneRunObservationDeltaLedger(shrunk.SeenByID), grownWatermark,
	)
	if err != nil {
		t.Fatalf("build shrinking-window delta: %v", err)
	}
	if second.frames == nil || len(second.frames) != 0 || secondGap != nil || second.window != shrunkWindow ||
		second.observedAt != shrunk.AcceptedAt || shrunkWatermark.highestObservedOrdinal != 2 || shrunkWatermark.window != shrunkWindow {
		t.Fatalf("window shrink with unchanged H = %#v / %#v", second, secondGap)
	}
}

func TestRunObservationDeltaDeepCopiesOptionalFramePointersAndInputs(t *testing.T) {
	key := retainedTestKey()
	state := retainedTestState(key, "opaque/rewrite:does-not-change-ordinal")
	state.Execution = "completed"
	state.Settlement = "accepted"
	state.HighestOrdinal = 1
	stateBefore := cloneRetainedTestState(state)
	ledger := cloneRunObservationDeltaLedger(state.SeenByID)
	ledgerBefore := cloneRunObservationDeltaLedger(ledger)

	got, gap, candidate, err := buildRunObservationRunDelta(state, ledger, runObservationWatermark{})
	if err != nil {
		t.Fatalf("build terminal delta: %v", err)
	}
	if gap != nil || len(got.frames) != 1 || candidate.highestObservedOrdinal != 1 {
		t.Fatalf("terminal current version should be valid: delta=%#v gap=%#v watermark=%#v", got, gap, candidate)
	}
	frame := got.frames[0]
	if frame.EventID != "opaque/rewrite:does-not-change-ordinal" || frame.ExecutionStatus == nil || frame.ExitCode == nil ||
		*frame.ExecutionStatus != contract.RunTraceCommandExecutionStatus("completed") || *frame.ExitCode != 0 {
		t.Fatalf("copied optional frame fields = %#v", frame)
	}
	if frame.ExecutionStatus == nil || state.Frames[0].Frame.ExecutionStatus == nil || frame.ExitCode == state.Frames[0].Frame.ExitCode {
		t.Fatal("delta optional frame pointers are missing or alias retained state")
	}
	*frame.ExecutionStatus = contract.RunTraceCommandExecutionStatus("mutated-output")
	*frame.ExitCode = 73
	if !reflect.DeepEqual(state, stateBefore) || !reflect.DeepEqual(ledger, ledgerBefore) {
		t.Fatal("mutating delta frame pointers changed captured inputs")
	}
}

func TestRunObservationDeltaDoesNotTreatReappearingOpaqueIDAsNew(t *testing.T) {
	key := retainedTestKey()
	state := retainedTestState(key, "opaque/id:with-no-ordering-meaning")
	state.Generation = 3
	state.AcceptedAt = retainedTestTime.Add(time.Minute).Format(time.RFC3339Nano)
	state.Frames[0].Frame.Timestamp = "rewritten-payload-timestamp"
	state.Frames[0].FirstOrdinal = 1
	state.SeenByID["opaque/id:with-no-ordering-meaning"] = 1
	state.HighestOrdinal = 1
	state.TotalFrameCount = 4

	got, gap, watermark, err := buildRunObservationRunDelta(
		state, cloneRunObservationDeltaLedger(state.SeenByID), runObservationWatermark{highestObservedOrdinal: 1},
	)
	if err != nil {
		t.Fatalf("build reappeared-ID delta: %v", err)
	}
	if got.frames == nil || len(got.frames) != 0 || gap != nil || watermark.highestObservedOrdinal != 1 {
		t.Fatalf("rewritten/reappeared opaque ID acquired a new ordinal: %#v / %#v / %#v", got, gap, watermark)
	}
}

func TestRunObservationDeltaRejectsMalformedOrUnavailableCapturesWithoutOutputs(t *testing.T) {
	tests := []struct {
		name         string
		availability retainedAvailability
		bad          string
	}{
		{name: "read-unavailable-marker", availability: retainedReadUnavailable},
		{name: "retention-unavailable-marker", availability: retainedRetentionUnavailable},
		{name: "terminal-retention-failure-marker", availability: retainedTerminalRetentionFailure},
		{name: "stop-scheduling-marker", availability: retainedStopScheduling},
		{name: "invalid-marker", availability: retainedAvailability(255)},
		{name: "nil-state", availability: retainedCurrent, bad: "nil-state"},
		{name: "negative-total", availability: retainedCurrent, bad: "negative-total"},
		{name: "total-less-than-retained", availability: retainedCurrent, bad: "total-less-than-retained"},
		{name: "prior-above-captured-high", availability: retainedCurrent, bad: "prior-above-high"},
		{name: "copied-ledger-missing-frame", availability: retainedCurrent, bad: "missing-frame"},
		{name: "copied-ledger-ordinal-disagrees", availability: retainedCurrent, bad: "ledger-ordinal-disagrees"},
		{name: "frame-above-captured-high", availability: retainedCurrent, bad: "frame-above-high"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			key := retainedTestKey()
			state := retainedTestState(key, "opaque-A")
			state.Availability = tt.availability
			ledger := cloneRunObservationDeltaLedger(state.SeenByID)
			prior := runObservationWatermark{}
			switch tt.bad {
			case "negative-total":
				state.TotalFrameCount = -1
			case "total-less-than-retained":
				state.TotalFrameCount = 0
			case "prior-above-high":
				prior.highestObservedOrdinal = state.HighestOrdinal + 1
			case "missing-frame":
				delete(ledger, "opaque-A")
			case "ledger-ordinal-disagrees":
				ledger["opaque-A"] = 2
				state.HighestOrdinal = 2
			case "frame-above-high":
				state.Frames[0].FirstOrdinal = 2
				ledger["opaque-A"] = 2
			}
			if tt.name == "nil-state" {
				state = nil
				ledger = nil
			}

			got, gap, candidate, err := buildRunObservationRunDelta(state, ledger, prior)
			if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
				t.Fatalf("error = %v, want typed unavailable", err)
			}
			if !reflect.DeepEqual(got, runObservationRunDelta{}) || gap != nil ||
				!reflect.DeepEqual(candidate, runObservationWatermark{}) {
				t.Fatalf("malformed/unavailable input returned partial data: %#v / %#v / %#v", got, gap, candidate)
			}
		})
	}
}

func TestRunObservationDeltaRejectsCapturedFrameWindowAboveRetentionCap(t *testing.T) {
	key := retainedTestKey()
	const count = runObservationRetainedFrameLimit + 1
	state := retainedTestState(key)
	state.Generation = 7
	state.AcceptedAt = retainedTestTime.Format(time.RFC3339Nano)
	state.TotalFrameCount = count
	state.HighestOrdinal = count
	state.Frames = make([]retainedObservedFrame, 0, count)
	state.SeenByID = make(map[string]uint64, count)
	for i := 1; i <= count; i++ {
		id := "over-cap-" + strconv.Itoa(i)
		ordinal := uint64(i)
		state.SeenByID[id] = ordinal
		state.Frames = append(state.Frames, retainedObservedFrame{
			Frame: retainedTestFrame(id, retainedTestTime.Format(time.RFC3339Nano)), FirstOrdinal: ordinal,
		})
	}

	got, gap, candidate, err := buildRunObservationRunDelta(state, cloneRunObservationDeltaLedger(state.SeenByID), runObservationWatermark{})
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("error = %v, want typed unavailable for over-cap captured window", err)
	}
	if !reflect.DeepEqual(got, runObservationRunDelta{}) || gap != nil ||
		!reflect.DeepEqual(candidate, runObservationWatermark{}) {
		t.Fatalf("over-cap captured window returned partial data: %#v / %#v / %#v", got, gap, candidate)
	}
}

func TestRunObservationDeltaAcceptsEmptySuccessfulTerminalCapture(t *testing.T) {
	key := retainedTestKey()
	state := retainedTestState(key)
	state.Execution = "completed"
	state.Settlement = "accepted"
	state.ObservationState = "validated"
	state.TotalFrameCount = 0
	state.HighestOrdinal = 0

	got, gap, watermark, err := buildRunObservationRunDelta(state, cloneRunObservationDeltaLedger(state.SeenByID), runObservationWatermark{})
	if err != nil {
		t.Fatalf("empty successful terminal capture: %v", err)
	}
	if gap != nil || got.frames == nil || len(got.frames) != 0 || got.caseID != key.caseID || got.runID != key.runID ||
		got.observedAt != state.AcceptedAt || got.execution != contract.RunExecutionStanding("completed") ||
		watermark.highestObservedOrdinal != 0 {
		t.Fatalf("empty terminal projection = %#v / %#v / %#v", got, gap, watermark)
	}
}
