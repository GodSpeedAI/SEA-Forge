package server

import (
	"net/http"
	"strings"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// temporalTrajectoryResponse mirrors the existing TemporalTrajectoryResponse contract. Its
// points are built only from the process's bounded retained kernel revisions.
type temporalTrajectoryResponse struct {
	CaseID     string               `json:"case_id"`
	BaseCursor string               `json:"base_cursor"`
	HeadCursor string               `json:"head_cursor"`
	Points     []temporalCheckpoint `json:"points"`
}

type temporalCheckpoint struct {
	Cursor                  string `json:"cursor"`
	Timestamp               string `json:"timestamp"`
	EventType               string `json:"event_type"`
	Summary                 string `json:"summary"`
	ActorID                 string `json:"actor_id"`
	ActorRole               string `json:"actor_role"`
	Consequential           bool   `json:"consequential"`
	ActiveStageID           string `json:"active_stage_id,omitempty"`
	CompletedPlanItemsCount int    `json:"completed_plan_items_count"`
	TotalPlanItemsCount     int    `json:"total_plan_items_count"`
}

func (s *Server) handleTrajectory(w http.ResponseWriter, r *http.Request) {
	query := r.URL.Query()
	caseIDs, hasCaseID := query["case_id"]
	if len(query) != 1 || !hasCaseID || len(caseIDs) != 1 || strings.TrimSpace(caseIDs[0]) == "" || strings.TrimSpace(caseIDs[0]) != caseIDs[0] {
		writeTypedError(w, http.StatusBadRequest, "invalid", "exactly one non-empty case_id is required")
		return
	}
	caseID := caseIDs[0]

	identity := identityFrom(r)
	if identity == nil || identity.Session == nil {
		writeTypedError(w, http.StatusUnauthorized, "unauthorized", "a logged-in session is required for trajectory reads")
		return
	}
	actor := sessionIdentityOf(r).Claim()
	verifier, ok := s.world.(PerspectiveVerifier)
	if !ok {
		writeTypedError(w, http.StatusServiceUnavailable, "unavailable", "trajectory identity verification is not configured")
		return
	}
	if err := verifier.VerifyPerspective(r.Context(), actor); err != nil {
		if apperr.KindOf(err) == apperr.KindAuthorityDenied {
			writeTypedError(w, http.StatusForbidden, "authority_denied", "the kernel refused this trajectory read")
		} else {
			writeTypedError(w, http.StatusBadGateway, "unavailable", "the kernel could not verify the session identity")
		}
		return
	}
	if s.store == nil {
		writeTypedError(w, http.StatusServiceUnavailable, "unavailable", "trajectory history is not configured")
		return
	}

	revisions := s.store.Trajectory(caseID)
	if len(revisions) == 0 {
		writeTypedError(w, http.StatusNotFound, "not_found", "no retained trajectory is available; the case may be unknown, the relay may still be replaying, or its revisions may have been evicted")
		return
	}
	points := make([]temporalCheckpoint, 0, len(revisions))
	for _, rev := range revisions {
		points = append(points, checkpointFromRevision(rev))
	}
	writeJSON(w, http.StatusOK, temporalTrajectoryResponse{
		CaseID:     caseID,
		BaseCursor: revisions[0].Cursor,
		HeadCursor: revisions[len(revisions)-1].Cursor,
		Points:     points,
	})
}

func checkpointFromRevision(rev projection.Revision) temporalCheckpoint {
	point := temporalCheckpoint{
		Cursor:        rev.Cursor,
		Timestamp:     revisionTimestamp(rev.At),
		EventType:     rev.Summary,
		Summary:       rev.Summary,
		Consequential: knownConsequentialEvent(rev.Summary),
	}
	activeStage := ""
	activeStages := 0
	for _, object := range rev.Snapshot.VisibleObjects {
		switch object.Kind {
		case "work_item", "milestone":
			point.TotalPlanItemsCount++
			if object.Status == "COMPLETED" {
				point.CompletedPlanItemsCount++
			}
		case "stage":
			if object.Status == "IN_PROGRESS" {
				activeStage = object.ID
				activeStages++
			}
		}
	}
	// An active stage is reported only when the source snapshot identifies exactly one.
	if activeStages == 1 {
		point.ActiveStageID = activeStage
	}
	// Relay revisions contain a serving perspective, not the actor that caused the kernel event.
	// Leave attribution unknown instead of copying the projected user into event attribution.
	return point
}

func revisionTimestamp(at time.Time) string {
	if at.IsZero() {
		return ""
	}
	return at.UTC().Format(time.RFC3339Nano)
}

// The kernel TraceKind vocabulary is intentionally enumerated here. Unknown and
// informational kinds do not get effects inferred from their payload.
func knownConsequentialEvent(kind string) bool {
	switch kind {
	case "case.submitted",
		"case.trace.case_created",
		"case.trace.run_started",
		"case.trace.plan_created",
		"case.trace.authority_evaluated",
		"case.trace.workspace_created",
		"case.trace.command_started",
		"case.trace.command_finished",
		"case.trace.artifact_captured",
		"case.trace.settlement_recorded",
		"case.trace.run_halted",
		"case.trace.run_finished",
		"case.trace.case_closed",
		"case.trace.milestone_achieved",
		"case.trace.plan_mutated",
		"case.trace.case_file_item_added",
		"case.trace.item_enabled",
		"case.trace.item_activated",
		"case.trace.item_completed",
		"case.trace.item_failed",
		"case.trace.item_terminated",
		"case.trace.human_task_completed",
		"case.trace.case_reopened",
		"case.trace.case_terminated":
		return true
	default:
		return false
	}
}
