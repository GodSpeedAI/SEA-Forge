package server

import (
	"encoding/json"
	"net/http"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

func TestTrajectoryReturnsCaseScopedRetainedRevisionsWithGroundedFields(t *testing.T) {
	h := newLiveHarness(t)
	activeStage := contract.CognitiveObject{ID: "stage-build", Kind: "stage", Status: "IN_PROGRESS"}
	objects := []contract.CognitiveObject{
		activeStage,
		{ID: "done", Kind: "work_item", Status: "COMPLETED"},
		{ID: "rejected", Kind: "work_item", Status: "REJECTED"},
		{ID: "waiting", Kind: "milestone", Status: "WAITING"},
	}
	for _, rev := range []projection.Revision{
		{
			Cursor: "01AAA", At: time.Date(2026, 9, 29, 12, 0, 0, 0, time.UTC),
			CaseID: "case_1", Summary: "case.submitted", Snapshot: contract.CognitiveWorldSnapshot{
				CaseID: "case_1", Cursor: "01AAA", VisibleObjects: objects,
				Perspective: contract.ActorPerspective{ActorID: "gateway_view", Role: "operator"},
			},
		},
		{
			Cursor: "01BBB", At: time.Date(2026, 9, 29, 12, 1, 0, 0, time.UTC),
			CaseID: "case_other", Summary: "case.trace.item_activated", Snapshot: snapshotFor("01BBB", "case_other"),
		},
		{
			Cursor: "01CCC", At: time.Date(2026, 9, 29, 12, 2, 0, 0, time.UTC),
			CaseID: "case_1", Summary: "case.trace.item_completed", Snapshot: contract.CognitiveWorldSnapshot{
				CaseID: "case_1", Cursor: "01CCC", VisibleObjects: objects,
			},
		},
	} {
		if err := h.store.Append(rev); err != nil {
			t.Fatal(err)
		}
	}
	var verified ports.ActorClaim
	h.world.verify = func(actor ports.ActorClaim) error {
		verified = actor
		return nil
	}
	op := h.login(t, "operator", "ignored-in-dev")
	resp, err := op.get("/api/trajectory?case_id=case_1")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("trajectory status = %d", resp.StatusCode)
	}
	var got temporalTrajectoryResponse
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatal(err)
	}
	if verified != (ports.ActorClaim{ActorID: "operator_local", Role: "operator"}) {
		t.Fatalf("trajectory must verify the session perspective before serving history: %+v", verified)
	}
	if got.CaseID != "case_1" || got.BaseCursor != "01AAA" || got.HeadCursor != "01CCC" || len(got.Points) != 2 {
		t.Fatalf("trajectory bounds/case scope are incorrect: %+v", got)
	}
	point := got.Points[0]
	if point.EventType != "case.submitted" || point.Summary != "case.submitted" || !point.Consequential {
		t.Fatalf("kernel event semantics were not preserved: %+v", point)
	}
	if point.Timestamp != "2026-09-29T12:00:00Z" || point.ActiveStageID != "stage-build" || point.TotalPlanItemsCount != 3 || point.CompletedPlanItemsCount != 1 {
		t.Fatalf("checkpoint fields must come from the retained snapshot and event: %+v", point)
	}
	if point.ActorID != "" || point.ActorRole != "" {
		t.Fatalf("the relay has no originating actor metadata; actor fields must remain unknown: %+v", point)
	}
}

func TestTrajectoryRequiresSessionBeforePerspectiveVerification(t *testing.T) {
	h := newLiveHarness(t)
	verified := false
	h.world.verify = func(ports.ActorClaim) error {
		verified = true
		return nil
	}
	resp, err := http.Get(h.ts.URL + "/api/trajectory?case_id=case_1")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusUnauthorized || verified {
		t.Fatalf("anonymous trajectory request must stop before perspective verification: status=%d verified=%t", resp.StatusCode, verified)
	}
}

func TestTrajectoryRejectsInvalidAndUnavailableHistory(t *testing.T) {
	h := newLiveHarness(t)
	op := h.login(t, "operator", "ignored-in-dev")
	for _, path := range []string{"/api/trajectory", "/api/trajectory?case_id=", "/api/trajectory?case_id=case_1&extra=1"} {
		resp, err := op.get(path)
		if err != nil {
			t.Fatal(err)
		}
		resp.Body.Close()
		if resp.StatusCode != http.StatusBadRequest {
			t.Fatalf("invalid trajectory query %q: status=%d", path, resp.StatusCode)
		}
	}
	resp, err := op.get("/api/trajectory?case_id=case_missing")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var body errorBody
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != http.StatusNotFound || body.Error.Kind != "not_found" {
		t.Fatalf("missing or unretained history must be typed not_found: status=%d body=%+v", resp.StatusCode, body)
	}
}

func TestTrajectoryRefusesDeniedPerspective(t *testing.T) {
	h := newLiveHarness(t)
	if err := h.store.Append(projection.Revision{
		Cursor: "01AAA", CaseID: "case_1", Summary: "case.submitted", Snapshot: snapshotFor("01AAA", "case_1"),
	}); err != nil {
		t.Fatal(err)
	}
	h.world.verify = func(ports.ActorClaim) error {
		return apperr.New(apperr.KindAuthorityDenied, "", "identity", "denied")
	}
	op := h.login(t, "operator", "ignored-in-dev")
	resp, err := op.get("/api/trajectory?case_id=case_1")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var body errorBody
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != http.StatusForbidden || body.Error.Kind != "authority_denied" {
		t.Fatalf("a denied session perspective must not read history: status=%d body=%+v", resp.StatusCode, body)
	}
}

func TestKnownConsequentialEventKindsAreExplicit(t *testing.T) {
	for _, kind := range []string{
		"case.submitted", "case.trace.case_created", "case.trace.run_started", "case.trace.plan_created",
		"case.trace.authority_evaluated", "case.trace.workspace_created", "case.trace.command_started",
		"case.trace.command_finished", "case.trace.artifact_captured", "case.trace.settlement_recorded",
		"case.trace.run_halted", "case.trace.run_finished", "case.trace.case_closed",
		"case.trace.milestone_achieved", "case.trace.plan_mutated", "case.trace.case_file_item_added",
		"case.trace.item_enabled", "case.trace.item_activated", "case.trace.item_completed",
		"case.trace.item_failed", "case.trace.item_terminated", "case.trace.human_task_completed",
		"case.trace.case_reopened", "case.trace.case_terminated",
	} {
		if !knownConsequentialEvent(kind) {
			t.Errorf("known state-changing event %q was not marked consequential", kind)
		}
	}
	for _, kind := range []string{"case.trace.internal_error", "case.trace.future_kind", "", "execution_progress"} {
		if knownConsequentialEvent(kind) {
			t.Errorf("informational or unknown event kind %q must not be consequential", kind)
		}
	}
}
