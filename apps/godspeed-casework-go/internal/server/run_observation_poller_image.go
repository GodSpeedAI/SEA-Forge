package server

import (
	"encoding/json"
	"errors"
)

const runObservationPollerImageSchemaVersion = "run-observation-poller-v1"

type runObservationPollerPhase uint8

const (
	runObservationPollerPhaseInitializing runObservationPollerPhase = iota
	runObservationPollerPhaseRunning
	runObservationPollerPhaseStopping
	runObservationPollerPhaseDraining
)

type runObservationInitializerResult uint8

const (
	runObservationInitializerPending runObservationInitializerResult = iota
	runObservationInitializerAccepted
	runObservationInitializerReadUnavailable
	runObservationInitializerRetentionUnavailable
	runObservationInitializerInvalid
	runObservationInitializerTerminalRetentionFailure
)

var errRunObservationPollerImageUnavailable = errors.New("run observation poller image unavailable")

type runObservationPollerImageKey struct {
	CaseID     string `json:"case_id"`
	RunID      string `json:"run_id"`
	PlanItemID string `json:"plan_item_id"`
}

// These explicit branches keep nil fields out of the canonical image and
// preserve the helper image as an unquoted JSON object.
type runObservationPollerImageWithoutCurrent struct {
	SchemaVersion string                          `json:"schema_version"`
	Phase         runObservationPollerPhase       `json:"phase"`
	InitResult    runObservationInitializerResult `json:"init_result"`
	Key           runObservationPollerImageKey    `json:"key"`
}

type runObservationPollerImageWithCurrent struct {
	SchemaVersion string                          `json:"schema_version"`
	Phase         runObservationPollerPhase       `json:"phase"`
	InitResult    runObservationInitializerResult `json:"init_result"`
	Current       json.RawMessage                 `json:"current"`
}

// marshalRunObservationPollerImage returns the canonical bounded wrapper for
// one poller state without mutating the retained state.
func marshalRunObservationPollerImage(
	key runObservationPollerKey,
	phase runObservationPollerPhase,
	initResult runObservationInitializerResult,
	current *runObservationRetainedState,
) ([]byte, error) {
	if phase > runObservationPollerPhaseDraining || initResult > runObservationInitializerTerminalRetentionFailure {
		return nil, errRunObservationPollerImageUnavailable
	}

	var image []byte
	var err error
	if current == nil {
		image, err = json.Marshal(runObservationPollerImageWithoutCurrent{
			SchemaVersion: runObservationPollerImageSchemaVersion,
			Phase:         phase,
			InitResult:    initResult,
			Key: runObservationPollerImageKey{
				CaseID: key.caseID, RunID: key.runID, PlanItemID: key.planItemID,
			},
		})
	} else {
		if current.Key != key {
			return nil, errRunObservationPollerImageUnavailable
		}
		inner, innerErr := marshalRunObservationRetainedImage(current)
		if innerErr != nil {
			return nil, errRunObservationPollerImageUnavailable
		}
		image, err = json.Marshal(runObservationPollerImageWithCurrent{
			SchemaVersion: runObservationPollerImageSchemaVersion,
			Phase:         phase,
			InitResult:    initResult,
			Current:       json.RawMessage(inner),
		})
	}
	if err != nil || len(image) > runObservationRetainedImageMaxBytes {
		return nil, errRunObservationPollerImageUnavailable
	}
	return image, nil
}
