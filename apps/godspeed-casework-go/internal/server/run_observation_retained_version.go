package server

import (
	"encoding/json"
	"fmt"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

const (
	runObservationRetainedImageMaxBytes = 1 << 20
	runObservationRetainedFrameLimit    = 1024
)

// retainedAvailability is encoded as a fixed one-digit code in the private
// retained-value image. Do not add candidate-derived error strings here.
type retainedAvailability uint8

const (
	retainedCurrent retainedAvailability = iota
	retainedReadUnavailable
	retainedRetentionUnavailable
	retainedTerminalRetentionFailure
	retainedStopScheduling
)

type retainedObservedFrame struct {
	Frame        ports.RunTraceFrame
	FirstOrdinal uint64
}

// runObservationRetainedState is immutable after publication. Candidate
// construction must deep-copy all frame pointers and the exact-ID ledger.
type runObservationRetainedState struct {
	Key              runObservationPollerKey
	Generation       uint64
	AcceptedAt       string
	Execution        string
	Settlement       string
	ObservationState string
	TotalFrameCount  int
	Frames           []retainedObservedFrame
	SeenByID         map[string]uint64
	HighestOrdinal   uint64
	Availability     retainedAvailability
}

type retainedCandidateOutcome uint8

const (
	retainedCandidateAccepted retainedCandidateOutcome = iota
	retainedCandidateRejected
	retainedCandidateInvalid
	retainedCandidateOrdinalOverflow
	retainedCandidateGenerationOverflow
)

type retainedCandidateResult struct {
	State   *runObservationRetainedState
	Outcome retainedCandidateOutcome
}

// buildRunObservationRetainedCandidate builds a bounded immutable version.
// Its snapshot argument is the safe projection returned by RunTracePort; the
// adapter owns validation of trace enums and frame metadata.
func buildRunObservationRetainedCandidate(
	previous *runObservationRetainedState,
	key runObservationPollerKey,
	snapshot ports.RunTraceSnapshot,
	acceptedAt time.Time,
) retainedCandidateResult {
	if snapshot.CaseID != key.caseID || snapshot.RunID != key.runID || snapshot.PlanItemID != key.planItemID {
		return retainedCandidateResult{Outcome: retainedCandidateInvalid}
	}
	if previous != nil && previous.Key != key {
		return retainedCandidateResult{Outcome: retainedCandidateInvalid}
	}
	if snapshot.TotalFrameCount < 0 || snapshot.TotalFrameCount < len(snapshot.Frames) ||
		len(snapshot.Frames) > runObservationRetainedFrameLimit {
		return retainedCandidateResult{Outcome: retainedCandidateInvalid}
	}
	seenInWindow := make(map[string]struct{}, len(snapshot.Frames))
	for _, frame := range snapshot.Frames {
		if _, exists := seenInWindow[frame.EventID]; exists {
			return retainedCandidateResult{Outcome: retainedCandidateInvalid}
		}
		seenInWindow[frame.EventID] = struct{}{}
	}

	if previous != nil {
		switch previous.Availability {
		case retainedTerminalRetentionFailure, retainedStopScheduling:
			return retainedCandidateResult{
				State: cloneRunObservationRetainedState(previous), Outcome: retainedCandidateRejected,
			}
		}
		if previous.Generation == ^uint64(0) {
			return retainedCandidateResult{
				State: retainedMarkerCopy(previous, retainedStopScheduling), Outcome: retainedCandidateGenerationOverflow,
			}
		}
	}

	ledger := make(map[string]uint64)
	var generation, highest uint64
	if previous == nil {
		generation = 1
	} else {
		generation = previous.Generation + 1
		highest = previous.HighestOrdinal
		for id, ordinal := range previous.SeenByID {
			ledger[id] = ordinal
		}
	}

	frames := make([]retainedObservedFrame, 0, len(snapshot.Frames))
	for _, sourceFrame := range snapshot.Frames {
		ordinal, exists := ledger[sourceFrame.EventID]
		if !exists {
			if highest == ^uint64(0) {
				if previous == nil {
					return retainedCandidateResult{Outcome: retainedCandidateOrdinalOverflow}
				}
				availability := retainedRetentionUnavailable
				if runObservationSnapshotIsTerminal(snapshot) {
					availability = retainedTerminalRetentionFailure
				}
				return retainedCandidateResult{
					State: retainedMarkerCopy(previous, availability), Outcome: retainedCandidateOrdinalOverflow,
				}
			}
			highest++
			ordinal = highest
			ledger[sourceFrame.EventID] = ordinal
		}
		frames = append(frames, retainedObservedFrame{
			Frame: cloneRunTraceFrame(sourceFrame), FirstOrdinal: ordinal,
		})
	}

	candidate := &runObservationRetainedState{
		Key: key, Generation: generation,
		AcceptedAt: acceptedAt.Format(time.RFC3339Nano),
		Execution:  snapshot.Execution, Settlement: snapshot.Settlement,
		ObservationState: "validated", TotalFrameCount: snapshot.TotalFrameCount,
		Frames: frames, SeenByID: ledger, HighestOrdinal: highest,
		Availability: retainedCurrent,
	}
	image, err := marshalRunObservationRetainedImage(candidate)
	if err == nil && len(image) <= runObservationRetainedImageMaxBytes {
		return retainedCandidateResult{State: candidate, Outcome: retainedCandidateAccepted}
	}

	if previous == nil {
		return retainedCandidateResult{Outcome: retainedCandidateRejected}
	}
	availability := retainedRetentionUnavailable
	if runObservationSnapshotIsTerminal(snapshot) {
		availability = retainedTerminalRetentionFailure
	}
	return retainedCandidateResult{
		State: retainedMarkerCopy(previous, availability), Outcome: retainedCandidateRejected,
	}
}

func runObservationSnapshotIsTerminal(snapshot ports.RunTraceSnapshot) bool {
	switch snapshot.Execution {
	case "completed", "failed", "terminated":
		return true
	default:
		return false
	}
}

func cloneRunTraceFrame(frame ports.RunTraceFrame) ports.RunTraceFrame {
	cloned := frame
	if frame.ExecutionStatus != nil {
		status := *frame.ExecutionStatus
		cloned.ExecutionStatus = &status
	}
	if frame.ExitCode != nil {
		code := *frame.ExitCode
		cloned.ExitCode = &code
	}
	return cloned
}

func cloneRunObservationRetainedState(state *runObservationRetainedState) *runObservationRetainedState {
	if state == nil {
		return nil
	}
	cloned := *state
	if state.Frames != nil {
		cloned.Frames = make([]retainedObservedFrame, len(state.Frames))
		for i, frame := range state.Frames {
			cloned.Frames[i] = retainedObservedFrame{
				Frame: cloneRunTraceFrame(frame.Frame), FirstOrdinal: frame.FirstOrdinal,
			}
		}
	}
	if state.SeenByID != nil {
		cloned.SeenByID = make(map[string]uint64, len(state.SeenByID))
		for id, ordinal := range state.SeenByID {
			cloned.SeenByID[id] = ordinal
		}
	}
	return &cloned
}

func retainedMarkerCopy(previous *runObservationRetainedState, availability retainedAvailability) *runObservationRetainedState {
	cloned := cloneRunObservationRetainedState(previous)
	if cloned != nil {
		cloned.Availability = availability
	}
	return cloned
}

type retainedImageFrame struct {
	EventID         string  `json:"event_id"`
	Kind            string  `json:"kind"`
	Timestamp       string  `json:"timestamp"`
	ExecutionStatus *string `json:"execution_status,omitempty"`
	ExitCode        *int64  `json:"exit_code,omitempty"`
	FirstOrdinal    string  `json:"first_ordinal"`
}

type retainedImageIdentity struct {
	EventID      string `json:"event_id"`
	FirstOrdinal string `json:"first_ordinal"`
}

// retainedImage is an explicit ordered accounting shape. Counters with
// lifecycle meaning are decimal strings so their JSON width stays fixed.
type retainedImage struct {
	SchemaVersion      string                  `json:"schema_version"`
	CaseID             string                  `json:"case_id"`
	RunID              string                  `json:"run_id"`
	PlanItemID         string                  `json:"plan_item_id"`
	Generation         string                  `json:"generation"`
	AcceptedAt         string                  `json:"accepted_at"`
	Execution          string                  `json:"execution"`
	Settlement         string                  `json:"settlement"`
	ObservationState   string                  `json:"observation_state"`
	TotalFrameCount    int                     `json:"total_frame_count"`
	RetainedFrameCount int                     `json:"retained_frame_count"`
	OmittedFrameCount  int                     `json:"omitted_frame_count"`
	Truncated          bool                    `json:"truncated"`
	HighestOrdinal     string                  `json:"highest_ordinal"`
	Availability       string                  `json:"availability"`
	Frames             []retainedImageFrame    `json:"frames"`
	Ledger             []retainedImageIdentity `json:"ledger"`
}

func fixedUint64Decimal(value uint64) string {
	decimal := strconv.FormatUint(value, 10)
	return strings.Repeat("0", 20-len(decimal)) + decimal
}

func marshalRunObservationRetainedImage(state *runObservationRetainedState) ([]byte, error) {
	if state == nil {
		return nil, fmt.Errorf("retained state is unavailable")
	}
	if state.Availability > retainedStopScheduling {
		return nil, fmt.Errorf("retained availability code is invalid")
	}
	frames := make([]retainedImageFrame, len(state.Frames))
	for i, retained := range state.Frames {
		frame := retained.Frame
		frames[i] = retainedImageFrame{
			EventID: frame.EventID, Kind: frame.Kind, Timestamp: frame.Timestamp,
			FirstOrdinal: fixedUint64Decimal(retained.FirstOrdinal),
		}
		if frame.ExecutionStatus != nil {
			status := *frame.ExecutionStatus
			frames[i].ExecutionStatus = &status
		}
		if frame.ExitCode != nil {
			code := *frame.ExitCode
			frames[i].ExitCode = &code
		}
	}
	ids := make([]string, 0, len(state.SeenByID))
	for id := range state.SeenByID {
		ids = append(ids, id)
	}
	sort.Strings(ids)
	ledger := make([]retainedImageIdentity, len(ids))
	for i, id := range ids {
		ledger[i] = retainedImageIdentity{EventID: id, FirstOrdinal: fixedUint64Decimal(state.SeenByID[id])}
	}
	omitted := state.TotalFrameCount - len(state.Frames)
	if omitted < 0 {
		omitted = 0
	}
	image := retainedImage{
		SchemaVersion: "run-observation-retained-v1",
		CaseID:        state.Key.caseID, RunID: state.Key.runID, PlanItemID: state.Key.planItemID,
		Generation: fixedUint64Decimal(state.Generation), AcceptedAt: state.AcceptedAt,
		Execution: state.Execution, Settlement: state.Settlement,
		ObservationState: state.ObservationState, TotalFrameCount: state.TotalFrameCount,
		RetainedFrameCount: len(state.Frames), OmittedFrameCount: omitted, Truncated: omitted > 0,
		HighestOrdinal: fixedUint64Decimal(state.HighestOrdinal),
		Availability:   strconv.FormatUint(uint64(state.Availability), 10),
		Frames:         frames, Ledger: ledger,
	}
	return json.Marshal(image)
}
