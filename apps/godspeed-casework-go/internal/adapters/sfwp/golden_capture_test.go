//go:build live

// Golden-frame capture: drives the REAL server over raw sockets and records verbatim
// request/response line pairs into testdata, where the non-live golden_test.go pins this client's
// codec to them. Run with:
//
//	GOLDEN_CAPTURE=testdata go test -tags live -run TestGoldenCapture ./internal/adapters/sfwp/
package sfwp

import (
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
	"time"
)

func TestGoldenCapture(t *testing.T) {
	outDir := os.Getenv("GOLDEN_CAPTURE")
	if outDir == "" {
		t.Skip("GOLDEN_CAPTURE not set; run with GOLDEN_CAPTURE=testdata to (re)record golden frames")
	}
	if err := os.MkdirAll(outDir, 0o755); err != nil {
		t.Fatal(err)
	}

	cell := newLiveCell(t)
	_ = context.Background() // all capture calls are synchronous raw-socket round trips

	var frames []goldenFrame
	record := func(name, req, resp string) {
		frames = append(frames, goldenFrame{Name: name, Request: req, Response: resp})
	}
	goldenPath := filepath.Join(outDir, "frames.json")

	rc := dialRaw(t, cell.socket)
	call := func(name string, req map[string]any) string {
		resp := rc.call(t, req)
		record(name, string(mustJSON(t, req)), resp)
		return resp
	}

	// Negotiation and probes.
	call("system.hello", map[string]any{"verb": "system_hello", "protocol_version": "1", "client": "golden-capture"})
	call("readiness.get", map[string]any{"verb": "readiness_get"})
	call("identity.get", map[string]any{"verb": "identity_get"})
	call("request.get_status.unknown", map[string]any{"verb": "request_get_status", "request_id": "req-never-issued"})
	call("case.entry_options", map[string]any{"verb": "case_entry_options"})

	actor := map[string]any{"actor_id": "operator_local", "role": "operator"}

	// Preflight + commit (the recorded happy commit).
	preflightReq := map[string]any{"verb": "case_preflight", "template_ref": sentryChainRef, "params": sentryChainParams()}
	preflightResp := call("case.preflight", preflightReq)
	var preflight PreflightView
	if err := json.Unmarshal([]byte(preflightResp), &preflight); err != nil {
		t.Fatalf("preflight response: %v", err)
	}

	commitReq := map[string]any{
		"verb":         "case_commit",
		"template_ref": sentryChainRef,
		"params":       sentryChainParams(),
		"policy":       policyRef,
		"entity":       "operator_local",
		"process":      "golden_capture",
		"timeout":      60,
		"request_id":   "req-golden-commit-1",
		"actor":        actor,
	}
	if preflight.Precondition != nil {
		commitReq["preconditions"] = map[string]any{
			"records": []map[string]any{{
				"ref":             preflight.Precondition.Ref,
				"expected_digest": preflight.Precondition.ExpectedDigest,
			}},
		}
	}
	commitResp := call("case.commit", commitReq)
	var commit CommitView
	if err := json.Unmarshal([]byte(commitResp), &commit); err != nil {
		t.Fatalf("commit response: %v", err)
	}
	caseID := commit.CaseID
	call("request.get_status.completed", map[string]any{"verb": "request_get_status", "request_id": "req-golden-commit-1"})

	// Navigation views.
	call("case.list", map[string]any{"verb": "case_list"})
	call("case.get_overview", map[string]any{"verb": "case_get_overview", "case_id": caseID})
	call("case.get_horizon", map[string]any{"verb": "case_get_horizon", "case_id": caseID})
	call("approval.list", map[string]any{"verb": "approval_list"})

	// Discretionary add (happy) and the dependency-cycle refusal.
	addReq := map[string]any{
		"verb":       "case_add_item",
		"case_id":    caseID,
		"policy":     policyRef,
		"request_id": "req-golden-add-1",
		"actor":      actor,
		"item": map[string]any{
			"plan_item_id":        "golden_extra",
			"name":                "golden discretionary report",
			"item_kind":           "sandboxed_task",
			"sandbox_class":       "local",
			"depends_on":          []string{"task_prepare"},
			"settlement_criteria": map[string]any{"required_artifacts": []string{"work/golden.md"}},
			"max_instances":       1,
			"operations": []map[string]any{{
				"kind":         "write_file",
				"path":         "work/golden.md",
				"content_hint": "golden discretionary output",
			}},
		},
	}
	call("case.add_item", addReq)
	cycleReq := map[string]any{
		"verb":       "case_add_item",
		"case_id":    caseID,
		"policy":     policyRef,
		"request_id": "req-golden-cycle-1",
		"actor":      actor,
		"item": map[string]any{
			"plan_item_id":        "golden_cycle",
			"name":                "self-dependent item",
			"item_kind":           "sandboxed_task",
			"depends_on":          []string{"golden_cycle"},
			"settlement_criteria": map[string]any{},
		},
	}
	cycleResp := rc.call(t, cycleReq)
	frames = append(frames, goldenFrame{Name: "case.add_item.cycle_refused", Request: string(mustJSON(t, cycleReq)), Response: cycleResp})

	// Execution: item.execute produces an episode. When the kernel executes
	// the episode to an accepted settlement, the artifact.get happy frame is
	// recorded from the run's captured artifact. When it settles rejected with
	// the T05 dispatch-finding signature, no artifact can exist anywhere in
	// the cell, so the happy frame is recorded as a documented tombstone
	// (empty Response + Note) that the golden decoder skips loudly instead of
	// the capture failing or, worse, fabricating a frame.
	execReq := map[string]any{
		"verb":       "item_execute",
		"case_id":    caseID,
		"item_id":    "task_prepare",
		"policy":     policyRef,
		"timeout":    60,
		"request_id": "req-golden-exec-1",
		"actor":      actor,
	}
	execResp := call("item.execute", execReq)
	var exec AdvanceView
	if err := json.Unmarshal([]byte(execResp), &exec); err != nil {
		t.Fatalf("item.execute response: %v", err)
	}
	if len(exec.Episodes) == 0 {
		t.Fatal("item.execute recorded no episodes")
	}
	digest := ""
	if exec.Episodes[0].SettlementStatus == "accepted" {
		digest = artifactDigestFromRun(cell, exec.Episodes[0].RunID)
		if digest == "" {
			t.Fatal("no artifact digest for the executed run")
		}
		call("artifact.get", map[string]any{"verb": "artifact_get", "digest": digest})
	} else {
		bases := cell.settlementBases(caseID)
		if !slicesContains(bases, executionDispatchFinding) {
			t.Fatalf("unexpected rejected episode (basis %v) - not the documented dispatch finding", bases)
		}
		t.Logf("recording artifact.get as a tombstone: %s", executionFindingNote)
		frames = append(frames, goldenFrame{
			Name:     "artifact.get",
			Request:  string(mustJSON(t, map[string]any{"verb": "artifact_get", "digest": ""})),
			Response: "",
			Note:     executionFindingNote,
		})
	}
	call("artifact.get.not_found", map[string]any{"verb": "artifact_get", "digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"})

	// Human task on a signoff-gate case.
	gateReq := map[string]any{
		"verb":         "case_commit",
		"template_ref": signoffGateRef,
		"params":       map[string]any{"change_summary": "golden capture"},
		"policy":       policyRef,
		"entity":       "operator_local",
		"process":      "golden_capture",
		"request_id":   "req-golden-commit-2",
		"actor":        actor,
	}
	gateResp := call("case.commit.signoff", gateReq)
	var gate CommitView
	if err := json.Unmarshal([]byte(gateResp), &gate); err != nil {
		t.Fatalf("signoff commit response: %v", err)
	}
	call("human_task.complete", map[string]any{
		"verb":          "human_task_complete",
		"case_id":       gate.CaseID,
		"item_id":       "signoff_release",
		"justification": "golden capture sign-off",
		"policy":        policyRef,
		"request_id":    "req-golden-human-1",
		"actor":         actor,
	})

	// Lifecycle: terminate, reopen, terminate again, plus the empty-reason refusal.
	thirdReq := map[string]any{
		"verb":         "case_commit",
		"template_ref": sentryChainRef,
		"params":       sentryChainParams(),
		"policy":       policyRef,
		"entity":       "operator_local",
		"process":      "golden_capture",
		"request_id":   "req-golden-commit-3",
		"actor":        actor,
	}
	thirdResp := call("case.commit.third", thirdReq)
	var third CommitView
	if err := json.Unmarshal([]byte(thirdResp), &third); err != nil {
		t.Fatalf("third commit response: %v", err)
	}
	call("case.terminate", map[string]any{
		"verb": "case_terminate", "case_id": third.CaseID, "reason": "golden capture termination",
		"policy": policyRef, "request_id": "req-golden-term-1", "actor": actor,
	})
	call("case.reopen", map[string]any{
		"verb": "case_reopen", "case_id": third.CaseID,
		"policy": policyRef, "request_id": "req-golden-reopen-1", "actor": actor,
	})
	call("case.terminate.empty_reason_refused", map[string]any{
		"verb": "case_terminate", "case_id": third.CaseID, "reason": "  ",
		"policy": policyRef, "request_id": "req-golden-term-2", "actor": actor,
	})
	call("case.terminate.second", map[string]any{
		"verb": "case_terminate", "case_id": third.CaseID, "reason": "closed again",
		"policy": policyRef, "request_id": "req-golden-term-3", "actor": actor,
	})

	// Identity refusals.
	identityReq := func(name string, req map[string]any) {
		resp := rc.call(t, req)
		frames = append(frames, goldenFrame{Name: name, Request: string(mustJSON(t, req)), Response: resp})
	}
	identityReq("case.commit.identity_not_bound", map[string]any{
		"verb": "case_commit", "template_ref": sentryChainRef, "params": sentryChainParams(),
		"policy": policyRef, "entity": "intruder", "request_id": "req-golden-refuse-1",
		"actor": map[string]any{"actor_id": "intruder", "role": "operator"},
	})
	identityReq("case.commit.identity_delegation_refused", map[string]any{
		"verb": "case_commit", "template_ref": sentryChainRef, "params": sentryChainParams(),
		"policy": policyRef, "entity": "operator_a", "request_id": "req-golden-refuse-2",
		"actor":        actor,
		"on_behalf_of": map[string]any{"actor_id": "operator_a", "role": "operator"},
	})
	identityReq("case.commit.durable_locator_required", map[string]any{
		"verb": "case_commit", "template_ref": sentryChainRef, "params": sentryChainParams(),
		"policy": policyRef, "entity": "operator_local", "actor": actor,
	})

	// Events: the subscribe ack and one pushed event line (captured on a dedicated connection).
	subRC := dialRaw(t, cell.socket)
	if _, err := subRC.nc.Write([]byte(append(mustJSON(t, map[string]any{"verb": "events_subscribe"}), '\n'))); err != nil {
		t.Fatal(err)
	}
	// Trigger a mutation, then read the ack and the following event frames.
	moreReq := map[string]any{
		"verb": "case_commit", "template_ref": sentryChainRef, "params": sentryChainParams(),
		"policy": policyRef, "entity": "operator_local", "process": "golden_capture",
		"request_id": "req-golden-commit-4", "actor": actor,
	}
	rc.call(t, moreReq)
	// The server replays the durable burst BEFORE writing the subscribe ack,
	// so lines must be classified: event frames before the ack are the replay,
	// the first non-event line is the ack, and the first event frame after the
	// ack is the live push for the mutation above.
	var ackLine, eventLine string
	_ = subRC.nc.SetReadDeadline(time.Now().Add(10 * time.Second))
	for ackLine == "" || eventLine == "" {
		line, err := subRC.br.ReadString('\n')
		if err != nil {
			t.Fatalf("subscribe lines: %v", err)
		}
		line = trimSpace(line)
		if _, isEvent, _ := DecodeEvent([]byte(line)); isEvent {
			if ackLine != "" && eventLine == "" {
				eventLine = line
			}
			continue
		}
		if ackLine == "" {
			ackLine = line
		}
	}
	frames = append(frames, goldenFrame{Name: "events.subscribe.ack",
		Request: string(mustJSON(t, map[string]any{"verb": "events_subscribe"})), Response: ackLine})
	frames = append(frames, goldenFrame{Name: "events.subscribe.pushed_event",
		Request: string(mustJSON(t, map[string]any{"verb": "events_subscribe"})), Response: eventLine, Event: true})
	// And a replayed subscribe from the head proves from_cursor framing.
	replayRC := dialRaw(t, cell.socket)
	if _, err := replayRC.nc.Write([]byte(append(mustJSON(t, map[string]any{"verb": "events_subscribe"}), '\n'))); err != nil {
		t.Fatal(err)
	}
	_ = replayRC.nc.SetReadDeadline(time.Now().Add(10 * time.Second))
	replayAck, err := replayRC.br.ReadString('\n')
	if err != nil {
		t.Fatalf("replay ack: %v", err)
	}
	frames = append(frames, goldenFrame{Name: "events.subscribe.replay_burst_first_line",
		Request: string(mustJSON(t, map[string]any{"verb": "events_subscribe"})), Response: trimSpace(replayAck), Note: "replayed frames precede the ack on the wire"})

	// events.get_range over the durable ledger.
	call("events.get_range", map[string]any{"verb": "events_get_range", "limit": 5})

	// Simulated old-server unknown verb (serde's exact failure shape; no real server can produce
	// this for a verb it knows, so it is recorded as a simulated frame and marked as such).
	frames = append(frames, goldenFrame{
		Name:     "case.add_item.unknown_verb_simulated",
		Request:  string(mustJSON(t, map[string]any{"verb": "case_add_item", "case_id": caseID, "request_id": "req-golden-old-1", "actor": actor, "item": map[string]any{"plan_item_id": "x", "name": "x", "item_kind": "sandboxed_task", "settlement_criteria": map[string]any{}}})),
		Response: `{"error":"bad request: unknown variant ` + "`case_add_item`" + `, expected one of submit, status"}`,
		Note:     "simulated: the parse failure an older server returns for a verb it predates (no error_class)",
	})

	raw, err := json.MarshalIndent(frames, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(goldenPath, append(raw, '\n'), 0o644); err != nil {
		t.Fatal(err)
	}
	t.Logf("recorded %d golden frames to %s", len(frames), goldenPath)
}

func mustJSON(t *testing.T, v any) []byte {
	t.Helper()
	raw, err := json.Marshal(v)
	if err != nil {
		t.Fatal(err)
	}
	return raw
}
