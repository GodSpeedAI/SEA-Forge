package server

import (
	"reflect"
	"strings"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

type runObservationPresentContextHistory interface {
	Trajectory(caseID string) []projection.Revision
}

type runObservationPresentContextRelay interface {
	CursorForCase(caseID string) (string, bool)
}

type runObservationPresentContext struct {
	caseID    string
	cursor    string
	parentIDs map[string]struct{}
}

func checkRunObservationPresentContext(
	history runObservationPresentContextHistory,
	relay runObservationPresentContextRelay,
	caseID string,
) (runObservationPresentContext, error) {
	unavailable := func() (runObservationPresentContext, error) {
		return runObservationPresentContext{}, apperr.New(
			apperr.KindUnavailable,
			"",
			"run_observation",
			"present case context is not available",
		)
	}
	if strings.TrimSpace(caseID) == "" || nilRunObservationPresentContextDependency(history) || nilRunObservationPresentContextDependency(relay) {
		return unavailable()
	}

	revisions := history.Trajectory(caseID)
	if len(revisions) == 0 {
		return unavailable()
	}
	newest := revisions[len(revisions)-1]
	if newest.CaseID != caseID || strings.TrimSpace(newest.Cursor) == "" ||
		newest.Snapshot.CaseID != caseID || newest.Snapshot.Cursor != newest.Cursor || newest.Facts == nil {
		return unavailable()
	}
	facts := newest.Facts
	if facts.Cursor != newest.Cursor || string(facts.Record.Ref) != caseID || string(facts.Overview.Ref) != caseID || string(facts.Horizon.Ref) != caseID || facts.Horizon.Items == nil {
		return unavailable()
	}

	parentIDs := make(map[string]struct{}, len(facts.Horizon.Items))
	for _, item := range facts.Horizon.Items {
		if strings.TrimSpace(item.ItemID) == "" {
			return unavailable()
		}
		if _, exists := parentIDs[item.ItemID]; exists {
			return unavailable()
		}
		parentIDs[item.ItemID] = struct{}{}
	}

	observedCursor, observed := relay.CursorForCase(caseID)
	if !observed || strings.TrimSpace(observedCursor) == "" || observedCursor != newest.Cursor {
		return unavailable()
	}
	return runObservationPresentContext{
		caseID:    caseID,
		cursor:    newest.Cursor,
		parentIDs: parentIDs,
	}, nil
}

func nilRunObservationPresentContextDependency(value any) bool {
	if value == nil {
		return true
	}
	reflected := reflect.ValueOf(value)
	switch reflected.Kind() {
	case reflect.Chan, reflect.Func, reflect.Interface, reflect.Map, reflect.Pointer, reflect.Slice:
		return reflected.IsNil()
	default:
		return false
	}
}
