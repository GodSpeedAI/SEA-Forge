package projection

import (
	"encoding/json"
	"reflect"
	"strings"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func TestBuildAddsCaseOwnedRunChildrenForEveryStandingPair(t *testing.T) {
	facts := baseFacts()
	facts.Overview.Stages = []string{"stage_build"}
	facts.Horizon.Items = []ports.HorizonItem{{
		ItemID: "item_build", Name: "Build release", Kind: "sandboxed_task",
		Execution: "enabled", Settlement: "unsettled", ParentStage: "stage_build",
	}}
	withoutRuns := Build(facts)
	parentBefore := objectByID(t, withoutRuns, "item_build")

	executions := []string{"pending", "enabled", "active", "completed", "failed", "terminated"}
	settlements := []string{"unsettled", "accepted", "rejected", "escalated"}
	want := make(map[string]ports.RunSummary, len(executions)*len(settlements))
	for _, execution := range executions {
		for _, settlement := range settlements {
			runID := "run_" + execution + "_" + settlement
			summary := ports.RunSummary{
				RunID: runID, CaseID: string(facts.Overview.Ref), PlanItemID: "item_build",
				Execution: execution, Settlement: settlement,
			}
			facts.Runs = append(facts.Runs, summary)
			want[runID] = summary
		}
	}

	snapshot := Build(facts)
	children := 0
	seen := make(map[string]int, len(want))
	for _, object := range snapshot.VisibleObjects {
		if object.Kind != "execution_trace" {
			continue
		}
		children++
		summary, ok := want[object.ID]
		if !ok {
			t.Errorf("unexpected execution_trace child %q", object.ID)
			continue
		}
		seen[object.ID]++
		if seen[object.ID] > 1 {
			t.Errorf("run %q emitted more than one execution_trace child", object.ID)
		}
		if object.ParentID == nil || *object.ParentID != summary.PlanItemID {
			t.Errorf("run %q parent = %v, want actual horizon item %q", object.ID, object.ParentID, summary.PlanItemID)
		}
		if len(object.Actions) != 0 {
			t.Errorf("run %q must have no actions, got %+v", object.ID, object.Actions)
		}
		encoded, err := json.Marshal(object)
		if err != nil {
			t.Fatalf("marshal run child %q: %v", object.ID, err)
		}
		var wire map[string]any
		if err := json.Unmarshal(encoded, &wire); err != nil {
			t.Fatalf("decode run child %q: %v", object.ID, err)
		}
		for _, forbidden := range []string{"frames", "observation", "poller", "trace_frames"} {
			if _, ok := wire[forbidden]; ok {
				t.Errorf("run child %q must not carry live observation state %q", object.ID, forbidden)
			}
		}
		standing := object.Badge
		if object.Explanation != nil {
			standing += " " + *object.Explanation
		}
		if !hasLabelledStanding(standing, "execution", summary.Execution) {
			t.Errorf("run %q does not expose execution standing %q under its label: %q", object.ID, summary.Execution, standing)
		}
		if !hasLabelledStanding(standing, "settlement", summary.Settlement) {
			t.Errorf("run %q does not expose settlement standing %q under its label: %q", object.ID, summary.Settlement, standing)
		}
		if summary.Execution == "completed" && summary.Settlement == "unsettled" && strings.Contains(strings.ToLower(standing), "accepted") {
			t.Errorf("completed/unsettled run %q implies accepted settlement: %q", object.ID, standing)
		}
	}
	if children != len(want) {
		t.Errorf("execution_trace children = %d, want all %d valid runs (hydration limit must not truncate captured facts)", children, len(want))
	}
	for _, execution := range executions {
		for _, settlement := range settlements {
			runID := "run_" + execution + "_" + settlement
			if seen[runID] != 1 {
				t.Errorf("run %q emitted %d execution_trace children, want exactly one", runID, seen[runID])
			}
		}
	}
	if got := objectByID(t, snapshot, "item_build"); !reflect.DeepEqual(got.Actions, parentBefore.Actions) {
		t.Errorf("adding run children changed parent item actions: before=%+v after=%+v", parentBefore.Actions, got.Actions)
	}
	if snapshot.AttentionFocus.PrimaryObjectID != withoutRuns.AttentionFocus.PrimaryObjectID {
		t.Errorf("adding run children changed plan primary focus: before=%q after=%q", withoutRuns.AttentionFocus.PrimaryObjectID, snapshot.AttentionFocus.PrimaryObjectID)
	}
	if !reflect.DeepEqual(snapshot.AttentionFocus.Narration, withoutRuns.AttentionFocus.Narration) {
		t.Errorf("adding run children changed plan focus narration: before=%v after=%v", withoutRuns.AttentionFocus.Narration, snapshot.AttentionFocus.Narration)
	}
	if !reflect.DeepEqual(attentionOrderFor(snapshot, "item_build"), attentionOrderFor(withoutRuns, "item_build")) {
		t.Errorf("adding run children changed parent item rank: before=%v after=%v", attentionOrderFor(withoutRuns, "item_build"), attentionOrderFor(snapshot, "item_build"))
	}
	if !reflect.DeepEqual(snapshot.Summary.ProgressPercent, withoutRuns.Summary.ProgressPercent) {
		t.Errorf("run summaries fabricated or changed plan progress: before=%v after=%v", withoutRuns.Summary.ProgressPercent, snapshot.Summary.ProgressPercent)
	}
}

func TestBuildOmitsInvalidRunChildrenWithoutDroppingValidSiblings(t *testing.T) {
	facts := baseFacts()
	facts.Overview.Stages = []string{"stage_build"}
	facts.Horizon.Items = []ports.HorizonItem{
		{ItemID: "item_build", Name: "Build", Kind: "sandboxed_task", Execution: "enabled", Settlement: "unsettled", ParentStage: "stage_build"},
		{ItemID: "item_other", Name: "Other", Kind: "sandboxed_task", Execution: "pending", Settlement: "unsettled", ParentStage: "stage_build"},
	}
	facts.Runs = []ports.RunSummary{
		{RunID: "run_valid", CaseID: string(facts.Overview.Ref), PlanItemID: "item_build", Execution: "active", Settlement: "unsettled"},
		{RunID: "", CaseID: string(facts.Overview.Ref), PlanItemID: "item_build", Execution: "active", Settlement: "unsettled"},
		{RunID: "run_duplicate", CaseID: string(facts.Overview.Ref), PlanItemID: "item_build", Execution: "active", Settlement: "unsettled"},
		{RunID: "run_duplicate", CaseID: string(facts.Overview.Ref), PlanItemID: "item_other", Execution: "completed", Settlement: "accepted"},
		{RunID: "run_foreign", CaseID: "case_foreign", PlanItemID: "item_build", Execution: "active", Settlement: "unsettled"},
		{RunID: "run_missing_parent", CaseID: string(facts.Overview.Ref), PlanItemID: "item_absent", Execution: "active", Settlement: "unsettled"},
		{RunID: "run_empty_parent", CaseID: string(facts.Overview.Ref), PlanItemID: "", Execution: "active", Settlement: "unsettled"},
		{RunID: "stage_build", CaseID: string(facts.Overview.Ref), PlanItemID: "item_build", Execution: "active", Settlement: "unsettled"},
		{RunID: "item_other", CaseID: string(facts.Overview.Ref), PlanItemID: "item_build", Execution: "active", Settlement: "unsettled"},
		{RunID: "run_unknown_execution", CaseID: string(facts.Overview.Ref), PlanItemID: "item_build", Execution: "suspended", Settlement: "unsettled"},
		{RunID: "run_unknown_settlement", CaseID: string(facts.Overview.Ref), PlanItemID: "item_build", Execution: "active", Settlement: "pending"},
	}

	snapshot := Build(facts)
	valid := objectByID(t, snapshot, "run_valid")
	if valid.Kind != "execution_trace" || valid.ParentID == nil || *valid.ParentID != "item_build" {
		t.Fatalf("valid sibling run was not linked to its actual parent: %+v", valid)
	}
	for _, invalidID := range []string{
		"run_duplicate", "run_foreign", "run_missing_parent", "run_empty_parent",
		"run_unknown_execution", "run_unknown_settlement",
	} {
		if hasRunChildID(snapshot, invalidID) {
			t.Errorf("invalid or colliding run %q must not create a child", invalidID)
		}
	}
	if got := objectByID(t, snapshot, "item_build"); got.Kind != "work_item" || got.ID != "item_build" {
		t.Errorf("a run ID collision must preserve its actual parent item: %+v", got)
	}
	if got := objectByID(t, snapshot, "stage_build"); got.Kind != "stage" {
		t.Errorf("a run ID collision must preserve its stage object: %+v", got)
	}
	if got := objectByID(t, snapshot, "item_other"); got.Kind != "work_item" {
		t.Errorf("a run ID collision must preserve its other horizon item: %+v", got)
	}
	if hasRunChildID(snapshot, "stage_build") || hasRunChildID(snapshot, "item_other") {
		t.Error("colliding run IDs must not replace stage/item objects with execution_trace children")
	}
	children := 0
	for _, object := range snapshot.VisibleObjects {
		if object.Kind == "execution_trace" {
			children++
		}
	}
	if children != 1 {
		t.Errorf("valid execution_trace siblings = %d, want 1; invalid rows must not displace valid siblings", children)
	}
}

func TestBuildRunChildrenDoNotManufactureParentAttention(t *testing.T) {
	cases := []ports.RunSummary{
		{RunID: "run_ready", Execution: "enabled", Settlement: "unsettled"},
		{RunID: "run_escalated", Execution: "completed", Settlement: "escalated"},
	}
	for _, run := range cases {
		t.Run(run.RunID, func(t *testing.T) {
			facts := baseFacts()
			facts.Overview.Stages = nil
			facts.Horizon.Items = []ports.HorizonItem{
				{ItemID: "item_first", Name: "First waiting item", Kind: "sandboxed_task", Execution: "pending", Settlement: "unsettled"},
				{ItemID: "item_second", Name: "Second waiting item", Kind: "sandboxed_task", Execution: "pending", Settlement: "unsettled"},
			}
			baseline := Build(facts)
			run.CaseID = string(facts.Overview.Ref)
			run.PlanItemID = "item_first"
			facts.Runs = []ports.RunSummary{run}
			withRun := Build(facts)

			if withRun.AttentionFocus.PrimaryObjectID != baseline.AttentionFocus.PrimaryObjectID {
				t.Errorf("run standing changed parent primary focus from %q to %q", baseline.AttentionFocus.PrimaryObjectID, withRun.AttentionFocus.PrimaryObjectID)
			}
			if !reflect.DeepEqual(withRun.AttentionFocus.Narration, baseline.AttentionFocus.Narration) {
				t.Errorf("run standing changed parent attention narration: before=%v after=%v", baseline.AttentionFocus.Narration, withRun.AttentionFocus.Narration)
			}
			if !reflect.DeepEqual(attentionOrderFor(withRun, "item_first", "item_second"), attentionOrderFor(baseline, "item_first", "item_second")) {
				t.Errorf("adding run children changed the relative plan-item order: before=%v after=%v", attentionOrderFor(baseline, "item_first", "item_second"), attentionOrderFor(withRun, "item_first", "item_second"))
			}
		})
	}
}

func TestStoreKeepsRunChildStandingAtCapturedCursor(t *testing.T) {
	facts := baseFacts()
	facts.Cursor = "01AAA"
	facts.Horizon.Items = []ports.HorizonItem{{
		ItemID: "item_build", Name: "Build", Kind: "sandboxed_task",
		Execution: "enabled", Settlement: "unsettled",
	}}
	facts.Runs = []ports.RunSummary{{
		RunID: "run_history", CaseID: string(facts.Overview.Ref), PlanItemID: "item_build",
		Execution: "completed", Settlement: "unsettled",
	}}
	store := NewStoreWithRetention(2)
	firstSnapshot := Build(facts)
	if err := store.Append(Revision{Cursor: facts.Cursor, At: facts.Now, CaseID: string(facts.Overview.Ref), Snapshot: firstSnapshot, Facts: &facts}); err != nil {
		t.Fatal(err)
	}
	firstRun := objectByID(t, firstSnapshot, "run_history")

	// A later authority read observes a changed run. The earlier cursor keeps its captured child.
	facts.Cursor = "01AAB"
	facts.Runs[0].Execution = "failed"
	facts.Runs[0].Settlement = "rejected"
	if err := store.Append(Revision{Cursor: facts.Cursor, At: facts.Now, CaseID: string(facts.Overview.Ref), Snapshot: Build(facts), Facts: &facts}); err != nil {
		t.Fatal(err)
	}
	old, err := store.At("01AAA")
	if err != nil {
		t.Fatal(err)
	}
	oldRun := objectByID(t, old.Snapshot, "run_history")
	if oldRun.Badge != firstRun.Badge || !reflect.DeepEqual(oldRun.Explanation, firstRun.Explanation) {
		t.Fatalf("later run standing changed the previously captured snapshot: before=%+v after=%+v", firstRun, oldRun)
	}
	if old.Facts == nil || len(old.Facts.Runs) != 1 || old.Facts.Runs[0].Execution != "completed" || old.Facts.Runs[0].Settlement != "unsettled" {
		t.Fatalf("later run summary changed retained historical facts: %+v", old.Facts)
	}
	old.Facts.Runs[0].Execution = "terminated"
	again, err := store.At("01AAA")
	if err != nil {
		t.Fatal(err)
	}
	if again.Facts.Runs[0].Execution != "completed" {
		t.Fatalf("mutating a returned revision changed retained run facts: %+v", again.Facts.Runs)
	}
}

func hasRunChildID(snapshot contract.CognitiveWorldSnapshot, id string) bool {
	for _, object := range snapshot.VisibleObjects {
		if object.ID == id && object.Kind == "execution_trace" {
			return true
		}
	}
	return false
}

func attentionOrderFor(snapshot contract.CognitiveWorldSnapshot, parentIDs ...string) []string {
	wanted := make(map[string]struct{}, len(parentIDs))
	for _, id := range parentIDs {
		wanted[id] = struct{}{}
	}
	var ordered []string
	for _, id := range snapshot.AttentionFocus.SalienceRank {
		if _, ok := wanted[id]; ok {
			ordered = append(ordered, id)
		}
	}
	return ordered
}

func hasLabelledStanding(text, label, value string) bool {
	lower := strings.ToLower(text)
	label = strings.ToLower(label)
	value = strings.ToLower(value)
	for offset := 0; offset < len(lower); {
		i := strings.Index(lower[offset:], label)
		if i < 0 {
			return false
		}
		i += offset + len(label)
		rest := strings.TrimLeft(lower[i:], " :=\t")
		if strings.HasPrefix(rest, value) {
			return true
		}
		offset = i
	}
	return false
}

func TestRunChildStatusReservesCompletedForAcceptedSettlement(t *testing.T) {
	facts := baseFacts()
	facts.Horizon.Items = []ports.HorizonItem{
		{ItemID: "item_build", Name: "build", Kind: "sandboxed_task", Execution: "completed", Settlement: "unsettled"},
	}
	for settlement, want := range map[string]string{
		"unsettled": "WAITING_ON_OTHERS",
		"accepted":  "COMPLETED",
		"rejected":  "REJECTED",
		"escalated": "ACTION_REQUIRED",
	} {
		facts.Runs = []ports.RunSummary{{RunID: "run_1", CaseID: string(facts.Overview.Ref), PlanItemID: "item_build", Execution: "completed", Settlement: settlement}}
		if got := objectByID(t, Build(facts), "run_1").Status; got != want {
			t.Errorf("completed run, settlement %s: status = %q, want %q", settlement, got, want)
		}
	}
}
