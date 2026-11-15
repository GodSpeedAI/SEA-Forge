//go:build live

// Golden-frame capture for the Field Case execution gate (WP-21a): drives the REAL server and
// merges the recorded frames (by name) into testdata/frames.json. The case is committed from a
// plain template, then made a Field Case by naming fde.field_case@1 as its template in the cell's
// plan.json (the server resolves the template from the store, never from a request tag). Run with:
//
//	GOLDEN_CAPTURE_GATE=testdata go test -tags live -run TestGoldenCaptureEpisodeGate ./internal/adapters/sfwp/
package sfwp

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestGoldenCaptureEpisodeGate(t *testing.T) {
	outDir := os.Getenv("GOLDEN_CAPTURE_GATE")
	if outDir == "" {
		t.Skip("GOLDEN_CAPTURE_GATE not set; run with GOLDEN_CAPTURE_GATE=testdata to record the execution-gate frames")
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

	preflightResp := rc.call(t, map[string]any{"verb": "case_preflight", "template_ref": sentryChainRef, "params": sentryChainParams()})
	var preflight PreflightView
	if err := json.Unmarshal([]byte(preflightResp), &preflight); err != nil {
		t.Fatalf("preflight response: %v", err)
	}
	commitReq := map[string]any{
		"verb": "case_commit", "template_ref": sentryChainRef, "params": sentryChainParams(),
		"policy": policyRef, "entity": "operator_local", "process": "golden_capture", "timeout": 60,
		"request_id": "req-golden-gate-commit", "actor": actor,
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
	episode := map[string]any{"case_id": caseID, "plan_item_id": "item-03", "n": 1, "kind": "evaluated"}
	delegate := func(extra map[string]any, reqID string, id string) map[string]any {
		req := map[string]any{"verb": "delegate", "case_id": id, "endpoint": "local-test", "instruction": "x",
			"max_turns": 1, "policy": policyRef, "request_id": reqID, "actor": actor}
		for k, v := range extra {
			req[k] = v
		}
		return req
	}

	// An unresolved case_id is refused, never assumed plain (C1).
	call("delegate.case_unresolved", delegate(nil, "req-gate-unresolved", "case_does_not_exist"))

	// Field Case: no episode_ref is FREEZE_MISSING on all three verbs, before any run.
	planPath := filepath.Join(cell.root, "cases", caseID, "plan.json")
	raw, err := os.ReadFile(planPath)
	if err != nil {
		t.Fatal(err)
	}
	var plan map[string]any
	if err := json.Unmarshal(raw, &plan); err != nil {
		t.Fatal(err)
	}
	plan["template_ref"] = "fde.field_case@1"
	if err := os.WriteFile(planPath, mustJSON(t, plan), 0o644); err != nil {
		t.Fatal(err)
	}
	call("item_execute.field_case.no_episode_ref", map[string]any{"verb": "item_execute", "case_id": caseID,
		"item_id": "item-03", "policy": policyRef, "timeout": 60, "request_id": "req-gate-exec-0", "actor": actor})
	call("case_advance.field_case.no_episode_ref", map[string]any{"verb": "case_advance", "case_id": caseID,
		"policy": policyRef, "timeout": 60, "request_id": "req-gate-adv-0", "actor": actor})
	call("delegate.field_case.no_episode_ref", delegate(nil, "req-gate-del-0", caseID))

	// RealityTrace records exist for the episode, nothing else: some but not all parts.
	if err := os.MkdirAll(filepath.Join(cell.root, "realitytrace"), 0o755); err != nil {
		t.Fatal(err)
	}
	var index strings.Builder
	for _, id := range []string{"plan:GATE", "question:GATE", "threshold:GATE"} {
		index.Write(mustJSON(t, map[string]any{"record_id": id, "episode_key": map[string]any{
			"case_id": caseID, "plan_item_id": "item-03", "n": 1}}))
		index.WriteString("\n")
	}
	if err := os.WriteFile(filepath.Join(cell.root, "realitytrace", "records.jsonl"), []byte(index.String()), 0o644); err != nil {
		t.Fatal(err)
	}
	call("item_execute.field_case.freeze_incomplete", map[string]any{"verb": "item_execute", "case_id": caseID,
		"item_id": "item-03", "episode_ref": episode, "policy": policyRef, "timeout": 60, "request_id": "req-gate-exec-1", "actor": actor})
	call("case_advance.field_case.freeze_incomplete", map[string]any{"verb": "case_advance", "case_id": caseID,
		"episode_ref": episode, "policy": policyRef, "timeout": 60, "request_id": "req-gate-adv-1", "actor": actor})
	call("delegate.field_case.freeze_incomplete", delegate(map[string]any{"episode_ref": episode,
		"assignment_id": "assign:" + caseID + ":item-03:1:1"}, "req-gate-del-1", caseID))

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
	out, err := json.MarshalIndent(all, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(goldenPath, append(out, '\n'), 0o644); err != nil {
		t.Fatal(err)
	}
	t.Logf("merged %d execution-gate frames into %s", len(frames), goldenPath)
}
