package projection

import (
	"context"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

type scopedLiveAuthority struct {
	ports.CaseAuthorityPort
	record        ports.CaseRecord
	records       []ports.CaseRecord
	overview      ports.CaseOverview
	horizon       ports.CaseHorizon
	runResult     ports.RunListResult
	scopedCalls   int
	scopedCases   []ports.CaseRef
	legacyCalls   int
	caseListCalls int
	overviewCalls int
	horizonCalls  int
	approvalCalls int
}

func (a *scopedLiveAuthority) ListCases(context.Context) ([]ports.CaseRecord, error) {
	a.caseListCalls++
	if a.records != nil {
		return a.records, nil
	}
	return []ports.CaseRecord{a.record}, nil
}

func (a *scopedLiveAuthority) CaseOverview(context.Context, ports.CaseRef) (ports.CaseOverview, error) {
	a.overviewCalls++
	return a.overview, nil
}

func (a *scopedLiveAuthority) CaseHorizon(context.Context, ports.CaseRef) (ports.CaseHorizon, error) {
	a.horizonCalls++
	return a.horizon, nil
}

func (a *scopedLiveAuthority) PendingApprovals(context.Context, ports.CaseRef) ([]ports.ApprovalRecord, error) {
	a.approvalCalls++
	return nil, nil
}

func (a *scopedLiveAuthority) RunsList(context.Context) ([]ports.RunSummary, error) {
	a.legacyCalls++
	return nil, apperr.New(apperr.KindInternal, "", "test", "legacy unscoped runs list must not be called")
}

func (a *scopedLiveAuthority) RunsListForCase(_ context.Context, caseRef ports.CaseRef) (ports.RunListResult, error) {
	a.scopedCalls++
	a.scopedCases = append(a.scopedCases, caseRef)
	return a.runResult, nil
}

func validScopedAuthority() *scopedLiveAuthority {
	return &scopedLiveAuthority{
		record:   ports.CaseRecord{Ref: "case_1", Summary: "case"},
		overview: ports.CaseOverview{Ref: "case_1"},
		horizon:  ports.CaseHorizon{Ref: "case_1", Items: []ports.HorizonItem{{ItemID: "item_1"}}},
		runResult: ports.RunListResult{
			Runs:          []ports.RunSummary{{RunID: "run_1", CaseID: "case_1", PlanItemID: "item_1", Execution: "completed", Settlement: "unsettled"}},
			UnreadableIDs: []string{"run_unreadable"},
		},
	}
}

func TestLiveSourceUsesOnlyScopedRunListAndRetainsUnreadableIDs(t *testing.T) {
	auth := validScopedAuthority()
	facts, err := NewLiveSource(auth, ports.ActorClaim{}).Facts(context.Background(), "case_1", ports.ActorClaim{}, "cursor")
	if err != nil {
		t.Fatal(err)
	}
	if auth.scopedCalls != 1 || auth.legacyCalls != 0 {
		t.Fatalf("scoped/legacy calls = %d/%d, want exactly 1/0", auth.scopedCalls, auth.legacyCalls)
	}
	if len(auth.scopedCases) != 1 || auth.scopedCases[0] != ports.CaseRef("case_1") {
		t.Fatalf("scoped case refs = %v, want exactly [case_1]", auth.scopedCases)
	}
	if len(facts.Runs) != 1 || facts.Runs[0].RunID != "run_1" {
		t.Fatalf("readable runs = %+v", facts.Runs)
	}
	if len(facts.UnreadableRunIDs) != 1 || facts.UnreadableRunIDs[0] != "run_unreadable" {
		t.Fatalf("unreadable run IDs = %v, want [run_unreadable]", facts.UnreadableRunIDs)
	}
}

func TestLiveSourceSelectsRequestedCaseAfterForeignRecord(t *testing.T) {
	auth := validScopedAuthority()
	auth.records = []ports.CaseRecord{{Ref: "case_other"}, auth.record}
	facts, err := NewLiveSource(auth, ports.ActorClaim{}).Facts(context.Background(), "case_1", ports.ActorClaim{}, "cursor")
	if err != nil {
		t.Fatal(err)
	}
	if facts.Record.Ref != ports.CaseRef("case_1") {
		t.Fatalf("selected record = %q, want exact requested case_1", facts.Record.Ref)
	}
	if auth.caseListCalls != 1 || auth.overviewCalls != 1 || auth.horizonCalls != 1 || auth.approvalCalls != 1 || auth.scopedCalls != 1 || auth.legacyCalls != 0 {
		t.Fatalf("authority calls list/overview/horizon/approvals/scoped/legacy = %d/%d/%d/%d/%d/%d, want 1/1/1/1/1/0",
			auth.caseListCalls, auth.overviewCalls, auth.horizonCalls, auth.approvalCalls, auth.scopedCalls, auth.legacyCalls)
	}
	if len(auth.scopedCases) != 1 || auth.scopedCases[0] != ports.CaseRef("case_1") {
		t.Fatalf("scoped case refs = %v, want exactly [case_1]", auth.scopedCases)
	}
}

func TestLiveSourceRejectsInvalidRequestedCaseBeforeAuthorityCalls(t *testing.T) {
	for _, caseID := range []string{"", " \t\n"} {
		t.Run(caseID, func(t *testing.T) {
			auth := validScopedAuthority()
			_, err := NewLiveSource(auth, ports.ActorClaim{}).Facts(context.Background(), caseID, ports.ActorClaim{}, "")
			if err == nil || apperr.KindOf(err) != apperr.KindInvalid {
				t.Fatalf("invalid case ID error = %v, want typed invalid", err)
			}
			if auth.caseListCalls != 0 || auth.scopedCalls != 0 || auth.legacyCalls != 0 {
				t.Fatalf("invalid case contacted authority: case-list=%d scoped=%d legacy=%d", auth.caseListCalls, auth.scopedCalls, auth.legacyCalls)
			}
		})
	}
}

func TestLiveSourceRejectsUnknownRequestedCaseBeforeDownstreamReads(t *testing.T) {
	cases := []struct {
		name    string
		records []ports.CaseRecord
	}{
		{name: "empty case list", records: []ports.CaseRecord{}},
		{name: "foreign case listed first", records: []ports.CaseRecord{{Ref: "case_other"}}},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			auth := validScopedAuthority()
			auth.records = tc.records
			_, err := NewLiveSource(auth, ports.ActorClaim{}).Facts(context.Background(), "case_1", ports.ActorClaim{}, "")
			if err == nil || apperr.KindOf(err) != apperr.KindInvalid {
				t.Fatalf("unknown requested case error = %v, want typed invalid/not-found", err)
			}
			if auth.caseListCalls != 1 || auth.overviewCalls != 0 || auth.horizonCalls != 0 || auth.approvalCalls != 0 || auth.scopedCalls != 0 || auth.legacyCalls != 0 {
				t.Fatalf("authority calls list/overview/horizon/approvals/scoped/legacy = %d/%d/%d/%d/%d/%d, want 1/0/0/0/0/0",
					auth.caseListCalls, auth.overviewCalls, auth.horizonCalls, auth.approvalCalls, auth.scopedCalls, auth.legacyCalls)
			}
		})
	}
}

func TestLiveSourceRejectsMismatchedAndUnownedScopedRunFacts(t *testing.T) {
	cases := []struct {
		name   string
		mutate func(*scopedLiveAuthority)
	}{
		{"overview-ref", func(a *scopedLiveAuthority) { a.overview.Ref = "case_other" }},
		{"horizon-ref", func(a *scopedLiveAuthority) { a.horizon.Ref = "case_other" }},
		{"foreign-readable-case", func(a *scopedLiveAuthority) { a.runResult.Runs[0].CaseID = "case_other" }},
		{"blank-run-id", func(a *scopedLiveAuthority) { a.runResult.Runs[0].RunID = " " }},
		{"blank-item-id", func(a *scopedLiveAuthority) { a.runResult.Runs[0].PlanItemID = " " }},
		{"missing-actual-parent", func(a *scopedLiveAuthority) { a.runResult.Runs[0].PlanItemID = "item_missing" }},
		{"duplicate-readable-id", func(a *scopedLiveAuthority) { a.runResult.Runs = append(a.runResult.Runs, a.runResult.Runs[0]) }},
		{"duplicate-unreadable-id", func(a *scopedLiveAuthority) {
			a.runResult.UnreadableIDs = append(a.runResult.UnreadableIDs, "run_unreadable")
		}},
		{"blank-unreadable-id", func(a *scopedLiveAuthority) { a.runResult.UnreadableIDs[0] = " " }},
		{"readable-unreadable-overlap", func(a *scopedLiveAuthority) { a.runResult.UnreadableIDs = []string{"run_1"} }},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			auth := validScopedAuthority()
			tc.mutate(auth)
			_, err := NewLiveSource(auth, ports.ActorClaim{}).Facts(context.Background(), "case_1", ports.ActorClaim{}, "")
			if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
				t.Fatalf("invalid scoped facts error = %v, want typed unavailable", err)
			}
			if auth.scopedCalls != 1 || auth.legacyCalls != 0 {
				t.Fatalf("scoped/legacy calls = %d/%d, want exactly 1/0", auth.scopedCalls, auth.legacyCalls)
			}
		})
	}
}
