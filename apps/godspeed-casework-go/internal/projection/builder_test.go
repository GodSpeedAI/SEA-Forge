// The pure builder's unit tests: the fold is driven with exact standing data shaped like the
// real kernel's view verbs produce it (verified live against sea-forge-server in T06's live
// golden test, which pins the same shapes from a real cell).
package projection

import (
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func baseFacts() CaseFacts {
	now := time.Date(2026, 9, 25, 12, 0, 0, 0, time.UTC)
	return CaseFacts{
		Record: ports.CaseRecord{Ref: "case_test", State: "active", Summary: "Execute plan"},
		Overview: ports.CaseOverview{
			Ref:         "case_test",
			State:       "active",
			Summary:     "Execute plan",
			TemplateRef: "e2e-sentry-chain@0.1.0",
			ItemCount:   3,
			Stages:      []string{},
		},
		Horizon: ports.CaseHorizon{Ref: "case_test", State: "active", EventsFolded: 2, LastEventID: "cev_000002"},
		Actor:   ports.ActorClaim{ActorID: "operator_local", Role: "operator"},
		Cursor:  "01M3D6VZ6SWAS3E2CB434JTBGE",
		Now:     now,
	}
}

func objectByID(t *testing.T, snap contract.CognitiveWorldSnapshot, id string) contract.CognitiveObject {
	t.Helper()
	for _, o := range snap.VisibleObjects {
		if o.ID == id {
			return o
		}
	}
	t.Fatalf("object %q missing from the snapshot", id)
	return contract.CognitiveObject{}
}

func hasAction(o contract.CognitiveObject, intent string) bool {
	for _, a := range o.Actions {
		if a.Intent == intent {
			return true
		}
	}
	return false
}

func snapshotHasAction(snap contract.CognitiveWorldSnapshot, intent string) bool {
	for _, a := range snap.AvailableActions {
		if a.Intent == intent {
			return true
		}
	}
	return false
}

func TestBuilderSentryChainOperatorSeesExecuteOnlyOnReadyItem(t *testing.T) {
	// The T03 sentry-chain standing right after commit (probed live): task_prepare is enabled
	// (manual activation, entry criteria met), task_publish and the milestone are pending.
	facts := baseFacts()
	facts.Horizon.Items = []ports.HorizonItem{
		{ItemID: "task_prepare", Name: "prepare dataset", Kind: "sandboxed_task", Execution: "enabled", Settlement: "unsettled"},
		{ItemID: "task_publish", Name: "publish dataset report", Kind: "sandboxed_task", Execution: "pending", Settlement: "unsettled"},
		{ItemID: "ms_chain_accepted", Name: "chain accepted", Kind: "milestone", Execution: "pending", Settlement: "unsettled"},
	}
	snap := Build(facts)

	ready := objectByID(t, snap, "task_prepare")
	if ready.Status != "READY_TO_BEGIN" {
		t.Fatalf("ready item status = %q, want READY_TO_BEGIN", ready.Status)
	}
	if !hasAction(ready, "EXECUTE_ITEM") {
		t.Fatal("operator must see EXECUTE_ITEM on the enabled (ready) item")
	}

	dependent := objectByID(t, snap, "task_publish")
	if dependent.Status != "WAITING" {
		t.Fatalf("dependent item status = %q, want WAITING", dependent.Status)
	}
	if hasAction(dependent, "EXECUTE_ITEM") {
		t.Fatal("the dependent item must not offer EXECUTE while its sentry has not fired")
	}
	if dependent.Explanation == nil {
		t.Fatal("the dependent item must show its sentry reason")
	}
	want := "No work has been recorded for this item yet: its entry sentries have not fired."
	if *dependent.Explanation != want {
		t.Fatalf("sentry reason = %q, want the honest standing-derived %q", *dependent.Explanation, want)
	}
}

func TestBuilderRolesNeverSeeActionsTheyCannotTake(t *testing.T) {
	facts := baseFacts()
	facts.Horizon.Items = []ports.HorizonItem{
		{ItemID: "task_prepare", Name: "prepare", Kind: "sandboxed_task", Execution: "enabled", Settlement: "unsettled"},
	}
	for _, role := range []string{"R-SO", "service", "system"} {
		facts.Actor = ports.ActorClaim{ActorID: "x", Role: role}
		snap := Build(facts)
		for _, o := range snap.VisibleObjects {
			if hasAction(o, "EXECUTE_ITEM") {
				t.Fatalf("role %s must not be offered EXECUTE_ITEM", role)
			}
		}
	}
	facts.Actor = ports.ActorClaim{ActorID: "rso", Role: "R-SO"}
	if snap := Build(facts); len(snap.AvailableActions) != 0 {
		t.Fatalf("R-SO with nothing to approve must get an empty action list, got %d", len(snap.AvailableActions))
	}
}

func TestBuilderDisjointExecutionAndSettlementStanding(t *testing.T) {
	facts := baseFacts()
	facts.Horizon.Items = []ports.HorizonItem{
		// Executed cleanly, NO settlement record: never rendered as settled work.
		{ItemID: "item_exec", Name: "executed", Kind: "sandboxed_task", Execution: "completed", Settlement: "unsettled"},
		// Executed and REJECTED: execution completed, settlement rejected.
		{ItemID: "item_rej", Name: "rejected work", Kind: "sandboxed_task", Execution: "completed", Settlement: "rejected"},
		// Failed execution, unsettled.
		{ItemID: "item_fail", Name: "failed work", Kind: "sandboxed_task", Execution: "failed", Settlement: "unsettled"},
	}
	snap := Build(facts)
	exec := objectByID(t, snap, "item_exec")
	if exec.Status != "COMPLETED" || exec.Badge != "Executed; settlement pending" {
		t.Fatalf("executed+unsettled rendered as %q/%q; the two standings must stay disjoint", exec.Status, exec.Badge)
	}
	rej := objectByID(t, snap, "item_rej")
	if rej.Status != "REJECTED" {
		t.Fatalf("rejected settlement rendered as %q, want REJECTED", rej.Status)
	}
	fail := objectByID(t, snap, "item_fail")
	if fail.Status != "FAILED" {
		t.Fatalf("failed execution rendered as %q, want FAILED", fail.Status)
	}
}

func TestBuilderApprovalOffersFollowTheInboxAndTheRole(t *testing.T) {
	facts := baseFacts()
	facts.Horizon.Items = []ports.HorizonItem{
		{ItemID: "signoff_release", Name: "operator sign-off", Kind: "human_task", Execution: "completed", Settlement: "escalated"},
	}
	facts.Approvals = []ports.ApprovalRecord{
		{ApprovalID: "apr_1", CaseID: "case_test", ItemID: "signoff_release", DecisionID: "dec_1"},
	}

	facts.Actor = ports.ActorClaim{ActorID: "rso_local", Role: "R-SO"}
	snap := Build(facts)
	o := objectByID(t, snap, "signoff_release")
	if !hasAction(o, "APPROVE_HUMAN_TASK") || !hasAction(o, "REJECT_HUMAN_TASK") {
		t.Fatal("an approver role must see approve/reject on an item with an open approval")
	}

	facts.Actor = ports.ActorClaim{ActorID: "operator_local", Role: "operator"}
	snap = Build(facts)
	for _, o := range snap.VisibleObjects {
		if hasAction(o, "APPROVE_HUMAN_TASK") || hasAction(o, "REJECT_HUMAN_TASK") {
			t.Fatal("the operator role must never be offered approval decisions (the T06 tooth)")
		}
	}

	// The kernel inbox is the source: without a record, nobody is offered the decision.
	facts.Approvals = nil
	facts.Actor = ports.ActorClaim{ActorID: "rso_local", Role: "R-SO"}
	snap = Build(facts)
	for _, o := range snap.VisibleObjects {
		if hasAction(o, "APPROVE_HUMAN_TASK") {
			t.Fatal("no approval record on the authority: the decision must not be offered")
		}
	}
}

func TestBuilderSentryReasonNamesOpenDependencies(t *testing.T) {
	facts := baseFacts()
	facts.Horizon.Items = []ports.HorizonItem{
		{ItemID: "a", Name: "first", Kind: "sandboxed_task", Execution: "active", Settlement: "unsettled"},
		{ItemID: "b", Name: "second", Kind: "sandboxed_task", Execution: "pending", Settlement: "unsettled", DependsOn: []string{"a"}},
	}
	snap := Build(facts)
	o := objectByID(t, snap, "b")
	if o.Explanation == nil || !strings.Contains(*o.Explanation, "Waiting on first (active, settlement unsettled)") {
		t.Fatalf("dependent explanation must name the open dependency from standing data, got %v", o.Explanation)
	}
	if o.DependsOn == nil || len(o.DependsOn) != 1 || o.DependsOn[0] != "a" {
		t.Fatalf("depends_on must ride on the object, got %v", o.DependsOn)
	}
}

func TestBuilderLifecycleOffersFollowCaseStateAndRole(t *testing.T) {
	facts := baseFacts()
	facts.Horizon.State = "terminated"
	snap := Build(facts)
	if !snapshotHasAction(snap, "REOPEN_CASE") {
		t.Fatal("a terminated case must offer REOPEN_CASE to a lifecycle role")
	}
	if snapshotHasAction(snap, "TERMINATE_CASE") {
		t.Fatal("a terminated case must not offer TERMINATE_CASE")
	}

	facts.Actor = ports.ActorClaim{ActorID: "rso", Role: "R-SO"}
	snap = Build(facts)
	if snapshotHasAction(snap, "REOPEN_CASE") {
		t.Fatal("R-SO is not a lifecycle role")
	}
}

func TestBuilderSummaryProgressAndAttention(t *testing.T) {
	facts := baseFacts()
	facts.Horizon.Items = []ports.HorizonItem{
		{ItemID: "a", Name: "done work", Kind: "sandboxed_task", Execution: "completed", Settlement: "accepted"},
		{ItemID: "b", Name: "executed work", Kind: "sandboxed_task", Execution: "completed", Settlement: "unsettled"},
		{ItemID: "c", Name: "ready work", Kind: "sandboxed_task", Execution: "enabled", Settlement: "unsettled"},
		{ItemID: "d", Name: "waiting work", Kind: "sandboxed_task", Execution: "pending", Settlement: "unsettled"},
	}
	snap := Build(facts)
	if snap.Summary.ProgressPercent == nil || *snap.Summary.ProgressPercent != 25 {
		t.Fatalf("progress must count only executed AND settled work (1 of 4), got %v", snap.Summary.ProgressPercent)
	}
	if snap.AttentionFocus.PrimaryObjectID != "c" {
		t.Fatalf("attention should sit on the ready item, got %q", snap.AttentionFocus.PrimaryObjectID)
	}
	if len(snap.AttentionFocus.SalienceRank) != 4 {
		t.Fatalf("salience rank must cover every object, got %d", len(snap.AttentionFocus.SalienceRank))
	}
	if snap.Cursor == "" || snap.CaseID != "case_test" || snap.WorldID != "world-case_test" {
		t.Fatalf("snapshot identity fields: cursor=%q case=%q world=%q", snap.Cursor, snap.CaseID, snap.WorldID)
	}
	if snap.Perspective.ActorID != "operator_local" || snap.Perspective.Role != "operator" {
		t.Fatalf("perspective: %+v", snap.Perspective)
	}
}

func TestEmptyWorldIsHonest(t *testing.T) {
	now := time.Date(2026, 9, 25, 12, 0, 0, 0, time.UTC)
	snap := EmptyWorld(ports.ActorClaim{ActorID: "gateway", Role: "service"}, "01HEAD", now)
	if snap.CaseID != "" || len(snap.VisibleObjects) != 0 || snap.Cursor != "01HEAD" {
		t.Fatalf("empty world shape: %+v", snap)
	}
	if snap.Summary.Headline != "No cases yet" {
		t.Fatalf("empty world headline: %q", snap.Summary.Headline)
	}
}

func TestTemplatesMapKernelTypesOntoTheContract(t *testing.T) {
	opts := []ports.TemplateOption{
		{
			TemplateRef: "e2e-sentry-chain@0.1.0",
			Description: "chain",
			Parameters: []ports.TemplateParameter{
				{Name: "dataset_name", Type: "string", Required: true},
				{Name: "max_rows", Type: "int", HasDefault: true, Default: "10"},
				{Name: "out_dir", Type: "path"},
			},
		},
	}
	templates, err := Templates(opts)
	if err != nil {
		t.Fatal(err)
	}
	if len(templates) != 1 || templates[0].TemplateRef != "e2e-sentry-chain@0.1.0" || templates[0].Title != "e2e-sentry-chain" {
		t.Fatalf("template mapping: %+v", templates)
	}
	types := map[string]string{}
	for _, p := range templates[0].Parameters {
		types[p.Name] = p.Type
	}
	if types["dataset_name"] != "string" || types["max_rows"] != "number" || types["out_dir"] != "string" {
		t.Fatalf("parameter type translation: %v", types)
	}
	if _, err := Templates([]ports.TemplateOption{{TemplateRef: "x", Parameters: []ports.TemplateParameter{{Name: "p", Type: "blob"}}}}); err == nil {
		t.Fatal("an unrepresentable kernel parameter type must be refused, not guessed")
	}
}

func TestPreflightResultCarriesDigestExactlyWhenPassed(t *testing.T) {
	passed := PreflightResult(ports.PreflightReport{
		OK:          true,
		TemplateRef: "e2e-sentry-chain@0.1.0",
		Precondition: ports.PreconditionDigest{
			Present:        true,
			TemplateRef:    "template:e2e-sentry-chain@0.1.0",
			ExpectedDigest: "sha256:385e",
		},
	}, map[string]any{"dataset_name": "orders"})
	if !passed.Passed || passed.Digest == nil || *passed.Digest != "sha256:385e" || len(passed.Reasons) != 0 {
		t.Fatalf("passing preflight mapping: %+v", passed)
	}
	failed := PreflightResult(ports.PreflightReport{
		OK:          false,
		TemplateRef: "e2e-sentry-chain@0.1.0",
		Errors:      []string{"Parameter dataset_name is required but was not provided."},
	}, map[string]any{})
	if failed.Passed || failed.Digest != nil || len(failed.Reasons) != 1 {
		t.Fatalf("failing preflight mapping: %+v", failed)
	}
}

func TestBuilderOffersDiscretionaryWorkOnlyOnLiveWorkToProposerRoles(t *testing.T) {
	facts := baseFacts()
	facts.Horizon.Items = []ports.HorizonItem{
		{ItemID: "task_prepare", Name: "prepare", Kind: "sandboxed_task", Execution: "enabled", Settlement: "unsettled"},
		{ItemID: "task_publish", Name: "publish", Kind: "sandboxed_task", Execution: "pending", Settlement: "unsettled"},
		{ItemID: "item_done", Name: "done", Kind: "sandboxed_task", Execution: "completed", Settlement: "accepted"},
		{ItemID: "item_failed", Name: "failed", Kind: "sandboxed_task", Execution: "failed", Settlement: "unsettled"},
		{ItemID: "ms_chain", Name: "chain", Kind: "milestone", Execution: "pending", Settlement: "unsettled"},
	}
	snap := Build(facts)
	for id, want := range map[string]bool{"task_prepare": true, "task_publish": true, "item_done": false, "item_failed": false, "ms_chain": false} {
		if got := hasAction(objectByID(t, snap, id), "ADD_DISCRETIONARY_WORK"); got != want {
			t.Fatalf("%s offers ADD_DISCRETIONARY_WORK = %v, want %v", id, got, want)
		}
	}
	if !snapshotHasAction(snap, "ADD_DISCRETIONARY_WORK") {
		t.Fatal("the offer must also ride the snapshot's available_actions")
	}
	// The offer is exactly what the intent guard allows: a non-proposer role never sees it.
	for _, role := range []string{"R-SO", "service", "system"} {
		facts.Actor = ports.ActorClaim{ActorID: "x", Role: role}
		for _, o := range Build(facts).VisibleObjects {
			if hasAction(o, "ADD_DISCRETIONARY_WORK") {
				t.Fatalf("role %s must not be offered ADD_DISCRETIONARY_WORK", role)
			}
		}
	}
}
