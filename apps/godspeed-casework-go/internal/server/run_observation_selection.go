package server

import (
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

const observationRunLimit = 8

// selectObservationRuns keeps the first eight validated, case-scoped summaries
// in the order required by the live observation cohort.
func selectObservationRuns(runs []ports.RunSummary) ([]ports.RunSummary, error) {
	var selected [observationRunLimit]ports.RunSummary
	selectedCount := 0

	for _, run := range runs {
		if observationRunRank(run.Execution) < 0 {
			return nil, apperr.New(apperr.KindUnavailable, "", "run_observation_selection", "the authority returned an unsupported run execution standing")
		}

		insertAt := selectedCount
		for i := 0; i < selectedCount && i < observationRunLimit; i++ {
			if observationRunLess(run, selected[i]) {
				insertAt = i
				break
			}
		}

		if insertAt >= observationRunLimit {
			continue
		}
		last := selectedCount
		if last < observationRunLimit {
			selectedCount++
		} else {
			last = observationRunLimit - 1
		}
		for i := last; i > insertAt; i-- {
			selected[i] = selected[i-1]
		}
		selected[insertAt] = run
	}

	out := make([]ports.RunSummary, selectedCount)
	copy(out, selected[:selectedCount])
	return out, nil
}

func observationRunRank(execution string) int {
	switch execution {
	case "active":
		return 0
	case "pending", "enabled":
		return 1
	case "completed", "failed", "terminated":
		return 2
	default:
		return -1
	}
}

func observationRunLess(left, right ports.RunSummary) bool {
	leftRank := observationRunRank(left.Execution)
	rightRank := observationRunRank(right.Execution)
	if leftRank != rightRank {
		return leftRank < rightRank
	}

	leftAt, leftHasTime := observationRunRecency(left)
	rightAt, rightHasTime := observationRunRecency(right)
	if leftHasTime != rightHasTime {
		return leftHasTime
	}
	if leftHasTime && !leftAt.Equal(rightAt) {
		return leftAt.After(rightAt)
	}
	return left.RunID < right.RunID
}

func observationRunRecency(run ports.RunSummary) (time.Time, bool) {
	if run.HasFinished {
		return run.FinishedAt, true
	}
	if run.HasStarted {
		return run.StartedAt, true
	}
	return time.Time{}, false
}
