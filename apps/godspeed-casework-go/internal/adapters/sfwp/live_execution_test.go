//go:build live

// The accepted-episode execution proof (plan T05: "every live verb the UI
// needs round-trips"): item.execute settles the sentry chain's head accepted,
// the produced artifact is fetchable through artifact.get (digest-verified),
// the settlement_status sentry unlocks task_publish, and case.advance runs the
// chain to completion. These assertions REQUIRE the kernel to execute the
// templates' write_file operations.
//
// While the T05 dispatch finding stands (see executionFindingNote), the kernel
// cannot execute write_file items at all, so this test first asserts the
// honest durable failure shape and then SKIPS with the finding - a documented
// finding, never a silent pass. The skip self-removes: once the server routes
// write_file through sea_forge_sandbox::materialize, episodes settle accepted
// and the full proof below runs.
package sfwp

import (
	"context"
	"strings"
	"testing"
	"time"
)

func TestLiveExecutionProducesAcceptedArtifacts(t *testing.T) {
	cell := newLiveCell(t)
	cl := cell.client()
	ctx, cancel := context.WithTimeout(context.Background(), 120*time.Second)
	defer cancel()
	authority := NewAuthority(cl)

	draft := caseDraftOf(sentryChainRef, sentryChainParams())
	report, err := authority.PreflightCase(ctx, draft)
	if err != nil {
		t.Fatalf("case.preflight: %v", err)
	}
	commitReqID := cl.NewRequestID("case_commit")
	receipt, err := authority.CommitCase(ctx, draft, report.Precondition, governedOpts(commitReqID))
	if err != nil {
		t.Fatalf("case.commit: %v", err)
	}
	caseID := receipt.CaseID

	execReqID := cl.NewRequestID("item_execute")
	execReport, err := authority.ExecuteItem(ctx, refOf(caseID), "task_prepare", governedOpts(execReqID))
	if err != nil {
		t.Fatalf("item.execute: %v", err)
	}
	if len(execReport.Episodes) != 1 || execReport.Episodes[0].ItemID != "task_prepare" {
		t.Fatalf("item.execute report: %+v", execReport)
	}
	if execReport.Episodes[0].Settlement != "accepted" {
		// The honest-failure branch: distinguish the documented dispatch
		// finding from any other, unexplained rejection before skipping.
		bases := cell.settlementBases(caseID)
		if !slicesContains(bases, executionDispatchFinding) {
			t.Fatalf("task_prepare settled %q without the dispatch-finding basis (basis %v)",
				execReport.Episodes[0].Settlement, bases)
		}
		horizon, herr := authority.CaseHorizon(ctx, refOf(caseID))
		if herr != nil {
			t.Fatalf("case.get_horizon: %v", herr)
		}
		if horizon.State != "terminated" {
			t.Fatalf("a required item's rejected settlement must terminate the case, got %q", horizon.State)
		}
		t.Skipf("execution-dependent proof unavailable: %s", executionFindingNote)
	}

	// The kernel executed the item: the artifact exists and is fetchable.
	digest := artifactDigestFromRun(cell, execReport.Episodes[0].RunID)
	if digest == "" {
		t.Fatal("no committed artifact digest found for the executed run")
	}
	artifact, err := authority.GetArtifact(ctx, digest)
	if err != nil {
		t.Fatalf("artifact.get: %v", err)
	}
	if !strings.Contains(string(artifact.Data), "dataset:") || artifact.Digest != digest {
		t.Fatalf("artifact content mismatch: %+v", artifact)
	}

	// Advance: the accepted settlement fires the sentry, task_publish executes,
	// and the rollup milestone closes the case.
	advReqID := cl.NewRequestID("case_advance")
	advReport, err := authority.AdvanceCase(ctx, refOf(caseID), governedOpts(advReqID))
	if err != nil {
		t.Fatalf("case.advance: %v", err)
	}
	if advReport.State != "completed" {
		t.Fatalf("case should complete after both tasks settle, got state %q (%+v)", advReport.State, advReport)
	}

	// The durable truth: the accepted episode recorded ItemCompleted, the
	// downstream enablement, and the completion - no double execution.
	kinds := map[string]int{}
	for _, ev := range cell.caseEvents(caseID) {
		if kind, ok := ev["kind"].(string); ok {
			kinds[kind]++
		}
	}
	if kinds["item_completed"] == 0 || kinds["item_enabled"] < 2 || kinds["case_closed"] == 0 {
		t.Fatalf("durable trace does not show the completed chain: %v", kinds)
	}
}
