package server

import (
	"context"
	"encoding/json"
	"net/http"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

type capturingFactsSource struct {
	facts      projection.CaseFacts
	factsCalls int
	snapCalls  int
}

func (s *capturingFactsSource) Facts(_ context.Context, _ string, actor ports.ActorClaim, cursor string) (projection.CaseFacts, error) {
	s.factsCalls++
	facts := projection.CloneFacts(&s.facts)
	facts.Actor = actor
	facts.Cursor = cursor
	return *facts, nil
}

func (s *capturingFactsSource) Snapshot(context.Context, string, ports.ActorClaim, string) (contract.CognitiveWorldSnapshot, error) {
	s.snapCalls++
	return contract.CognitiveWorldSnapshot{}, nil
}

func (*capturingFactsSource) NewestCaseID(context.Context) (string, error) { return "case_1", nil }

func TestRelayBuildsSnapshotAndRetainedFactsFromOneCapture(t *testing.T) {
	store := projection.NewStore()
	source := &capturingFactsSource{facts: *retainedFacts("", "case_1", "pending")}
	relay := NewRelay(nil, source, store, RelayOptions{DefaultActor: ports.ActorClaim{ActorID: "operator_local", Role: "operator"}})
	relay.accept(context.Background(), KernelEvent{Cursor: "01AAA", CaseID: "case_1", Kind: "case.trace.item_activated", At: time.Now()})
	rev, err := store.At("01AAA")
	if err != nil {
		t.Fatal(err)
	}
	if source.factsCalls != 1 || source.snapCalls != 0 || rev.Facts == nil || rev.Facts.Cursor != "01AAA" || rev.Snapshot.Cursor != "01AAA" {
		t.Fatalf("relay must build and retain one captured cursor view: facts calls=%d snapshot calls=%d revision=%+v", source.factsCalls, source.snapCalls, rev)
	}
}

func retainedFacts(cursor, caseID, execution string) *projection.CaseFacts {
	return &projection.CaseFacts{
		Record:   ports.CaseRecord{Ref: ports.CaseRef(caseID), State: "active", Summary: "captured " + execution},
		Overview: ports.CaseOverview{Ref: ports.CaseRef(caseID), State: "active", Stages: []string{"stage"}},
		Horizon: ports.CaseHorizon{Ref: ports.CaseRef(caseID), State: "active", Items: []ports.HorizonItem{{
			ItemID: "item", Name: "Item", Kind: "sandboxed_task", Execution: execution,
		}}},
		Actor:  ports.ActorClaim{ActorID: "operator_local", Role: "operator"},
		Cursor: cursor,
		Now:    time.Date(2026, 9, 29, 12, 0, 0, 0, time.UTC),
	}
}

func appendFacts(t *testing.T, store *projection.Store, cursor, caseID, execution string) {
	t.Helper()
	facts := retainedFacts(cursor, caseID, execution)
	if err := store.Append(projection.Revision{
		Cursor: cursor, At: facts.Now, CaseID: caseID, Summary: "case.trace." + execution,
		Snapshot: projection.Build(*facts), Facts: facts,
	}); err != nil {
		t.Fatal(err)
	}
}

func TestSessionHistoricalAndSSEViewsUseRetainedFacts(t *testing.T) {
	h := newLiveHarness(t)
	appendFacts(t, h.store, "01AAA", "case_1", "pending")
	appendFacts(t, h.store, "01BBB", "case_1", "enabled")
	op := h.login(t, "operator", "ignored-in-dev")
	rso := h.login(t, "rso", "ignored-in-dev")

	resp, err := rso.get(h.ts.URL + "/api/world?case_id=case_1&cursor=01AAA")
	if err != nil {
		t.Fatal(err)
	}
	var historical worldResponse
	if err := json.NewDecoder(resp.Body).Decode(&historical); err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	if historical.Snapshot.Cursor != "01AAA" || historical.Snapshot.Summary.StatusPhrase != "Waiting on entry criteria." {
		t.Fatalf("historical read must retain the requested old revision: %+v", historical.Snapshot)
	}
	if historical.Snapshot.Perspective.ActorID != "rso_local" || historical.Snapshot.Perspective.Role != "R-SO" {
		t.Fatalf("historical read must identify its session actor: %+v", historical.Snapshot.Perspective)
	}

	resp, err = op.get(h.ts.URL + "/api/world?cursor=01BBB")
	if err != nil {
		t.Fatal(err)
	}
	var current worldResponse
	if err := json.NewDecoder(resp.Body).Decode(&current); err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	if current.Snapshot.Perspective.ActorID != "operator_local" || len(current.Snapshot.AvailableActions) == 0 {
		t.Fatalf("operator historical read should retain operator offers: %+v", current.Snapshot)
	}

	stream, err := rso.get(h.ts.URL + "/api/events?last=01AAA")
	if err != nil {
		t.Fatal(err)
	}
	defer stream.Body.Close()
	events := streamLiveSSE(t, stream.Body)
	nextLiveEvent(t, events, "hello")
	frameEvent := nextLiveEvent(t, events, "role-specific replay")
	var frame contract.StreamEvent
	if err := json.Unmarshal([]byte(frameEvent.data), &frame); err != nil {
		t.Fatal(err)
	}
	snap, ok := frame.Payload.(map[string]any)
	if !ok || snap["cursor"] != "01BBB" {
		t.Fatalf("SSE replay must preserve its kernel cursor: %#v", frame.Payload)
	}
	perspective, _ := snap["perspective"].(map[string]any)
	if perspective["actor_id"] != "rso_local" || perspective["role"] != "R-SO" {
		t.Fatalf("SSE replay must identify the authenticated actor: %#v", snap["perspective"])
	}
	if actions, _ := snap["available_actions"].([]any); len(actions) != 0 {
		t.Fatalf("R-SO stream must not inherit operator execution offers: %#v", actions)
	}
}

func TestSessionReadRefusalsPrecedeCachedLookupAndFailClosed(t *testing.T) {
	h := newLiveHarness(t)
	appendFacts(t, h.store, "01AAA", "case_1", "enabled")
	op := h.login(t, "operator", "ignored-in-dev")
	for _, path := range []string{
		"/api/world?cursor=01AAA&actor=",
		"/api/world?cursor=01AAA&role=operator",
		"/api/world?case_id=case_other&cursor=01AAA",
	} {
		resp, err := op.get(h.ts.URL + path)
		if err != nil {
			t.Fatal(err)
		}
		resp.Body.Close()
		if resp.StatusCode != http.StatusBadRequest {
			t.Errorf("%s status = %d, want 400", path, resp.StatusCode)
		}
	}
	h.world.verify = func(ports.ActorClaim) error {
		return apperr.New(apperr.KindAuthorityDenied, "", "identity", "revoked delegation")
	}
	for _, path := range []string{"/api/world?cursor=01AAA", "/api/events"} {
		resp, err := op.get(h.ts.URL + path)
		if err != nil {
			t.Fatal(err)
		}
		resp.Body.Close()
		if resp.StatusCode != http.StatusForbidden {
			t.Errorf("revoked perspective %s status = %d, want fail-closed 403", path, resp.StatusCode)
		}
	}
}

func TestLegacyRevisionCannotCrossSessionPerspective(t *testing.T) {
	h := newLiveHarness(t)
	legacy := snapshotFor("01AAA", "case_1") // relay default: operator perspective, no captured facts
	if err := h.store.Append(projection.Revision{Cursor: "01AAA", CaseID: "case_1", Snapshot: legacy}); err != nil {
		t.Fatal(err)
	}
	rso := h.login(t, "rso", "ignored-in-dev")
	resp, err := rso.get(h.ts.URL + "/api/world?cursor=01AAA")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusServiceUnavailable {
		t.Fatalf("legacy operator revision must not be guessed into R-SO perspective: status %d", resp.StatusCode)
	}
}
