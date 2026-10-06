package server

import (
	"encoding/json"
	"reflect"
	"strings"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
)

const hydrationTestMaxBytes = 1 << 20

func hydrationCount(value int) *int { return &value }

func hydrationFrame(eventID, timestamp string) contract.RunTraceFrame {
	status := contract.RunTraceCommandExecutionStatus("completed")
	exitCode := int64(0)
	return contract.RunTraceFrame{
		EventID: eventID, Kind: contract.RunTraceFrameKind("command_finished"), Timestamp: timestamp,
		ExecutionStatus: &status, ExitCode: &exitCode,
	}
}

func hydrationRun(runID string, frames ...contract.RunTraceFrame) contract.RunTraceRunObservation {
	ownedFrames := make([]contract.RunTraceFrame, len(frames))
	copy(ownedFrames, frames)
	return contract.RunTraceRunObservation{
		RunID: runID, ObservedAt: "2026-10-06T12:00:00Z", PlanItemID: "item-1",
		Execution: contract.RunExecutionStanding("active"), Settlement: contract.RunSettlementStanding("unsettled"),
		ObservationState: contract.RunTraceRunObservationState("validated"), Frames: ownedFrames,
		TotalFrameCount: len(frames), RetainedFrameCount: len(frames),
	}
}

func hydrationObservation(runs ...contract.RunTraceRunObservation) contract.RunTraceObservation {
	ownedRuns := make([]contract.RunTraceRunObservation, len(runs))
	copy(ownedRuns, runs)
	selected := len(runs)
	if selected > 8 {
		selected = 8
	}
	return contract.RunTraceObservation{
		CaseID: "case-1", ObservedAt: "2026-10-06T12:00:00Z",
		RunListState:     contract.RunTraceListState("complete"),
		ObservationState: contract.RunTraceObservationState("complete"),
		ListedRunCount:   hydrationCount(len(runs)), SelectedRunCount: hydrationCount(selected),
		ValidatedRunCount: hydrationCount(selected), UnreadableRunCount: hydrationCount(0),
		UnavailableRunCount: hydrationCount(0), OmittedRunCount: hydrationCount(len(runs) - selected),
		HydrationReadBudget: contract.RunTraceHydrationReadBudget{Limit: 8, ReadsAttempted: selected},
		Runs:                ownedRuns,
	}
}

func cloneHydrationObservation(input contract.RunTraceObservation) contract.RunTraceObservation {
	output := input
	cloneCount := func(value *int) *int {
		if value == nil {
			return nil
		}
		copy := *value
		return &copy
	}
	output.ListedRunCount = cloneCount(input.ListedRunCount)
	output.SelectedRunCount = cloneCount(input.SelectedRunCount)
	output.ValidatedRunCount = cloneCount(input.ValidatedRunCount)
	output.UnreadableRunCount = cloneCount(input.UnreadableRunCount)
	output.UnavailableRunCount = cloneCount(input.UnavailableRunCount)
	output.OmittedRunCount = cloneCount(input.OmittedRunCount)
	if input.Runs != nil {
		output.Runs = make([]contract.RunTraceRunObservation, len(input.Runs))
		copy(output.Runs, input.Runs)
	}
	for i := range output.Runs {
		if input.Runs[i].Frames != nil {
			output.Runs[i].Frames = make([]contract.RunTraceFrame, len(input.Runs[i].Frames))
			copy(output.Runs[i].Frames, input.Runs[i].Frames)
		}
		for j := range output.Runs[i].Frames {
			if status := input.Runs[i].Frames[j].ExecutionStatus; status != nil {
				copy := *status
				output.Runs[i].Frames[j].ExecutionStatus = &copy
			}
			if code := input.Runs[i].Frames[j].ExitCode; code != nil {
				copy := *code
				output.Runs[i].Frames[j].ExitCode = &copy
			}
		}
	}
	return output
}

func hydrationJSON(t *testing.T, observation contract.RunTraceObservation) []byte {
	t.Helper()
	encoded, err := json.Marshal(observation)
	if err != nil {
		t.Fatalf("marshal observation: %v", err)
	}
	return encoded
}

func padHydrationToJSONSize(t *testing.T, observation contract.RunTraceObservation, target int) contract.RunTraceObservation {
	t.Helper()
	current := len(hydrationJSON(t, observation))
	if current > target {
		t.Fatalf("fixture is already %d bytes; cannot pad to %d", current, target)
	}
	observation.CaseID += strings.Repeat("x", target-current)
	if got := len(hydrationJSON(t, observation)); got != target {
		t.Fatalf("padded fixture size=%d, want %d", got, target)
	}
	return observation
}

func requireHydrationUnavailable(t *testing.T, observation contract.RunTraceObservation) {
	t.Helper()
	original := cloneHydrationObservation(observation)
	got, err := boundRunObservationHydration(observation)
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("boundRunObservationHydration error=%v, want typed unavailable", err)
	}
	if !reflect.DeepEqual(got, contract.RunTraceObservation{}) {
		t.Fatalf("failure returned partial observation: %+v", got)
	}
	if !reflect.DeepEqual(observation, original) {
		t.Fatal("failure mutated caller input")
	}
}

func requireHydrationSuccess(t *testing.T, observation contract.RunTraceObservation) contract.RunTraceObservation {
	t.Helper()
	got, err := boundRunObservationHydration(observation)
	if err != nil {
		t.Fatalf("boundRunObservationHydration: %v", err)
	}
	if size := len(hydrationJSON(t, got)); size > hydrationTestMaxBytes {
		t.Fatalf("serialized payload=%d bytes, exceeds %d", size, hydrationTestMaxBytes)
	}
	return got
}

func TestBoundRunObservationHydrationPreservesEmptyAndShortPayloads(t *testing.T) {
	for _, input := range []contract.RunTraceObservation{
		hydrationObservation(),
		hydrationObservation(hydrationRun("run-1", hydrationFrame("evt-1", "2026-10-06T11:59:00Z"))),
	} {
		got := requireHydrationSuccess(t, input)
		if !reflect.DeepEqual(got, input) {
			t.Fatalf("short payload changed:\n got: %+v\nwant: %+v", got, input)
		}
	}
}

func TestBoundRunObservationHydrationExactCapAndCapPlusOne(t *testing.T) {
	t.Run("exact cap remains unchanged", func(t *testing.T) {
		input := padHydrationToJSONSize(t, hydrationObservation(
			hydrationRun("run-1", hydrationFrame("evt-1", "2026-10-06T11:59:00Z"))), hydrationTestMaxBytes)
		got := requireHydrationSuccess(t, input)
		if !reflect.DeepEqual(got, input) {
			t.Fatal("exact-cap payload was modified")
		}
		if size := len(hydrationJSON(t, got)); size != hydrationTestMaxBytes {
			t.Fatalf("serialized payload=%d, want exact cap %d", size, hydrationTestMaxBytes)
		}
	})

	t.Run("cap plus one omits the explicitly oldest frame", func(t *testing.T) {
		input := hydrationObservation(hydrationRun("run-1",
			hydrationFrame("evt-new", "2026-10-06T11:59:00Z"),
			hydrationFrame("evt-old", "2026-10-06T10:00:00Z")))
		input = padHydrationToJSONSize(t, input, hydrationTestMaxBytes+1)
		original := cloneHydrationObservation(input)
		got := requireHydrationSuccess(t, input)
		if len(got.Runs) != 1 || len(got.Runs[0].Frames) != 1 || got.Runs[0].Frames[0].EventID != "evt-new" {
			t.Fatalf("retained frames=%+v, want only evt-new", got.Runs)
		}
		if got.Runs[0].TotalFrameCount != 2 || got.Runs[0].RetainedFrameCount != 1 ||
			got.Runs[0].OmittedFrameCount != 1 || !got.Runs[0].Truncated {
			t.Fatalf("frame counts after omission=%+v", got.Runs[0])
		}
		if size := len(hydrationJSON(t, got)); size > hydrationTestMaxBytes {
			t.Fatalf("serialized payload=%d, exceeds cap %d", size, hydrationTestMaxBytes)
		}
		if !reflect.DeepEqual(input, original) {
			t.Fatal("trimming mutated caller input")
		}
		if &got.Runs[0] == &input.Runs[0] || &got.Runs[0].Frames[0] == &input.Runs[0].Frames[0] {
			t.Fatal("trimmed run/frame arrays alias caller storage")
		}
		got.Runs[0].RunID = "mutated-run"
		got.Runs[0].Frames[0].EventID = "mutated-event"
		*got.Runs[0].Frames[0].ExecutionStatus = contract.RunTraceCommandExecutionStatus("timed_out")
		*got.Runs[0].Frames[0].ExitCode = 9
		for _, count := range []*int{got.ListedRunCount, got.SelectedRunCount, got.ValidatedRunCount,
			got.UnreadableRunCount, got.UnavailableRunCount, got.OmittedRunCount} {
			if count == nil {
				t.Fatal("successful trimmed payload lost a cohort count")
			}
			*count++
		}
		if !reflect.DeepEqual(input, original) {
			t.Fatal("mutating trimmed output changed caller input")
		}
	})
}

func TestBoundRunObservationHydrationSelectsOldestDeterministically(t *testing.T) {
	tests := []struct {
		name  string
		input contract.RunTraceObservation
		want  map[string][]string
	}{
		{
			name: "global timestamp order preserves unsorted per-run order",
			input: hydrationObservation(hydrationRun("run-z",
				hydrationFrame("evt-new", "2026-10-06T12:00:00Z"),
				hydrationFrame("evt-old", "2026-10-06T10:00:00Z"),
				hydrationFrame("evt-mid", "2026-10-06T11:00:00Z"))),
			want: map[string][]string{"run-z": {"evt-new", "evt-mid"}},
		},
		{
			name: "equal instants with differing offsets use complete run ID bytes",
			input: hydrationObservation(
				hydrationRun("run-10", hydrationFrame("evt-10", "2026-10-06T13:00:00+01:00")),
				hydrationRun("run-1", hydrationFrame("evt-1", "2026-10-06T12:00:00Z"))),
			want: map[string][]string{"run-10": {"evt-10"}, "run-1": {}},
		},
		{
			name: "equal run and instant use complete event ID bytes",
			input: hydrationObservation(hydrationRun("run-1",
				hydrationFrame("evt-z", "2026-10-06T12:00:00Z"),
				hydrationFrame("evt-a", "2026-10-06T12:00:00+00:00"))),
			want: map[string][]string{"run-1": {"evt-z"}},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			input := padHydrationToJSONSize(t, test.input, hydrationTestMaxBytes+1)
			got := requireHydrationSuccess(t, input)
			if len(got.Runs) != len(test.want) {
				t.Fatalf("run count=%d, want %d", len(got.Runs), len(test.want))
			}
			for i, run := range got.Runs {
				if run.RunID != test.input.Runs[i].RunID {
					t.Fatalf("run order[%d]=%q, want %q", i, run.RunID, test.input.Runs[i].RunID)
				}
				wantIDs, ok := test.want[run.RunID]
				if !ok {
					t.Fatalf("unexpected run %q", run.RunID)
				}
				gotIDs := make([]string, len(run.Frames))
				for i, frame := range run.Frames {
					gotIDs[i] = frame.EventID
				}
				if !reflect.DeepEqual(gotIDs, wantIDs) {
					t.Fatalf("run %q frame order=%v, want %v", run.RunID, gotIDs, wantIDs)
				}
			}
		})
	}
}

func TestBoundRunObservationHydrationOversizedFramesAcrossRuns(t *testing.T) {
	runs := []contract.RunTraceRunObservation{
		hydrationRun("run-old", hydrationFrame("old-"+strings.Repeat("x", 500_000), "2026-10-06T01:00:00Z")),
	}
	want := make(map[string][]string, 7)
	want["run-old"] = []string{}
	timestamps := []string{
		"2026-10-06T02:00:00Z", "2026-10-06T03:00:00Z", "2026-10-06T04:00:00Z",
		"2026-10-06T05:00:00Z", "2026-10-06T06:00:00Z", "2026-10-06T07:00:00Z", "2026-10-06T08:00:00Z",
	}
	for i := 0; i < 7; i++ {
		runID := "run-" + strings.Repeat("r", i+1)
		eventID := "kept-" + strings.Repeat("y", 100_000)
		runs = append(runs, hydrationRun(runID, hydrationFrame(eventID, timestamps[i])))
		want[runID] = []string{eventID}
	}
	input := hydrationObservation(runs...)
	if size := len(hydrationJSON(t, input)); size <= hydrationTestMaxBytes {
		t.Fatalf("fixture size=%d, want naturally oversized frames across runs", size)
	}
	got := requireHydrationSuccess(t, input)
	if len(got.Runs) != len(want) {
		t.Fatalf("retained runs=%d, want %d after removing the globally oldest frame", len(got.Runs), len(want))
	}
	for i, run := range got.Runs {
		wantIDs, ok := want[run.RunID]
		if !ok {
			t.Fatalf("run[%d] has unexpected identity %q", i, run.RunID)
		}
		gotIDs := make([]string, len(run.Frames))
		for j, frame := range run.Frames {
			gotIDs[j] = frame.EventID
		}
		if !reflect.DeepEqual(gotIDs, wantIDs) {
			t.Fatalf("run[%d] %q frames=%v, want %v", i, run.RunID, gotIDs, wantIDs)
		}
	}

	trimmed := got.Runs[0]
	original := input.Runs[0]
	if trimmed.RunID != original.RunID || trimmed.ObservedAt != original.ObservedAt ||
		trimmed.PlanItemID != original.PlanItemID || trimmed.Execution != original.Execution ||
		trimmed.Settlement != original.Settlement || trimmed.ObservationState != original.ObservationState {
		t.Fatalf("emptied run lost identity/standing/observation metadata: got %+v want %+v", trimmed, original)
	}
	if trimmed.TotalFrameCount != original.TotalFrameCount || trimmed.RetainedFrameCount != 0 ||
		trimmed.OmittedFrameCount != original.OmittedFrameCount+1 || !trimmed.Truncated || len(trimmed.Frames) != 0 {
		t.Fatalf("emptied run counts=%+v, want total %d, retained 0, omitted %d, truncated", trimmed,
			original.TotalFrameCount, original.OmittedFrameCount+1)
	}
	for i := 1; i < len(got.Runs); i++ {
		retained, expected := got.Runs[i], input.Runs[i]
		if retained.RunID != expected.RunID || len(retained.Frames) != len(expected.Frames) ||
			retained.TotalFrameCount != expected.TotalFrameCount ||
			retained.RetainedFrameCount != expected.RetainedFrameCount ||
			retained.OmittedFrameCount != expected.OmittedFrameCount || retained.Truncated != expected.Truncated {
			t.Fatalf("unaffected run[%d] frame/count metadata changed: got %+v want %+v", i, retained, expected)
		}
		if !reflect.DeepEqual(retained, expected) {
			t.Fatalf("unaffected run[%d] metadata or frames changed: got %+v want %+v", i, retained, expected)
		}
	}
}

func TestBoundRunObservationHydrationPreservesRingOmissionsAndMetadata(t *testing.T) {
	run := hydrationRun("run-1",
		hydrationFrame("evt-old", "2026-10-06T10:00:00Z"),
		hydrationFrame("evt-new", "2026-10-06T11:00:00Z"))
	run.TotalFrameCount, run.RetainedFrameCount, run.OmittedFrameCount, run.Truncated = 5, 2, 3, true
	input := hydrationObservation(run, hydrationRun("run-2"))
	input = padHydrationToJSONSize(t, input, hydrationTestMaxBytes+1)
	want := cloneHydrationObservation(input)
	want.Runs[0].Frames = want.Runs[0].Frames[1:]
	want.Runs[0].RetainedFrameCount = 1
	want.Runs[0].OmittedFrameCount = 4
	got := requireHydrationSuccess(t, input)
	if len(got.Runs) != 2 || got.Runs[0].RunID != "run-1" || got.Runs[1].RunID != "run-2" {
		t.Fatalf("run ordering/identities changed: %+v", got.Runs)
	}
	first := got.Runs[0]
	if len(first.Frames) != 1 || first.Frames[0].EventID != "evt-new" || first.TotalFrameCount != 5 ||
		first.RetainedFrameCount != 1 || first.OmittedFrameCount != 4 || !first.Truncated {
		t.Fatalf("ring and hydration counts=%+v, want total 5, retained 1, omitted 4, truncated", first)
	}
	if !reflect.DeepEqual(got.Runs[1], input.Runs[1]) {
		t.Fatalf("unaffected run changed: got %+v want %+v", got.Runs[1], input.Runs[1])
	}
	if got.CaseID != input.CaseID || got.ObservedAt != input.ObservedAt || got.RunListState != input.RunListState ||
		got.ObservationState != input.ObservationState || got.HydrationReadBudget != input.HydrationReadBudget {
		t.Fatalf("cohort metadata changed: %+v", got)
	}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("payload fields differ beyond the single oldest-frame omission:\n got: %+v\nwant: %+v", got, want)
	}
	for name, pair := range map[string][2]*int{
		"listed": {got.ListedRunCount, input.ListedRunCount}, "selected": {got.SelectedRunCount, input.SelectedRunCount},
		"validated": {got.ValidatedRunCount, input.ValidatedRunCount}, "unreadable": {got.UnreadableRunCount, input.UnreadableRunCount},
		"unavailable": {got.UnavailableRunCount, input.UnavailableRunCount}, "omitted": {got.OmittedRunCount, input.OmittedRunCount},
	} {
		if pair[0] == nil || pair[1] == nil || *pair[0] != *pair[1] {
			t.Errorf("%s cohort count changed: got %v, want %v", name, pair[0], pair[1])
		}
	}
}

func TestBoundRunObservationHydrationDoesNotAliasCallerData(t *testing.T) {
	input := hydrationObservation(hydrationRun("run-1", hydrationFrame("evt-1", "2026-10-06T11:00:00Z")))
	original := cloneHydrationObservation(input)
	got := requireHydrationSuccess(t, input)
	if len(got.Runs) != 1 || len(got.Runs[0].Frames) != 1 {
		t.Fatalf("returned payload lost run/frame data: %+v", got)
	}
	if &got.Runs[0] == &input.Runs[0] || &got.Runs[0].Frames[0] == &input.Runs[0].Frames[0] {
		t.Fatal("returned run/frame slices alias caller storage")
	}
	got.Runs[0].RunID = "changed-run"
	got.Runs[0].Frames[0].EventID = "changed-event"
	*got.Runs[0].Frames[0].ExecutionStatus = contract.RunTraceCommandExecutionStatus("timed_out")
	*got.Runs[0].Frames[0].ExitCode = 9
	for _, count := range []*int{got.ListedRunCount, got.SelectedRunCount, got.ValidatedRunCount,
		got.UnreadableRunCount, got.UnavailableRunCount, got.OmittedRunCount} {
		if count == nil {
			t.Fatal("successful payload lost a cohort count")
		}
		*count++
	}
	if !reflect.DeepEqual(input, original) {
		t.Fatalf("mutating returned payload changed caller input:\n got: %+v\nwant: %+v", input, original)
	}
}

func TestBoundRunObservationHydrationRejectsUnrepresentableInputs(t *testing.T) {
	t.Run("metadata alone exceeds cap", func(t *testing.T) {
		input := hydrationObservation()
		input.CaseID = strings.Repeat("x", hydrationTestMaxBytes)
		requireHydrationUnavailable(t, input)
	})
	t.Run("more than eight runs", func(t *testing.T) {
		runs := make([]contract.RunTraceRunObservation, 9)
		for i := range runs {
			runs[i] = hydrationRun("run-" + strings.Repeat("x", i+1))
		}
		input := hydrationObservation(runs...)
		if input.HydrationReadBudget.Limit != 8 || input.HydrationReadBudget.ReadsAttempted != 8 ||
			*input.SelectedRunCount != 8 || *input.ValidatedRunCount != 8 || *input.OmittedRunCount != 1 {
			t.Fatalf("nine-run fixture has unrelated invalid metadata: %+v", input)
		}
		requireHydrationUnavailable(t, input)
	})
	t.Run("more than 1024 frames in one run", func(t *testing.T) {
		frames := make([]contract.RunTraceFrame, 1025)
		for i := range frames {
			frames[i] = hydrationFrame("evt-"+strings.Repeat("x", i+1), "2026-10-06T11:00:00Z")
		}
		requireHydrationUnavailable(t, hydrationObservation(hydrationRun("run-1", frames...)))
	})
	t.Run("inconsistent frame counts", func(t *testing.T) {
		run := hydrationRun("run-1", hydrationFrame("evt-1", "2026-10-06T11:00:00Z"))
		run.TotalFrameCount = 2
		requireHydrationUnavailable(t, hydrationObservation(run))
	})
	t.Run("unparseable timestamp", func(t *testing.T) {
		requireHydrationUnavailable(t, hydrationObservation(hydrationRun("run-1", hydrationFrame("evt-1", "yesterday"))))
	})
}
