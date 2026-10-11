package server

import (
	"encoding/json"
	"sort"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
)

const runObservationHydrationMaxBytes = 1 << 20

type hydrationFrameCoordinate struct {
	runIndex   int
	frameIndex int
	runID      string
	eventID    string
	instant    time.Time
}

func boundRunObservationHydration(input contract.RunTraceObservation) (contract.RunTraceObservation, error) {
	coordinates, err := hydrationFrameCoordinates(input)
	if err != nil {
		return contract.RunTraceObservation{}, err
	}

	unchanged := cloneRunTraceObservation(input)
	encoded, err := json.Marshal(unchanged)
	if err != nil {
		return contract.RunTraceObservation{}, unavailableHydration("run observation could not be serialized")
	}
	if len(encoded) <= runObservationHydrationMaxBytes {
		return unchanged, nil
	}
	if len(coordinates) == 0 {
		return contract.RunTraceObservation{}, unavailableHydration("run observation metadata exceeds the payload limit")
	}

	sort.Slice(coordinates, func(i, j int) bool {
		left, right := coordinates[i], coordinates[j]
		if !left.instant.Equal(right.instant) {
			return left.instant.Before(right.instant)
		}
		if left.runID != right.runID {
			return left.runID < right.runID
		}
		return left.eventID < right.eventID
	})

	allRemoved, allRemovedBytes, err := observationWithOldestFramesRemoved(input, coordinates, len(coordinates))
	if err != nil {
		return contract.RunTraceObservation{}, err
	}
	if len(allRemovedBytes) > runObservationHydrationMaxBytes {
		return contract.RunTraceObservation{}, unavailableHydration("run observation metadata exceeds the payload limit")
	}

	// Removing one frame always removes its nonempty JSON object and separators.
	// The affected run's decimal retained/omitted counts can add at most one byte,
	// while the frame object is much larger, so encoded size is monotone by prefix.
	low, high := 1, len(coordinates)
	best := allRemoved
	for low < high {
		mid := low + (high-low)/2
		candidate, candidateBytes, err := observationWithOldestFramesRemoved(input, coordinates, mid)
		if err != nil {
			return contract.RunTraceObservation{}, err
		}
		if len(candidateBytes) <= runObservationHydrationMaxBytes {
			high = mid
			best = candidate
		} else {
			low = mid + 1
		}
	}
	return best, nil
}

func hydrationFrameCoordinates(input contract.RunTraceObservation) ([]hydrationFrameCoordinate, error) {
	if len(input.Runs) > 8 {
		return nil, unavailableHydration("run observation exceeds the run bound")
	}
	coordinates := make([]hydrationFrameCoordinate, 0, len(input.Runs)*1024)
	for runIndex, run := range input.Runs {
		if len(run.Frames) > 1024 {
			return nil, unavailableHydration("run observation exceeds the frame bound")
		}
		if run.TotalFrameCount < 0 || run.RetainedFrameCount < 0 || run.OmittedFrameCount < 0 ||
			run.RetainedFrameCount != len(run.Frames) || run.TotalFrameCount < run.RetainedFrameCount ||
			run.OmittedFrameCount != run.TotalFrameCount-run.RetainedFrameCount ||
			run.Truncated != (run.OmittedFrameCount > 0) {
			return nil, unavailableHydration("run observation has inconsistent frame counts")
		}
		for frameIndex, frame := range run.Frames {
			instant, err := time.Parse(time.RFC3339Nano, frame.Timestamp)
			if err != nil {
				return nil, unavailableHydration("run observation has an invalid frame timestamp")
			}
			coordinates = append(coordinates, hydrationFrameCoordinate{
				runIndex: runIndex, frameIndex: frameIndex, runID: run.RunID,
				eventID: frame.EventID, instant: instant,
			})
		}
	}
	return coordinates, nil
}

func observationWithOldestFramesRemoved(
	input contract.RunTraceObservation,
	coordinates []hydrationFrameCoordinate,
	removeCount int,
) (contract.RunTraceObservation, []byte, error) {
	removed := make([][]bool, len(input.Runs))
	for i, run := range input.Runs {
		if run.Frames != nil {
			removed[i] = make([]bool, len(run.Frames))
		}
	}
	for _, coordinate := range coordinates[:removeCount] {
		removed[coordinate.runIndex][coordinate.frameIndex] = true
	}

	output := cloneRunTraceObservation(input)
	for runIndex, run := range input.Runs {
		if run.Frames == nil {
			output.Runs[runIndex].Frames = nil
			output.Runs[runIndex].RetainedFrameCount = 0
			output.Runs[runIndex].OmittedFrameCount = run.TotalFrameCount
			output.Runs[runIndex].Truncated = run.TotalFrameCount > 0
			continue
		}
		frames := make([]contract.RunTraceFrame, 0, len(run.Frames))
		for frameIndex := range run.Frames {
			if !removed[runIndex][frameIndex] {
				frames = append(frames, output.Runs[runIndex].Frames[frameIndex])
			}
		}
		output.Runs[runIndex].Frames = frames
		output.Runs[runIndex].RetainedFrameCount = len(frames)
		output.Runs[runIndex].OmittedFrameCount = run.TotalFrameCount - len(frames)
		output.Runs[runIndex].Truncated = output.Runs[runIndex].OmittedFrameCount > 0
	}

	encoded, err := json.Marshal(output)
	if err != nil {
		return contract.RunTraceObservation{}, nil, unavailableHydration("run observation could not be serialized")
	}
	return output, encoded, nil
}

func cloneRunTraceObservation(input contract.RunTraceObservation) contract.RunTraceObservation {
	output := input
	output.ListedRunCount = cloneHydrationCount(input.ListedRunCount)
	output.SelectedRunCount = cloneHydrationCount(input.SelectedRunCount)
	output.ValidatedRunCount = cloneHydrationCount(input.ValidatedRunCount)
	output.UnreadableRunCount = cloneHydrationCount(input.UnreadableRunCount)
	output.UnavailableRunCount = cloneHydrationCount(input.UnavailableRunCount)
	output.OmittedRunCount = cloneHydrationCount(input.OmittedRunCount)
	if input.Runs != nil {
		output.Runs = make([]contract.RunTraceRunObservation, len(input.Runs))
		copy(output.Runs, input.Runs)
	}
	for runIndex := range input.Runs {
		if input.Runs[runIndex].Frames != nil {
			output.Runs[runIndex].Frames = make([]contract.RunTraceFrame, len(input.Runs[runIndex].Frames))
			copy(output.Runs[runIndex].Frames, input.Runs[runIndex].Frames)
		}
		for frameIndex := range input.Runs[runIndex].Frames {
			frame := input.Runs[runIndex].Frames[frameIndex]
			if frame.ExecutionStatus != nil {
				status := *frame.ExecutionStatus
				output.Runs[runIndex].Frames[frameIndex].ExecutionStatus = &status
			}
			if frame.ExitCode != nil {
				code := *frame.ExitCode
				output.Runs[runIndex].Frames[frameIndex].ExitCode = &code
			}
		}
	}
	return output
}

func cloneHydrationCount(input *int) *int {
	if input == nil {
		return nil
	}
	output := *input
	return &output
}

func unavailableHydration(message string) error {
	return apperr.New(apperr.KindUnavailable, "", "run_observation", message)
}
