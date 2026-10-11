package server

import (
	"sort"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
)

type runObservationDeltaWindow struct {
	generation         uint64
	totalFrameCount    int
	retainedFrameCount int
	omittedFrameCount  int
	truncated          bool
}

type runObservationRunDelta struct {
	caseID           string
	runID            string
	planItemID       string
	observedAt       string
	execution        contract.RunExecutionStanding
	settlement       contract.RunSettlementStanding
	observationState contract.RunTraceRunObservationState
	frames           []contract.RunTraceFrame
	window           runObservationDeltaWindow
}

type runObservationWindowGap struct {
	runID                     string
	previousOrdinal           uint64
	currentOrdinal            uint64
	evictedObservedEventIDs   []string
	evictedObservedEventCount int
	currentWindow             runObservationDeltaWindow
	unobservedLossPossible    bool
	unknownMissedFrameCount   bool
}

type runObservationWatermark struct {
	highestObservedOrdinal uint64
	execution              contract.RunExecutionStanding
	settlement             contract.RunSettlementStanding
	observationState       contract.RunTraceRunObservationState
	window                 runObservationDeltaWindow
}

func buildRunObservationRunDelta(
	state *runObservationRetainedState,
	copiedLedger map[string]uint64,
	prior runObservationWatermark,
) (runObservationRunDelta, *runObservationWindowGap, runObservationWatermark, error) {
	if state == nil || state.Availability != retainedCurrent ||
		state.TotalFrameCount < 0 || state.TotalFrameCount < len(state.Frames) ||
		len(state.Frames) > runObservationRetainedFrameLimit ||
		prior.highestObservedOrdinal > state.HighestOrdinal {
		return unavailableRunObservationDelta()
	}

	highestOrdinal := state.HighestOrdinal
	ledgerOrdinalIDs := make(map[uint64]string, len(copiedLedger))
	var capturedLedgerCount uint64
	for id, ordinal := range copiedLedger {
		if ordinal == 0 {
			return unavailableRunObservationDelta()
		}
		if ordinal > highestOrdinal {
			// The copied ledger may have been observed after this version was
			// captured. Those identities belong to a later projection.
			continue
		}
		if _, exists := ledgerOrdinalIDs[ordinal]; exists {
			return unavailableRunObservationDelta()
		}
		ledgerOrdinalIDs[ordinal] = id
		capturedLedgerCount++
	}
	if capturedLedgerCount != highestOrdinal {
		return unavailableRunObservationDelta()
	}

	window := runObservationDeltaWindow{
		generation:         state.Generation,
		totalFrameCount:    state.TotalFrameCount,
		retainedFrameCount: len(state.Frames),
		omittedFrameCount:  state.TotalFrameCount - len(state.Frames),
	}
	window.truncated = window.omittedFrameCount > 0

	windowIDs := make(map[string]struct{}, len(state.Frames))
	frames := make([]contract.RunTraceFrame, 0, len(state.Frames))
	for _, retained := range state.Frames {
		frame := retained.Frame
		ordinal := retained.FirstOrdinal
		if ordinal == 0 || ordinal > highestOrdinal {
			return unavailableRunObservationDelta()
		}
		ledgerOrdinal, exists := copiedLedger[frame.EventID]
		if !exists || ledgerOrdinal != ordinal {
			return unavailableRunObservationDelta()
		}
		if _, exists := windowIDs[frame.EventID]; exists {
			return unavailableRunObservationDelta()
		}
		windowIDs[frame.EventID] = struct{}{}

		if ordinal <= prior.highestObservedOrdinal {
			continue
		}

		projected := contract.RunTraceFrame{
			EventID: frame.EventID, Kind: contract.RunTraceFrameKind(frame.Kind), Timestamp: frame.Timestamp,
		}
		if frame.ExecutionStatus != nil {
			status := contract.RunTraceCommandExecutionStatus(*frame.ExecutionStatus)
			projected.ExecutionStatus = &status
		}
		if frame.ExitCode != nil {
			exitCode := *frame.ExitCode
			projected.ExitCode = &exitCode
		}
		frames = append(frames, projected)
	}

	delta := runObservationRunDelta{
		caseID:           state.Key.caseID,
		runID:            state.Key.runID,
		planItemID:       state.Key.planItemID,
		observedAt:       state.AcceptedAt,
		execution:        contract.RunExecutionStanding(state.Execution),
		settlement:       contract.RunSettlementStanding(state.Settlement),
		observationState: contract.RunTraceRunObservationState(state.ObservationState),
		frames:           frames,
		window:           window,
	}
	candidate := runObservationWatermark{
		highestObservedOrdinal: highestOrdinal,
		execution:              delta.execution,
		settlement:             delta.settlement,
		observationState:       delta.observationState,
		window:                 window,
	}

	gapIDs := make([]string, 0)
	for id, ordinal := range copiedLedger {
		if ordinal <= prior.highestObservedOrdinal || ordinal > highestOrdinal {
			continue
		}
		if _, present := windowIDs[id]; !present {
			gapIDs = append(gapIDs, id)
		}
	}
	if len(gapIDs) == 0 {
		return delta, nil, candidate, nil
	}
	sort.Strings(gapIDs)
	gap := &runObservationWindowGap{
		runID:                     state.Key.runID,
		previousOrdinal:           prior.highestObservedOrdinal,
		currentOrdinal:            highestOrdinal,
		evictedObservedEventIDs:   gapIDs,
		evictedObservedEventCount: len(gapIDs),
		currentWindow:             window,
		unobservedLossPossible:    true,
		unknownMissedFrameCount:   true,
	}
	return delta, gap, candidate, nil
}

func unavailableRunObservationDelta() (runObservationRunDelta, *runObservationWindowGap, runObservationWatermark, error) {
	return runObservationRunDelta{}, nil, runObservationWatermark{}, runObservationUnavailable("run observation delta projection is unavailable")
}
