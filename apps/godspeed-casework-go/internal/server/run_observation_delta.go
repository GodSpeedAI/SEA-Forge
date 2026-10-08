package server

import "github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"

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
	runID                       string
	previousOrdinal             uint64
	currentOrdinal              uint64
	evictedObservedEventIDs     []string
	evictedObservedEventCount   int
	currentWindow               runObservationDeltaWindow
	unobservedLossPossible      bool
	unknownMissedFrameCount     bool
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
	return runObservationRunDelta{}, nil, runObservationWatermark{}, runObservationUnavailable("run observation delta projection is unavailable")
}
