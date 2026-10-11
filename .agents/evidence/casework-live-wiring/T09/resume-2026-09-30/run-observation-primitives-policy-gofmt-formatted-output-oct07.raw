package server

import (
	"encoding/json"
	"reflect"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func requireRetainedPolicySafeCopy(
	t *testing.T,
	prior, priorBefore *runObservationRetainedState,
	priorImage []byte,
	got *runObservationRetainedState,
	wantAvailability retainedAvailability,
) {
	t.Helper()
	if prior == nil || priorBefore == nil || got == nil {
		t.Fatalf("prior/pre-call/result state must be nonnil: prior=%#v pre-call=%#v result=%#v", prior, priorBefore, got)
	}
	priorAfterImage, err := marshalRunObservationRetainedImage(prior)
	if err != nil {
		t.Fatal(err)
	}
	gotImage, err := marshalRunObservationRetainedImage(got)
	if err != nil {
		t.Fatal(err)
	}
	want := cloneRetainedTestState(priorBefore)
	want.Availability = wantAvailability
	wantImage, err := marshalRunObservationRetainedImage(want)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(got, want) || !reflect.DeepEqual(gotImage, wantImage) || len(gotImage) != len(priorImage) {
		t.Fatalf("safe marker copy mismatch: got=%#v want=%#v priorBytes=%d gotBytes=%d", got, want, len(priorImage), len(gotImage))
	}
	if !reflect.DeepEqual(prior, priorBefore) || !reflect.DeepEqual(priorAfterImage, priorImage) {
		t.Fatal("candidate construction mutated caller-owned prior state or canonical image")
	}
	if len(prior.Frames) == 0 || len(got.Frames) == 0 || prior.SeenByID == nil || got.SeenByID == nil {
		t.Fatal("safe copy fixture requires frames and ledgers")
	}
	if prior.Frames[0].Frame.ExecutionStatus == nil || got.Frames[0].Frame.ExecutionStatus == nil ||
		prior.Frames[0].Frame.ExitCode == nil || got.Frames[0].Frame.ExitCode == nil {
		t.Fatal("safe copy optional frame pointers must be nonnil")
	}
	if prior.Frames[0].Frame.ExecutionStatus == got.Frames[0].Frame.ExecutionStatus ||
		prior.Frames[0].Frame.ExitCode == got.Frames[0].Frame.ExitCode {
		t.Fatal("result safe state aliases prior optional frame pointers")
	}
	*got.Frames[0].Frame.ExecutionStatus = "result-only"
	*got.Frames[0].Frame.ExitCode = 81
	got.SeenByID["result-only"] = 99
	priorAfterMutationImage, err := marshalRunObservationRetainedImage(prior)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(prior, priorBefore) || !reflect.DeepEqual(priorAfterMutationImage, priorImage) {
		t.Fatal("mutating returned safe copy changed caller-owned prior state or canonical image")
	}
}

func TestRunObservationRetainedPolicyOverflowMarkersAreExactAndAtomic(t *testing.T) {
	key := retainedTestKey()
	maxUint := ^uint64(0)
	makeMaxPrior := func() *runObservationRetainedState {
		prior := retainedTestState(key, "already-seen")
		prior.HighestOrdinal = maxUint
		prior.SeenByID["already-seen"] = maxUint
		prior.Frames[0].FirstOrdinal = maxUint
		return prior
	}
	for _, tc := range []struct {
		name      string
		execution string
		want      retainedAvailability
	}{
		{name: "ordinal_nonterminal", execution: "active", want: retainedRetentionUnavailable},
		{name: "ordinal_terminal", execution: "completed", want: retainedTerminalRetentionFailure},
	} {
		t.Run(tc.name, func(t *testing.T) {
			prior := makeMaxPrior()
			priorBefore := cloneRetainedTestState(prior)
			priorImage, err := marshalRunObservationRetainedImage(prior)
			if err != nil {
				t.Fatal(err)
			}
			candidate := retainedTestSnapshot(key, retainedTestFrame("new-ordinal", "2026-10-07T12:01:00Z"))
			candidate.Execution = tc.execution
			candidate.Settlement = "unsettled"
			result := buildRunObservationRetainedCandidate(prior, key, candidate, retainedTestTime.Add(time.Second))
			if result.Outcome != retainedCandidateOrdinalOverflow {
				t.Fatalf("ordinal overflow outcome=%v, want %v", result.Outcome, retainedCandidateOrdinalOverflow)
			}
			if result.State == nil || result.State.Availability != tc.want {
				t.Fatalf("ordinal overflow state=%#v, want exact availability %v", result.State, tc.want)
			}
			requireRetainedPolicySafeCopy(t, prior, priorBefore, priorImage, result.State, tc.want)
		})
	}

	prior := retainedTestState(key, "generation-anchor")
	prior.Generation = maxUint
	before := cloneRetainedTestState(prior)
	priorImage, err := marshalRunObservationRetainedImage(prior)
	if err != nil {
		t.Fatal(err)
	}
	candidate := retainedTestSnapshot(key, retainedTestFrame("new-generation", "2026-10-07T12:02:00Z"))
	result := buildRunObservationRetainedCandidate(prior, key, candidate, retainedTestTime.Add(time.Second))
	if result.Outcome != retainedCandidateGenerationOverflow || result.State == nil || result.State.Availability != retainedStopScheduling {
		t.Fatalf("generation overflow=%v/%#v, want stop-scheduling safe copy", result.Outcome, result.State)
	}
	if !reflect.DeepEqual(prior, before) || result.State.Generation != before.Generation || result.State.AcceptedAt != before.AcceptedAt {
		t.Fatal("generation overflow changed generation, timestamp, or caller prior")
	}
	resultImage, err := marshalRunObservationRetainedImage(result.State)
	if err != nil {
		t.Fatal(err)
	}
	if len(resultImage) != len(priorImage) {
		t.Fatalf("generation marker image length=%d, prior=%d", len(resultImage), len(priorImage))
	}
	requireRetainedPolicySafeCopy(t, prior, before, priorImage, result.State, retainedStopScheduling)
}

func TestRunObservationRetainedPolicyTerminalityUsesExecutionOnly(t *testing.T) {
	key := retainedTestKey()
	for _, tc := range []struct {
		name       string
		execution  string
		settlement string
		want       retainedAvailability
	}{
		{name: "completed_unsettled", execution: "completed", settlement: "unsettled", want: retainedTerminalRetentionFailure},
		{name: "failed_unsettled", execution: "failed", settlement: "unsettled", want: retainedTerminalRetentionFailure},
		{name: "terminated_unsettled", execution: "terminated", settlement: "unsettled", want: retainedTerminalRetentionFailure},
		{name: "active_accepted", execution: "active", settlement: "accepted", want: retainedRetentionUnavailable},
		{name: "active_rejected", execution: "active", settlement: "rejected", want: retainedRetentionUnavailable},
	} {
		t.Run(tc.name, func(t *testing.T) {
			prior := retainedTestState(key, "safe-before-overflow")
			priorBefore := cloneRetainedTestState(prior)
			priorImage, err := marshalRunObservationRetainedImage(prior)
			if err != nil {
				t.Fatal(err)
			}
			candidateFrame := retainedTestFrame("oversize-"+strings.Repeat("x", runObservationRetainedImageMaxBytes), "2026-10-07T12:03:00Z")
			candidate := retainedTestSnapshot(key, candidateFrame)
			candidate.Execution = tc.execution
			candidate.Settlement = tc.settlement
			result := buildRunObservationRetainedCandidate(prior, key, candidate, retainedTestTime.Add(time.Second))
			if result.Outcome != retainedCandidateRejected || result.State == nil || result.State.Availability != tc.want {
				t.Fatalf("execution/settlement %q/%q gave %v/%#v, want rejected marker %v", tc.execution, tc.settlement, result.Outcome, result.State, tc.want)
			}
			image, err := marshalRunObservationRetainedImage(result.State)
			if err != nil {
				t.Fatal(err)
			}
			if strings.Contains(string(image), candidate.Frames[0].EventID) {
				t.Fatal("over-budget candidate event ID leaked into the safe marker image")
			}
			requireRetainedPolicySafeCopy(t, prior, priorBefore, priorImage, result.State, tc.want)
		})
	}
}

func TestRunObservationRetainedPolicyFinalMarkersRefuseLaterCandidates(t *testing.T) {
	key := retainedTestKey()
	for _, marker := range []retainedAvailability{retainedTerminalRetentionFailure, retainedStopScheduling} {
		t.Run(string(rune('0'+marker)), func(t *testing.T) {
			prior := retainedTestState(key, "safe-final-state")
			prior.Availability = marker
			priorBefore := cloneRetainedTestState(prior)
			priorImage, err := marshalRunObservationRetainedImage(prior)
			if err != nil {
				t.Fatal(err)
			}
			candidate := retainedTestSnapshot(key, retainedTestFrame("fitting-later", "2026-10-07T12:04:00Z"))
			candidate.Execution = "active"
			candidate.Settlement = "unsettled"
			result := buildRunObservationRetainedCandidate(prior, key, candidate, retainedTestTime.Add(time.Second))
			if result.Outcome != retainedCandidateRejected || result.State == nil || result.State.Availability != marker {
				t.Fatalf("final marker %v accepted/changed on later candidate: %v/%#v", marker, result.Outcome, result.State)
			}
			if result.State.Generation != prior.Generation || result.State.AcceptedAt != prior.AcceptedAt ||
				!reflect.DeepEqual(result.State.SeenByID, prior.SeenByID) {
				t.Fatal("final marker refusal changed generation, accepted timestamp, or lifetime ledger")
			}
			requireRetainedPolicySafeCopy(t, prior, priorBefore, priorImage, result.State, marker)
		})
	}
}

func TestRunObservationRetainedPolicyPreservesResponseCounts(t *testing.T) {
	key := retainedTestKey()
	snapshot := retainedTestSnapshot(key, retainedTestFrame("window-tail", "2026-10-07T12:05:00Z"))
	snapshot.TotalFrameCount = 5
	first := requireAcceptedRetainedCandidate(t, buildRunObservationRetainedCandidate(nil, key, snapshot, retainedTestTime))
	assertCounts := func(state *runObservationRetainedState, total, retained, omitted int, truncated bool) {
		t.Helper()
		image, err := marshalRunObservationRetainedImage(state)
		if err != nil {
			t.Fatal(err)
		}
		var counts struct {
			Total     int  `json:"total_frame_count"`
			Retained  int  `json:"retained_frame_count"`
			Omitted   int  `json:"omitted_frame_count"`
			Truncated bool `json:"truncated"`
		}
		if err := json.Unmarshal(image, &counts); err != nil {
			t.Fatal(err)
		}
		if counts.Total != total || counts.Retained != retained || counts.Omitted != omitted || counts.Truncated != truncated {
			t.Fatalf("encoded response counts=%+v, want total/retained/omitted/truncated=%d/%d/%d/%t", counts, total, retained, omitted, truncated)
		}
	}
	if len(first.SeenByID) != 1 || first.TotalFrameCount != 5 {
		t.Fatalf("source total=%d and lifetime ledger size=%d were conflated", first.TotalFrameCount, len(first.SeenByID))
	}
	assertCounts(first, 5, 1, 4, true)

	later := retainedTestSnapshot(key, retainedTestFrame("window-tail", "2026-10-07T12:05:00Z"))
	later.TotalFrameCount = 2
	second := requireAcceptedRetainedCandidate(t, buildRunObservationRetainedCandidate(first, key, later, retainedTestTime.Add(time.Second)))
	if second.Generation != first.Generation+1 || second.TotalFrameCount != 2 {
		t.Fatalf("lower later source total was not accepted: generation=%d total=%d", second.Generation, second.TotalFrameCount)
	}
	assertCounts(second, 2, 1, 1, true)
}

func TestRunObservationRetainedPolicyRecoveryAndInputRejection(t *testing.T) {
	key := retainedTestKey()
	prior := retainedTestState(key, "safe-recoverable")
	prior.Availability = retainedReadUnavailable
	accepted := requireAcceptedRetainedCandidate(t, buildRunObservationRetainedCandidate(prior, key,
		retainedTestSnapshot(key, retainedTestFrame("safe-recoverable", "2026-10-07T12:06:00Z")), retainedTestTime.Add(time.Second)))
	if accepted.Availability != retainedCurrent {
		t.Fatalf("read-unavailable recovery availability=%v, want current", accepted.Availability)
	}

	otherKey := runObservationPollerKey{caseID: "case-other", runID: key.runID, planItemID: key.planItemID}
	validSnapshot := retainedTestSnapshot(key)
	priorMismatch := retainedTestState(otherKey, "prior-key")
	for _, tc := range []struct {
		name      string
		previous  *runObservationRetainedState
		requested runObservationPollerKey
		snapshot  ports.RunTraceSnapshot
	}{
		{name: "requested_snapshot_key_mismatch", requested: key, snapshot: retainedTestSnapshot(otherKey)},
		{name: "prior_requested_key_mismatch", previous: priorMismatch, requested: key, snapshot: validSnapshot},
	} {
		t.Run(tc.name, func(t *testing.T) {
			var before *runObservationRetainedState
			if tc.previous != nil {
				before = cloneRetainedTestState(tc.previous)
			}
			result := buildRunObservationRetainedCandidate(tc.previous, tc.requested, tc.snapshot, retainedTestTime)
			if result.Outcome != retainedCandidateInvalid || result.State != nil {
				t.Fatalf("mismatched key outcome/state=%v/%#v, want invalid/nil", result.Outcome, result.State)
			}
			if tc.previous != nil && !reflect.DeepEqual(tc.previous, before) {
				t.Fatal("invalid key input mutated prior state")
			}
		})
	}

	for _, total := range []int{-1, 0} {
		t.Run("invalid_total_"+string(rune('0'+(-total))), func(t *testing.T) {
			snapshot := retainedTestSnapshot(key, retainedTestFrame("one-frame", "2026-10-07T12:07:00Z"))
			snapshot.TotalFrameCount = total
			result := buildRunObservationRetainedCandidate(nil, key, snapshot, retainedTestTime)
			if result.Outcome != retainedCandidateInvalid || result.State != nil {
				t.Fatalf("total %d outcome/state=%v/%#v, want invalid/nil", total, result.Outcome, result.State)
			}
		})
	}
	belowWindow := retainedTestSnapshot(key, retainedTestFrame("one-frame", "2026-10-07T12:07:00Z"))
	belowWindow.TotalFrameCount = 0
	result := buildRunObservationRetainedCandidate(nil, key, belowWindow, time.Time{})
	if result.Outcome != retainedCandidateInvalid || result.State != nil {
		t.Fatalf("total below window outcome/state=%v/%#v, want invalid/nil", result.Outcome, result.State)
	}
}
