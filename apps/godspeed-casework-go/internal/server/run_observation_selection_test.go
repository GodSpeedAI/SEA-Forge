package server

import (
	"reflect"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func selectionRun(runID, execution, settlement string, started time.Time, hasStarted bool, finished time.Time, hasFinished bool, evidenceCount int) ports.RunSummary {
	return ports.RunSummary{
		RunID: runID, CaseID: "case_fixture", PlanItemID: "item_" + runID,
		Execution: execution, Settlement: settlement,
		StartedAt: started, HasStarted: hasStarted, FinishedAt: finished, HasFinished: hasFinished,
		EvidenceCount: evidenceCount,
	}
}

func selectionInstant(hour int) time.Time {
	return time.Date(2026, time.October, 6, hour, 0, 0, 0, time.UTC)
}

func TestSelectObservationRunsRanksGroupsAndKeepsTheFirstEight(t *testing.T) {
	activeA := time.Date(2026, time.October, 6, 9, 0, 0, 0, time.FixedZone("minus-four", -4*60*60))
	activeZ := selectionInstant(13)
	runs := []ports.RunSummary{
		selectionRun("run_terminal_missing", "terminated", "escalated", time.Time{}, false, time.Time{}, false, 10),
		selectionRun("run_pending_start", "pending", "unsettled", selectionInstant(17), true, time.Time{}, false, 11),
		selectionRun("run_active_missing", "active", "rejected", time.Time{}, false, time.Time{}, false, 12),
		selectionRun("run_failed", "failed", "accepted", selectionInstant(16), true, selectionInstant(19), true, 13),
		selectionRun("run_active_finished", "active", "unsettled", selectionInstant(20), true, selectionInstant(12), true, 14),
		selectionRun("run_enabled_finished", "enabled", "rejected", selectionInstant(20), true, selectionInstant(15), true, 15),
		selectionRun("run_active_z", "active", "accepted", selectionInstant(11), true, activeZ, true, 16),
		selectionRun("run_completed", "completed", "unsettled", selectionInstant(8), true, selectionInstant(21), true, 17),
		selectionRun("run_active_a", "active", "escalated", selectionInstant(10), true, activeA, true, 18),
		selectionRun("run_active_started", "active", "accepted", selectionInstant(14), true, time.Time{}, false, 19),
	}
	want := []ports.RunSummary{
		selectionRun("run_active_started", "active", "accepted", selectionInstant(14), true, time.Time{}, false, 19),
		selectionRun("run_active_a", "active", "escalated", selectionInstant(10), true, activeA, true, 18),
		selectionRun("run_active_z", "active", "accepted", selectionInstant(11), true, activeZ, true, 16),
		selectionRun("run_active_finished", "active", "unsettled", selectionInstant(20), true, selectionInstant(12), true, 14),
		selectionRun("run_active_missing", "active", "rejected", time.Time{}, false, time.Time{}, false, 12),
		selectionRun("run_pending_start", "pending", "unsettled", selectionInstant(17), true, time.Time{}, false, 11),
		selectionRun("run_enabled_finished", "enabled", "rejected", selectionInstant(20), true, selectionInstant(15), true, 15),
		selectionRun("run_completed", "completed", "unsettled", selectionInstant(8), true, selectionInstant(21), true, 17),
	}

	for _, input := range [][]ports.RunSummary{runs, reverseSelectionRuns(runs)} {
		before := append([]ports.RunSummary(nil), input...)
		got, err := selectObservationRuns(input)
		if err != nil {
			t.Fatalf("selectObservationRuns() error = %v, want successful selection", err)
		}
		if !reflect.DeepEqual(got, want) {
			t.Errorf("selected runs = %+v, want %+v", got, want)
		}
		if !reflect.DeepEqual(input, before) {
			t.Errorf("input summaries changed: got %+v, before %+v", input, before)
		}
	}
}

func TestSelectObservationRunsRanksAllTerminalStatesTogether(t *testing.T) {
	runs := []ports.RunSummary{
		selectionRun("run_terminal_missing", "terminated", "rejected", time.Time{}, false, time.Time{}, false, 6),
		selectionRun("run_terminal_completed_old", "completed", "accepted", selectionInstant(12), true, selectionInstant(15), true, 5),
		selectionRun("run_terminal_failed_latest", "failed", "unsettled", selectionInstant(11), true, selectionInstant(18), true, 4),
		selectionRun("run_terminal_completed_tie_z", "completed", "escalated", selectionInstant(10), true, selectionInstant(16), true, 3),
		selectionRun("run_terminal_started_fallback", "terminated", "accepted", selectionInstant(17), true, time.Time{}, false, 2),
		selectionRun("run_terminal_failed_tie_a", "failed", "rejected", selectionInstant(9), true, selectionInstant(16), true, 1),
	}
	want := []ports.RunSummary{
		selectionRun("run_terminal_failed_latest", "failed", "unsettled", selectionInstant(11), true, selectionInstant(18), true, 4),
		selectionRun("run_terminal_started_fallback", "terminated", "accepted", selectionInstant(17), true, time.Time{}, false, 2),
		selectionRun("run_terminal_completed_tie_z", "completed", "escalated", selectionInstant(10), true, selectionInstant(16), true, 3),
		selectionRun("run_terminal_failed_tie_a", "failed", "rejected", selectionInstant(9), true, selectionInstant(16), true, 1),
		selectionRun("run_terminal_completed_old", "completed", "accepted", selectionInstant(12), true, selectionInstant(15), true, 5),
		selectionRun("run_terminal_missing", "terminated", "rejected", time.Time{}, false, time.Time{}, false, 6),
	}
	before := append([]ports.RunSummary(nil), runs...)
	got, err := selectObservationRuns(runs)
	if err != nil {
		t.Fatalf("selectObservationRuns() error = %v, want successful terminal selection", err)
	}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("selected terminal runs = %+v, want %+v", got, want)
	}
	if !reflect.DeepEqual(runs, before) {
		t.Errorf("terminal input summaries changed: got %+v, before %+v", runs, before)
	}
}

func TestSelectObservationRunsHandlesEmptyAndShortInput(t *testing.T) {
	t.Run("empty", func(t *testing.T) {
		var runs []ports.RunSummary
		got, err := selectObservationRuns(runs)
		if err != nil {
			t.Fatalf("selectObservationRuns(%d runs) error = %v, want success", len(runs), err)
		}
		if len(got) != 0 {
			t.Errorf("empty input selected %d runs, want none", len(got))
		}
	})

	t.Run("empty nonnil", func(t *testing.T) {
		runs := []ports.RunSummary{}
		got, err := selectObservationRuns(runs)
		if err != nil {
			t.Fatalf("selectObservationRuns(%d runs) error = %v, want success", len(runs), err)
		}
		if len(got) != 0 {
			t.Errorf("empty input selected %d runs, want none", len(got))
		}
	})

	t.Run("short", func(t *testing.T) {
		runs := []ports.RunSummary{
			selectionRun("run_short", "enabled", "unsettled", selectionInstant(9), true, time.Time{}, false, 4),
		}
		before := append([]ports.RunSummary(nil), runs...)
		got, err := selectObservationRuns(runs)
		if err != nil {
			t.Fatalf("selectObservationRuns(%d runs) error = %v, want success", len(runs), err)
		}
		if len(got) != len(runs) || !reflect.DeepEqual(got, runs) {
			t.Fatalf("selected runs = %+v, want input order and values %+v", got, runs)
		}
		got[0].RunID = "mutated-output"
		if !reflect.DeepEqual(runs, before) {
			t.Errorf("mutating short output changed input: got %+v, before %+v", runs, before)
		}
	})
}

func TestSelectObservationRunsRejectsUnknownExecutionWithoutPartialSelection(t *testing.T) {
	runs := []ports.RunSummary{
		selectionRun("run_active_1", "active", "unsettled", selectionInstant(1), true, time.Time{}, false, 1),
		selectionRun("run_active_2", "active", "unsettled", selectionInstant(2), true, time.Time{}, false, 2),
		selectionRun("run_active_3", "active", "unsettled", selectionInstant(3), true, time.Time{}, false, 3),
		selectionRun("run_active_4", "active", "unsettled", selectionInstant(4), true, time.Time{}, false, 4),
		selectionRun("run_active_5", "active", "unsettled", selectionInstant(5), true, time.Time{}, false, 5),
		selectionRun("run_active_6", "active", "unsettled", selectionInstant(6), true, time.Time{}, false, 6),
		selectionRun("run_active_7", "active", "unsettled", selectionInstant(7), true, time.Time{}, false, 7),
		selectionRun("run_active_8", "active", "unsettled", selectionInstant(8), true, time.Time{}, false, 8),
		selectionRun("run_unsupported", "unknown", "accepted", selectionInstant(9), true, time.Time{}, false, 9),
	}
	before := append([]ports.RunSummary(nil), runs...)
	got, err := selectObservationRuns(runs)
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("unknown execution error = %v, want typed unavailable", err)
	}
	if len(got) != 0 {
		t.Fatalf("unknown execution returned partial selection %+v", got)
	}
	if !reflect.DeepEqual(runs, before) {
		t.Errorf("unsupported-standing input summaries changed: got %+v, before %+v", runs, before)
	}
}

func reverseSelectionRuns(runs []ports.RunSummary) []ports.RunSummary {
	reversed := make([]ports.RunSummary, len(runs))
	for i := range runs {
		reversed[len(runs)-1-i] = runs[i]
	}
	return reversed
}
