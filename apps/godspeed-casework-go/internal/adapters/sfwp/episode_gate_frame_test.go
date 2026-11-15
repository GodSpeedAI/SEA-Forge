package sfwp

import (
	"encoding/json"
	"strings"
	"testing"
)

func frameOf(t *testing.T, r *Request, err error) map[string]any {
	t.Helper()
	if err != nil {
		t.Fatal(err)
	}
	raw, err := json.Marshal(r.body)
	if err != nil {
		t.Fatal(err)
	}
	var out map[string]any
	if err := json.Unmarshal(raw, &out); err != nil {
		t.Fatal(err)
	}
	return out
}

func TestEpisodeGateFrames(t *testing.T) {
	ep := EpisodeRef{CaseID: "case-0001", PlanItemID: "item-03", N: 1, Kind: "evaluated"}
	wantEp := map[string]any{"case_id": "case-0001", "plan_item_id": "item-03", "n": float64(1), "kind": "evaluated"}

	r, err := NewCaseAdvanceEpisode("case-0001", &ep, nil, "p", 60, "req-1", Governance{})
	f := frameOf(t, r, err)
	if got := f["episode_ref"].(map[string]any); got["plan_item_id"] != wantEp["plan_item_id"] || got["n"] != wantEp["n"] {
		t.Fatalf("case_advance episode_ref: %v", got)
	}
	if _, ok := f["transition_items"]; ok {
		t.Fatal("an episode advance must not carry transition_items")
	}

	r, err = NewCaseAdvanceEpisode("case-0001", nil, []TransitionItem{{ItemID: "item-05", Transition: "skip"}}, "p", 60, "req-2", Governance{})
	f = frameOf(t, r, err)
	items := f["transition_items"].([]any)
	if len(items) != 1 || items[0].(map[string]any)["transition_only"] != true {
		t.Fatalf("transition_items must carry transition_only: true: %v", items)
	}
	if _, ok := f["episode_ref"]; ok {
		t.Fatal("a transition-only advance must not carry an episode_ref")
	}

	r, err = NewItemExecuteEpisode("case-0001", "item-03", ep, "p", 60, "req-3", Governance{})
	f = frameOf(t, r, err)
	if f["item_id"] != "item-03" || f["episode_ref"] == nil {
		t.Fatalf("item_execute frame: %v", f)
	}

	r, err = NewDelegate("case-0001", "http://127.0.0.1:8099/agent", "do it", 8, &ep, "assign:case-0001:item-03:1:1", "p", "req-4", Governance{})
	f = frameOf(t, r, err)
	if r.Verb() != "delegate" || f["case_id"] != "case-0001" || f["assignment_id"] != "assign:case-0001:item-03:1:1" || f["episode_ref"] == nil {
		t.Fatalf("delegate frame: %v", f)
	}
	// A plain case: case_id only, no episode fields (byte-identical to the legacy frame plus case_id).
	r, err = NewDelegate("case-0001", "http://127.0.0.1:8099/agent", "do it", 8, nil, "", "p", "req-5", Governance{})
	f = frameOf(t, r, err)
	if _, ok := f["episode_ref"]; ok {
		t.Fatal("a plain delegate carries no episode_ref")
	}
	if _, err := NewDelegate("", "e", "i", 1, nil, "", "p", "req-6", Governance{}); err == nil || !strings.Contains(err.Error(), "case_id") {
		t.Fatalf("a delegate without case_id must be refused client-side: %v", err)
	}
}
