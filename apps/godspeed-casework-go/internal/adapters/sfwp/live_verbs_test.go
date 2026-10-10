//go:build live

// The live verb ladder: every verb the UI needs round-trips against the real kernel, plus the
// cheap refusal paths (identity, delegation, input, dependency cycle).
package sfwp

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func TestLiveVerbRoundTrip(t *testing.T) {
	cell := newLiveCell(t)
	cl := cell.client()
	ctx, cancel := context.WithTimeout(context.Background(), 120*time.Second)
	defer cancel()

	// --- Negotiation and inspect probes -----------------------------------------
	hello, err := cl.Hello(ctx, "godspeed-casework-go-live-test")
	if err != nil {
		t.Fatalf("system.hello: %v", err)
	}
	if hello.ServerProtocolVersion != "1" {
		t.Fatalf("protocol major: got %q", hello.ServerProtocolVersion)
	}
	for _, want := range []string{"case.commit", "case.add_item", "item.execute", "human_task.complete", "artifact.get"} {
		found := false
		for _, m := range hello.ImplementedMethods {
			if m == want {
				found = true
				break
			}
		}
		if !found {
			t.Fatalf("server does not advertise %s (methods: %v)", want, hello.ImplementedMethods)
		}
	}

	readiness, err := NewAuthority(cl).Readiness(ctx)
	if err != nil {
		t.Fatalf("readiness.get: %v", err)
	}
	if readiness.Overall == "" {
		t.Fatal("readiness.get returned no overall verdict")
	}
	t.Logf("readiness overall: %s", readiness.Overall)

	identity, err := NewAuthority(cl).ResolveIdentity(ctx, governanceOf(operatorGov()))
	if err != nil {
		t.Fatalf("identity.get: %v", err)
	}
	if len(identity.Available) != 1 || identity.Available[0].ActorID != "operator_local" {
		t.Fatalf("identity.get available: %+v", identity.Available)
	}

	// --- Authoring: entry options, preflight, commit ----------------------------
	options, err := NewAuthority(cl).EntryOptions(ctx)
	if err != nil {
		t.Fatalf("case.entry_options: %v", err)
	}
	refs := map[string]bool{}
	for _, opt := range options {
		refs[opt.TemplateRef] = true
	}
	if !refs[sentryChainRef] || !refs[signoffGateRef] {
		t.Fatalf("entry options missing the E2E templates: %+v", options)
	}

	authority := NewAuthority(cl)
	draft := caseDraftOf(sentryChainRef, sentryChainParams())
	report, err := authority.PreflightCase(ctx, draft)
	if err != nil {
		t.Fatalf("case.preflight: %v", err)
	}
	if !report.OK || len(report.Items) != 3 || !report.Precondition.Present {
		t.Fatalf("preflight: ok=%v items=%d pin=%+v errors=%v", report.OK, len(report.Items), report.Precondition, report.Errors)
	}

	commitReqID := cl.NewRequestID("case_commit")
	receipt, err := authority.CommitCase(ctx, draft, report.Precondition, governedOpts(commitReqID))
	if err != nil {
		t.Fatalf("case.commit: %v", err)
	}
	caseID := receipt.CaseID
	t.Logf("committed case %s (state %s)", caseID, receipt.State)

	// request.get_status returns the recorded terminal outcome.
	outcome, err := authority.RequestStatus(ctx, commitReqID)
	if err != nil {
		t.Fatalf("request.get_status: %v", err)
	}
	if outcome.Status != "completed" || outcome.Result == nil || outcome.Result.CaseID != caseID {
		t.Fatalf("request.get_status: %+v", outcome)
	}

	// --- Navigation: list, overview, horizon ------------------------------------
	cases, err := authority.ListCases(ctx)
	if err != nil {
		t.Fatalf("case.list: %v", err)
	}
	found := false
	for _, c := range cases {
		if string(c.Ref) == caseID {
			found = true
		}
	}
	if !found {
		t.Fatalf("case.list does not contain %s: %+v", caseID, cases)
	}

	overview, err := authority.CaseOverview(ctx, refOf(caseID))
	if err != nil {
		t.Fatalf("case.get_overview: %v", err)
	}
	if overview.TemplateRef != sentryChainRef || overview.ItemCount != 3 {
		t.Fatalf("overview: %+v", overview)
	}

	horizon, err := authority.CaseHorizon(ctx, refOf(caseID))
	if err != nil {
		t.Fatalf("case.get_horizon: %v", err)
	}
	taskPrepare := horizonItem(t, horizon, "task_prepare")
	if taskPrepare.Execution != "enabled" {
		t.Fatalf("task_prepare should be enabled after commit, got %+v", taskPrepare)
	}

	// --- Discretionary work: add_item (happy + cycle refusal) --------------------
	addReqID := cl.NewRequestID("case_add_item")
	itemID, err := authority.AddCaseItem(ctx, refOf(caseID), discretionaryItem("extra_report", "task_prepare"), governedOpts(addReqID))
	if err != nil {
		t.Fatalf("case.add_item: %v", err)
	}
	if itemID != "extra_report" {
		t.Fatalf("add_item returned %q", itemID)
	}

	cycleReqID := cl.NewRequestID("case_add_item")
	cycleItem := discretionaryItem("cycle_item", "")
	cycleItem.DependsOn = []string{"cycle_item"} // self-dependency: a cycle by construction
	_, err = authority.AddCaseItem(ctx, refOf(caseID), cycleItem, governedOpts(cycleReqID))
	if err == nil {
		t.Fatal("a self-dependent item must be refused")
	}
	var refusal *Refusal
	if !errors.As(err, &refusal) {
		t.Fatalf("cycle refusal must carry its class, got %v", err)
	}
	t.Logf("cycle refusal class: %s (%s)", refusal.Class, refusal.Message)
	if !strings.Contains(refusal.Class, "cycle") && !strings.Contains(refusal.Message, "cycle") {
		t.Fatalf("expected a cycle refusal, got class=%q msg=%q", refusal.Class, refusal.Message)
	}

	// --- Execution: the execution verbs round-trip and report the authority's
	// own verdict. The accepted-episode proof (artifacts through artifact.get,
	// sentry unlocking, advance to completion) lives in
	// TestLiveExecutionProducesAcceptedArtifacts, which documents the T05
	// dispatch finding while the kernel cannot execute write_file items.
	execReqID := cl.NewRequestID("item_execute")
	execReport, err := authority.ExecuteItem(ctx, refOf(caseID), "task_prepare", governedOpts(execReqID))
	if err != nil {
		t.Fatalf("item.execute: %v", err)
	}
	if len(execReport.Episodes) != 1 || execReport.Episodes[0].ItemID != "task_prepare" {
		t.Fatalf("item.execute report: %+v", execReport)
	}
	assertEpisodeOutcome(t, cell, caseID, execReport.Episodes[0])

	// case.advance round-trips on its own case (the first case's required item
	// settles here, terminating it either way; a terminal case would make the
	// advance a refusal rather than a run of the engine).
	advDraft := caseDraftOf(sentryChainRef, sentryChainParams())
	advCommitID := cl.NewRequestID("case_commit")
	advCase, err := authority.CommitCase(ctx, advDraft, ports.PreconditionDigest{}, governedOpts(advCommitID))
	if err != nil {
		t.Fatalf("advance-case commit: %v", err)
	}
	advReqID := cl.NewRequestID("case_advance")
	advReport, err := authority.AdvanceCase(ctx, refOf(advCase.CaseID), governedOpts(advReqID))
	if err != nil {
		t.Fatalf("case.advance: %v", err)
	}
	if advReport.CaseID != advCase.CaseID || advReport.State == "" {
		t.Fatalf("case.advance report: %+v", advReport)
	}
	for _, ep := range advReport.Episodes {
		assertEpisodeOutcome(t, cell, advCase.CaseID, ep)
	}

	// --- Human task: signoff gate parks, completion resolves ----------------------
	gateDraft := caseDraftOf(signoffGateRef, map[string]string{"change_summary": "t05 live wiring"})
	gateReqID := cl.NewRequestID("case_commit")
	gate, err := authority.CommitCase(ctx, gateDraft, ports.PreconditionDigest{}, governedOpts(gateReqID))
	if err != nil {
		t.Fatalf("signoff commit: %v", err)
	}
	gateHorizon, err := authority.CaseHorizon(ctx, refOf(gate.CaseID))
	if err != nil {
		t.Fatalf("gate horizon: %v", err)
	}
	signoff := horizonItem(t, gateHorizon, "signoff_release")
	if signoff.Execution != "active" {
		t.Fatalf("signoff_release should park active, got %+v", signoff)
	}
	htReqID := cl.NewRequestID("human_task_complete")
	if err := authority.CompleteHumanTask(ctx, refOf(gate.CaseID), "signoff_release",
		"live wiring test sign-off", governedOpts(htReqID)); err != nil {
		t.Fatalf("human_task.complete: %v", err)
	}

	// --- Lifecycle: terminate then reopen -----------------------------------------
	thirdDraft := caseDraftOf(sentryChainRef, sentryChainParams())
	thirdReqID := cl.NewRequestID("case_commit")
	third, err := authority.CommitCase(ctx, thirdDraft, ports.PreconditionDigest{}, governedOpts(thirdReqID))
	if err != nil {
		t.Fatalf("third commit: %v", err)
	}
	termReqID := cl.NewRequestID("case_terminate")
	if err := authority.TerminateCase(ctx, refOf(third.CaseID), "t05 terminate round-trip", governedOpts(termReqID)); err != nil {
		t.Fatalf("case.terminate: %v", err)
	}
	reopenReqID := cl.NewRequestID("case_reopen")
	if err := authority.ReopenCase(ctx, refOf(third.CaseID), "t05 reopen round-trip", governedOpts(reopenReqID)); err != nil {
		t.Fatalf("case.reopen: %v", err)
	}
	term2ReqID := cl.NewRequestID("case_terminate")
	if err := authority.TerminateCase(ctx, refOf(third.CaseID), "closed again after reopen", governedOpts(term2ReqID)); err != nil {
		t.Fatalf("second case.terminate: %v", err)
	}

	// --- Approvals: the inbox is a governed view even when empty -------------------
	approvals, err := authority.PendingApprovals(ctx, "")
	if err != nil {
		t.Fatalf("approval.list: %v (an empty inbox must still be readable)", err)
	}
	if len(approvals) != 0 {
		t.Logf("approval.list returned %d pending approvals (none expected in this ladder)", len(approvals))
	}

	// --- Refusals -------------------------------------------------------------------

	// identity_not_bound: an actor this uid does not hold.
	boundOpts := governedOpts(cl.NewRequestID("case_commit"))
	boundOpts.Governance = govWithActor("intruder", "operator")
	_, err = authority.CommitCase(ctx, draft, ports.PreconditionDigest{}, boundOpts)
	assertRefusal(t, err, "identity_not_bound", apperr.KindAuthorityDenied)

	// identity_delegation_refused: on_behalf_of on a cell with no gateway section.
	delegOpts := governedOpts(cl.NewRequestID("case_commit"))
	delegOpts.Governance = govDelegated("operator_a", "operator")
	_, err = authority.CommitCase(ctx, draft, ports.PreconditionDigest{}, delegOpts)
	assertRefusal(t, err, "identity_delegation_refused", apperr.KindAuthorityDenied)

	// input refusal: a termination with no recorded reason is not a governed judgment.
	emptyReqID := cl.NewRequestID("case_terminate")
	err = authority.TerminateCase(ctx, refOf(third.CaseID), "  ", governedOpts(emptyReqID))
	if err == nil {
		t.Fatal("an empty terminate reason must be refused")
	}
	if got := apperr.KindOf(err); got != apperr.KindInvalid {
		t.Fatalf("empty-reason refusal should be invalid, got %s (%v)", got, err)
	}
}

func assertRefusal(t *testing.T, err error, class string, kind apperr.Kind) {
	t.Helper()
	if err == nil {
		t.Fatalf("expected a %s refusal", class)
	}
	if got := apperr.KindOf(err); got != kind {
		t.Fatalf("%s refusal should map to %s, got %s (%v)", class, kind, got, err)
	}
	var refusal *Refusal
	if !errors.As(err, &refusal) || refusal.Class != class {
		t.Fatalf("refusal class %s must survive translation, got %+v", class, refusal)
	}
	if !refusal.NoSideEffect {
		t.Fatalf("a %s refusal must state no_side_effect", class)
	}
}

// assertEpisodeOutcome requires every episode to carry the authority's own
// terminal verdict. A rejection must carry the documented dispatch-finding
// basis in the case's durable ledger - any other rejection is a different,
// unexplained failure. Accepted episodes need no extra assertion here: the
// tests that depend on accepted work assert it themselves.
func assertEpisodeOutcome(t *testing.T, cell *liveCell, caseID string, ep ports.EpisodeReport) {
	t.Helper()
	switch ep.Settlement {
	case "accepted":
	case "rejected":
		bases := cell.settlementBases(caseID)
		if !slicesContains(bases, executionDispatchFinding) {
			t.Fatalf("episode %s settled rejected without the documented dispatch finding (basis %v)", ep.ItemID, bases)
		}
		t.Logf("episode %s settled rejected by the kernel: %s", ep.ItemID, executionFindingNote)
	default:
		t.Fatalf("episode %s carries no terminal settlement verdict: %+v", ep.ItemID, ep)
	}
}

func horizonItem(t *testing.T, h ports.CaseHorizon, id string) ports.HorizonItem {
	t.Helper()
	for _, it := range h.Items {
		if it.ItemID == id {
			return it
		}
	}
	t.Fatalf("no horizon item %s in %+v", id, h)
	return ports.HorizonItem{}
}

// refOf/caseDraftOf/governedOpts/discretionaryItem keep the ladder above reading as decisions
// rather than struct literals.
func refOf(id string) ports.CaseRef { return ports.CaseRef(id) }

func caseDraftOf(ref string, params map[string]string) ports.CaseDraft {
	return ports.CaseDraft{TemplateRef: ref, Params: params}
}

func governedOpts(id string) ports.GovernedOptions {
	return ports.GovernedOptions{
		Governance: ports.Governance{Actor: ports.ActorClaim{ActorID: "operator_local", Role: "operator"}},
		Policy:     policyRef,
		RequestID:  id,
		TimeoutSec: 60,
	}
}

func governanceOf(g Governance) ports.Governance {
	return ports.Governance{Actor: ports.ActorClaim{ActorID: g.Actor.ActorID, Role: g.Actor.Role}}
}

func govWithActor(actorID, role string) ports.Governance {
	return ports.Governance{Actor: ports.ActorClaim{ActorID: actorID, Role: role}}
}

func govDelegated(actorID, role string) ports.Governance {
	return ports.Governance{
		Actor:      ports.ActorClaim{ActorID: "operator_local", Role: "operator"},
		OnBehalfOf: &ports.ActorClaim{ActorID: actorID, Role: role},
	}
}

func discretionaryItem(id, after string) ports.CaseItemProposal {
	item := ports.CaseItemProposal{
		ItemID:       id,
		Name:         "discretionary " + id,
		Kind:         "sandboxed_task",
		SandboxClass: "local",
		Operations: []ports.ItemOperation{{
			Kind:        "write_file",
			Path:        "work/" + id + ".md",
			ContentHint: "discretionary report " + id,
		}},
		RequiredArtifacts: []string{"work/" + id + ".md"},
		MaxInstances:      1,
	}
	if after != "" {
		item.DependsOn = []string{after}
	}
	return item
}
