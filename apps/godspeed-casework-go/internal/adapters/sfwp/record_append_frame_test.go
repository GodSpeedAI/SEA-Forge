package sfwp

import (
	"encoding/json"
	"testing"
)

func TestNewCaseRecordAppendRequiresBothKeysAndMarksTheVerbDurable(t *testing.T) {
	var g Governance
	if _, err := NewCaseRecordAppend("case_1", "episode.frozen", map[string]any{}, "k-1", "", g); err == nil {
		t.Fatal("a missing request_id must be refused client-side")
	}
	if _, err := NewCaseRecordAppend("case_1", "episode.frozen", map[string]any{}, "", "req-1", g); err == nil {
		t.Fatal("a missing idempotency_key must be refused client-side")
	}
	r, err := NewCaseRecordAppend("case_1", "episode.frozen", map[string]any{"a": 1}, "k-1", "req-1", g)
	if err != nil {
		t.Fatal(err)
	}
	if !r.IsMutation() || r.IsTransportRetrySafe() {
		t.Fatalf("case_record_append must be a correlation-tracked mutation: mutation=%v", r.IsMutation())
	}
}

// TestGoldenCaseRecordAppend pins the builder to the frames recorded from the real server by
// golden_capture_record_append_test.go (case.record.append.*).
func TestGoldenCaseRecordAppend(t *testing.T) {
	frames := loadGolden(t)
	for _, name := range []string{"case.record.append.freeze", "case.record.append.freeze_replayed", "case.record.append.settle", "case.record.append.refused_free_key"} {
		golden := byName(t, frames, name)
		var req struct {
			CaseID         string         `json:"case_id"`
			Kind           string         `json:"kind"`
			Payload        map[string]any `json:"payload"`
			IdempotencyKey string         `json:"idempotency_key"`
			RequestID      string         `json:"request_id"`
			Actor          struct {
				ActorID string `json:"actor_id"`
				Role    string `json:"role"`
			} `json:"actor"`
		}
		if err := json.Unmarshal([]byte(golden.Request), &req); err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		built, err := NewCaseRecordAppend(req.CaseID, req.Kind, req.Payload, req.IdempotencyKey, req.RequestID,
			Governance{Actor: Actor{ActorID: req.Actor.ActorID, Role: req.Actor.Role}})
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		got, err := EncodeRequest(built)
		if err != nil {
			t.Fatal(err)
		}
		if !jsonEqual(t, string(got), golden.Request) {
			t.Fatalf("%s: encoded request drifted from the recorded frame:\n got: %s\nwant: %s", name, got, golden.Request)
		}
	}
	var freeze, replay, settle, refused map[string]any
	for name, into := range map[string]*map[string]any{
		"case.record.append.freeze":           &freeze,
		"case.record.append.freeze_replayed":  &replay,
		"case.record.append.settle":           &settle,
		"case.record.append.refused_free_key": &refused,
	} {
		if err := json.Unmarshal([]byte(byName(t, frames, name).Response), into); err != nil {
			t.Fatalf("%s response: %v", name, err)
		}
	}
	if freeze["outcome"] != "recorded" || freeze["kind"] != "episode.frozen" || freeze["replayed"] != false {
		t.Fatalf("freeze response: %v", freeze)
	}
	if replay["outcome"] != "recorded" || replay["replayed"] != true || replay["record_id"] != freeze["record_id"] {
		t.Fatalf("replay must return the original record: %v vs %v", replay, freeze)
	}
	if settle["outcome"] != "recorded" || settle["kind"] != "episode.settled" {
		t.Fatalf("settle response: %v", settle)
	}
	if refused["outcome"] != "refused" || refused["code"] != "FREEZE_KEY_EXISTS" {
		t.Fatalf("refusal response: %v", refused)
	}
}
