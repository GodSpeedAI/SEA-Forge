//go:build live

// Golden-frame capture for case_record_append (WP-14): drives the REAL server and merges the
// recorded frames (by name) into testdata/frames.json, leaving every other recorded frame as it
// was. Run with:
//
//	GOLDEN_CAPTURE_APPEND=testdata go test -tags live -run TestGoldenCaptureCaseRecordAppend ./internal/adapters/sfwp/
package sfwp

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// canonicalJSON is RFC 8785 for the ASCII, integer-only values the fixtures carry: encoding/json
// sorts object keys and, with HTML escaping off, writes strings the way JCS does for ASCII.
func canonicalJSON(t *testing.T, v any) []byte {
	t.Helper()
	var buf bytes.Buffer
	enc := json.NewEncoder(&buf)
	enc.SetEscapeHTML(false)
	if err := enc.Encode(v); err != nil {
		t.Fatal(err)
	}
	return bytes.TrimRight(buf.Bytes(), "\n")
}

func loadAppendFixture(t *testing.T, name string) map[string]any {
	t.Helper()
	raw, err := os.ReadFile(filepath.Join(repoRoot(t), "contracts", "fde", "fixtures", "case-record-append", "valid", name))
	if err != nil {
		t.Fatal(err)
	}
	dec := json.NewDecoder(bytes.NewReader(raw))
	dec.UseNumber()
	var out map[string]any
	if err := dec.Decode(&out); err != nil {
		t.Fatal(err)
	}
	return out
}

func TestGoldenCaptureCaseRecordAppend(t *testing.T) {
	outDir := os.Getenv("GOLDEN_CAPTURE_APPEND")
	if outDir == "" {
		t.Skip("GOLDEN_CAPTURE_APPEND not set; run with GOLDEN_CAPTURE_APPEND=testdata to record the case.record.append frames")
	}
	cell := newLiveCell(t)
	rc := dialRaw(t, cell.socket)
	actor := map[string]any{"actor_id": "operator_local", "role": "operator"}
	var frames []goldenFrame
	call := func(name string, req map[string]any) string {
		resp := rc.call(t, req)
		frames = append(frames, goldenFrame{Name: name, Request: string(mustJSON(t, req)), Response: resp})
		return resp
	}

	// A real committed case to address (same commit the main capture records).
	preflightResp := rc.call(t, map[string]any{"verb": "case_preflight", "template_ref": sentryChainRef, "params": sentryChainParams()})
	var preflight PreflightView
	if err := json.Unmarshal([]byte(preflightResp), &preflight); err != nil {
		t.Fatalf("preflight response: %v", err)
	}
	commitReq := map[string]any{
		"verb": "case_commit", "template_ref": sentryChainRef, "params": sentryChainParams(),
		"policy": policyRef, "entity": "operator_local", "process": "golden_capture", "timeout": 60,
		"request_id": "req-golden-record-commit", "actor": actor,
	}
	if preflight.Precondition != nil {
		commitReq["preconditions"] = map[string]any{"records": []map[string]any{{
			"ref": preflight.Precondition.Ref, "expected_digest": preflight.Precondition.ExpectedDigest}}}
	}
	var commit CommitView
	if err := json.Unmarshal([]byte(rc.call(t, commitReq)), &commit); err != nil {
		t.Fatalf("commit response: %v", err)
	}
	caseID := commit.CaseID

	// The freeze: the fixture's frozen reference re-addressed to this case, digest recomputed.
	freeze := loadAppendFixture(t, "request-episode-frozen.json")
	payload := freeze["payload"].(map[string]any)
	reference := payload["reference"].(map[string]any)
	body := reference["body"].(map[string]any)
	body["case_id"] = caseID
	sum := sha256.Sum256(canonicalJSON(t, body))
	digest := hex.EncodeToString(sum[:])
	item := body["plan_item_id"].(string)
	episodeRef := fmt.Sprintf("episode:%s:%s:%s@sha256:%s", caseID, item, body["n"], digest)
	payload["episode_ref"], payload["digest"], reference["episode_ref"] = episodeRef, digest, episodeRef

	// RealityTrace records must exist first (freeze order); the cell's index file stands in.
	refs := payload["realitytrace_refs"].(map[string]any)
	ids := []string{refs["plan_id"].(string)}
	for _, f := range []string{"question_ids", "threshold_ids"} {
		for _, id := range refs[f].([]any) {
			ids = append(ids, id.(string))
		}
	}
	var index strings.Builder
	for _, id := range ids {
		index.Write(mustJSON(t, map[string]any{"record_id": id}))
		index.WriteString("\n")
	}
	if err := os.MkdirAll(filepath.Join(cell.root, "realitytrace"), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(cell.root, "realitytrace", "records.jsonl"), []byte(index.String()), 0o644); err != nil {
		t.Fatal(err)
	}

	freezeReq := map[string]any{"verb": "case_record_append", "case_id": caseID, "kind": "episode.frozen",
		"payload": payload, "idempotency_key": "golden-freeze-1", "request_id": "req-golden-freeze-1", "actor": actor}
	call("case.record.append.freeze", freezeReq)
	// Contract-level replay: the same idempotency_key and body under a NEW request_id. (The same
	// request_id would be answered from the transport's correlation record, replayed=false.)
	replayReq := map[string]any{}
	for k, v := range freezeReq {
		replayReq[k] = v
	}
	replayReq["request_id"] = "req-golden-freeze-1-retry"
	call("case.record.append.freeze_replayed", replayReq)

	intruder := map[string]any{}
	for k, v := range freezeReq {
		intruder[k] = v
	}
	intruder["idempotency_key"], intruder["request_id"] = "golden-freeze-2", "req-golden-freeze-2"
	call("case.record.append.refused_free_key", intruder)

	settleFixture := loadAppendFixture(t, "request-episode-settled.json")
	settlePayload := settleFixture["payload"].(map[string]any)
	settlePayload["episode_ref"] = episodeRef
	call("case.record.append.settle", map[string]any{"verb": "case_record_append", "case_id": caseID,
		"kind": "episode.settled", "payload": settlePayload, "idempotency_key": "golden-settle-1",
		"request_id": "req-golden-settle-1", "actor": actor})

	// The real server also emitted the interaction events through its outbox: one per ledger
	// record plus one for the I-2 decision, no duplicate for the replay.
	spool, err := os.ReadFile(filepath.Join(cell.root, "interaction-events", caseID+".jsonl"))
	if err != nil {
		t.Fatalf("no interaction events were delivered: %v", err)
	}
	if n := strings.Count(strings.TrimSpace(string(spool)), "\n") + 1; n != 3 {
		t.Fatalf("expected 3 interaction events (2 records + 1 decision), got %d:\n%s", n, spool)
	}

	goldenPath := filepath.Join(outDir, "frames.json")
	var all []goldenFrame
	if raw, err := os.ReadFile(goldenPath); err == nil {
		if err := json.Unmarshal(raw, &all); err != nil {
			t.Fatal(err)
		}
	}
	for _, f := range frames {
		replaced := false
		for i := range all {
			if all[i].Name == f.Name {
				all[i], replaced = f, true
			}
		}
		if !replaced {
			all = append(all, f)
		}
	}
	raw, err := json.MarshalIndent(all, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(goldenPath, append(raw, '\n'), 0o644); err != nil {
		t.Fatal(err)
	}
	t.Logf("merged %d case.record.append frames into %s", len(frames), goldenPath)
}
