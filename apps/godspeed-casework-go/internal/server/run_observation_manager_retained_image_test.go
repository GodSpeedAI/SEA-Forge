package server

import (
	"bytes"
	"encoding/json"
	"reflect"
	"strings"
	"testing"
)

func TestRunObservationPollerImageNilCurrentHasOnlyKeyAndControls(t *testing.T) {
	key := retainedTestKey()
	got, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseInitializing, runObservationInitializerPending, nil,
	)
	if err != nil {
		t.Fatalf("marshal nil-current image: %v", err)
	}
	want := `{"schema_version":"run-observation-poller-v1","phase":0,"init_result":0,"key":{"case_id":"case-1","run_id":"run-1","plan_item_id":"item-1"}}`
	if string(got) != want {
		t.Fatalf("nil-current image = %s, want exact ordered image %s", got, want)
	}
	if bytes.Count(got, []byte(`"case_id"`)) != 1 || bytes.Contains(got, []byte(`"current"`)) {
		t.Fatalf("nil-current key/current shape is wrong: %s", got)
	}
	for _, forbidden := range []string{"execution", "settlement", "accepted_at", "frames", "ledger"} {
		if bytes.Contains(got, []byte(`"`+forbidden+`"`)) {
			t.Fatalf("nil-current image contains synthetic helper field %q: %s", forbidden, got)
		}
	}
}

func TestRunObservationPollerImageCurrentIsRawObjectAndKeyAppearsOnce(t *testing.T) {
	key := retainedTestKey()
	state := retainedTestState(key, "opaque-A")
	first, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseRunning, runObservationInitializerAccepted, state,
	)
	if err != nil {
		t.Fatalf("marshal current image: %v", err)
	}
	second, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseRunning, runObservationInitializerAccepted, state,
	)
	if err != nil {
		t.Fatalf("repeat marshal current image: %v", err)
	}
	if !bytes.Equal(first, second) {
		t.Fatal("current image encoding is not deterministic")
	}
	inner, err := marshalRunObservationRetainedImage(state)
	if err != nil {
		t.Fatalf("marshal canonical helper image: %v", err)
	}
	want := append([]byte(`{"schema_version":"run-observation-poller-v1","phase":1,"init_result":1,"current":`), inner...)
	want = append(want, '}')
	if !bytes.Equal(first, want) {
		t.Fatalf("current image = %s, want complete canonical wrapper %s", first, want)
	}
	if bytes.Count(first, []byte(`"case_id"`)) != 1 || bytes.Contains(first, []byte(`"key"`)) {
		t.Fatalf("current image must encode the key once inside current: %s", first)
	}
	var wrapper map[string]json.RawMessage
	if err := json.Unmarshal(first, &wrapper); err != nil {
		t.Fatalf("decode wrapper: %v", err)
	}
	current := wrapper["current"]
	if len(current) == 0 || current[0] != '{' {
		t.Fatalf("current is not an embedded JSON object: %s", current)
	}
	if _, exists := wrapper["key"]; exists {
		t.Fatalf("current branch redundantly encoded wrapper key: %s", first)
	}
	if len(first) > runObservationRetainedImageMaxBytes {
		t.Fatalf("current image size %d exceeds bound %d", len(first), runObservationRetainedImageMaxBytes)
	}
}

func TestRunObservationPollerImageCombinedExactLimitAndOneByteOver(t *testing.T) {
	key := retainedTestKey()
	prior := retainedTestState(key, "opaque-A")
	prior.Frames[0].Frame.Timestamp = "2026-10-07T12:00:00.1Z"
	probe, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseRunning, runObservationInitializerAccepted, prior,
	)
	if err != nil {
		t.Fatalf("marshal base image: %v", err)
	}
	padding := runObservationRetainedImageMaxBytes - len(probe)
	if padding <= 0 {
		t.Fatalf("base combined image already exceeds bound: %d", len(probe))
	}
	prior.Key.caseID += strings.Repeat("x", padding)
	key = prior.Key
	exact, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseRunning, runObservationInitializerAccepted, prior,
	)
	if err != nil {
		t.Fatalf("marshal exact-limit image: %v", err)
	}
	if len(exact) != runObservationRetainedImageMaxBytes {
		t.Fatalf("exact combined image size %d, want %d", len(exact), runObservationRetainedImageMaxBytes)
	}
	exactInner, err := marshalRunObservationRetainedImage(prior)
	if err != nil {
		t.Fatalf("marshal exact helper image: %v", err)
	}
	if len(exactInner) >= runObservationRetainedImageMaxBytes {
		t.Fatalf("exact combined wrapper inner image %d must fit below its local bound", len(exactInner))
	}

	candidate := cloneRetainedTestState(prior)
	candidate.Frames[0].Frame.Timestamp = "2026-10-07T12:00:00.12Z"
	inner, err := marshalRunObservationRetainedImage(candidate)
	if err != nil {
		t.Fatalf("marshal candidate helper image: %v", err)
	}
	if len(inner) >= runObservationRetainedImageMaxBytes {
		t.Fatalf("candidate helper image %d must fit below its local bound", len(inner))
	}
	if len(inner) != len(exactInner)+1 {
		t.Fatalf("one-digit timestamp extension changed helper size by %d bytes, want exactly 1", len(inner)-len(exactInner))
	}
	candidateOracle := append([]byte(`{"schema_version":"run-observation-poller-v1","phase":1,"init_result":1,"current":`), inner...)
	candidateOracle = append(candidateOracle, '}')
	if len(candidateOracle) != runObservationRetainedImageMaxBytes+1 {
		t.Fatalf("independent canonical candidate wrapper size %d, want exactly %d", len(candidateOracle), runObservationRetainedImageMaxBytes+1)
	}
	// This expected wrapper is built from the canonical helper bytes, without
	// using the poller encoder output to establish the +1 length.
	wantOverLimit := len(exact) + len(inner) - len(exactInner)
	if wantOverLimit != runObservationRetainedImageMaxBytes+1 {
		t.Fatalf("predicted combined candidate size %d, want exactly one byte over", wantOverLimit)
	}
	got, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseRunning, runObservationInitializerAccepted, candidate,
	)
	if err == nil || got != nil {
		t.Fatalf("combined +1 result = (%d bytes, %v), want nil bytes and generic refusal", len(got), err)
	}
	if err.Error() != errRunObservationPollerImageUnavailable.Error() {
		t.Fatalf("combined +1 error = %v, want fixed generic error", err)
	}
}

func TestRunObservationPollerImageMarkerControlsKeepExactPriorWidth(t *testing.T) {
	key := retainedTestKey()
	prior := retainedTestState(key, "opaque-A")
	probe, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseRunning, runObservationInitializerAccepted, prior,
	)
	if err != nil {
		t.Fatalf("marshal base image: %v", err)
	}
	padding := runObservationRetainedImageMaxBytes - len(probe)
	if padding <= 0 {
		t.Fatalf("base combined image already exceeds bound: %d", len(probe))
	}
	prior.Key.caseID += strings.Repeat("x", padding)
	key = prior.Key
	priorBefore := cloneRetainedTestState(prior)
	base, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseRunning, runObservationInitializerAccepted, prior,
	)
	if err != nil {
		t.Fatalf("marshal exact prior image: %v", err)
	}
	if len(base) != runObservationRetainedImageMaxBytes {
		t.Fatalf("prior combined size %d, want exact limit", len(base))
	}

	for availability := retainedCurrent; availability <= retainedStopScheduling; availability++ {
		marker := retainedMarkerCopy(prior, availability)
		phase := runObservationPollerPhaseRunning
		if availability == retainedTerminalRetentionFailure || availability == retainedStopScheduling {
			phase = runObservationPollerPhaseStopping
		}
		encoded, err := marshalRunObservationPollerImage(
			key, phase, runObservationInitializerAccepted, marker,
		)
		if err != nil {
			t.Fatalf("marshal marker availability %d: %v", availability, err)
		}
		if len(encoded) != len(base) {
			t.Fatalf("marker availability %d changed image size %d -> %d", availability, len(base), len(encoded))
		}
		if !reflect.DeepEqual(prior, priorBefore) {
			t.Fatalf("marker availability %d mutated prior state", availability)
		}
		if marker == prior || marker.Frames[0].Frame.ExitCode == prior.Frames[0].Frame.ExitCode {
			t.Fatalf("marker availability %d aliases prior state", availability)
		}
	}
}

func TestRunObservationPollerImageOversizedNilCurrentReturnsGenericRefusal(t *testing.T) {
	key := retainedTestKey()
	key.caseID += strings.Repeat("x", runObservationRetainedImageMaxBytes)
	got, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseInitializing, runObservationInitializerPending, nil,
	)
	if err == nil || got != nil {
		t.Fatalf("oversized nil-current result = (%d bytes, %v), want nil bytes and refusal", len(got), err)
	}
	if err.Error() != errRunObservationPollerImageUnavailable.Error() {
		t.Fatalf("oversized key error = %v, want fixed generic error", err)
	}
	if strings.Contains(err.Error(), key.caseID) {
		t.Fatal("oversized key appeared in error")
	}
}

func TestRunObservationPollerImageDoesNotMutateStateOrAliasBytes(t *testing.T) {
	key := retainedTestKey()
	state := retainedTestState(key, "opaque-A")
	before := cloneRetainedTestState(state)
	first, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseRunning, runObservationInitializerAccepted, state,
	)
	if err != nil {
		t.Fatalf("marshal current image: %v", err)
	}
	expected := append([]byte(nil), first...)
	first[0] = '['
	if !reflect.DeepEqual(state, before) {
		t.Fatal("encoding or mutating returned bytes changed source state")
	}
	second, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseRunning, runObservationInitializerAccepted, state,
	)
	if err != nil {
		t.Fatalf("remarshal current image: %v", err)
	}
	if !bytes.Equal(second, expected) {
		t.Fatal("returned bytes alias or changed the encoder's source state")
	}

	nilCurrent, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseInitializing, runObservationInitializerPending, nil,
	)
	if err != nil {
		t.Fatalf("marshal nil-current image: %v", err)
	}
	nilCurrentCopy := append([]byte(nil), nilCurrent...)
	nilCurrent[0] = '['
	nilCurrentAgain, err := marshalRunObservationPollerImage(
		key, runObservationPollerPhaseInitializing, runObservationInitializerPending, nil,
	)
	if err != nil {
		t.Fatalf("remarshal nil-current image: %v", err)
	}
	if !bytes.Equal(nilCurrentAgain, nilCurrentCopy) {
		t.Fatal("nil-current returned bytes alias another encoding")
	}
}

func TestRunObservationPollerImageRejectsInvalidCodesAndCurrentKeyMismatch(t *testing.T) {
	key := retainedTestKey()
	state := retainedTestState(key, "opaque-A")
	cases := []struct {
		name     string
		phase    runObservationPollerPhase
		init     runObservationInitializerResult
		imageKey runObservationPollerKey
		current  *runObservationRetainedState
	}{
		{
			name: "invalid_phase", phase: runObservationPollerPhase(4),
			init: runObservationInitializerPending, imageKey: key,
		},
		{
			name: "invalid_initializer_result", phase: runObservationPollerPhaseInitializing,
			init: runObservationInitializerResult(6), imageKey: key,
		},
		{
			name: "current_key_mismatch", phase: runObservationPollerPhaseRunning,
			init: runObservationInitializerAccepted, imageKey: key, current: state,
		},
	}
	cases[2].current = cloneRetainedTestState(state)
	cases[2].current.Key.runID = "different-run"
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			got, err := marshalRunObservationPollerImage(tc.imageKey, tc.phase, tc.init, tc.current)
			if err == nil || got != nil {
				t.Fatalf("invalid input result = (%d bytes, %v), want nil bytes and generic error", len(got), err)
			}
			if err.Error() != errRunObservationPollerImageUnavailable.Error() {
				t.Fatalf("invalid input error = %v, want fixed generic error", err)
			}
			if strings.Contains(err.Error(), key.caseID) {
				t.Fatal("invalid input key appeared in error")
			}
		})
	}
}
