package projection

import (
	"errors"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func rev(cursor string, caseID string) Revision {
	return Revision{
		Cursor:   cursor,
		At:       time.Date(2026, 9, 25, 12, 0, 0, 0, time.UTC),
		CaseID:   caseID,
		Summary:  "case.trace.item_activated",
		Snapshot: emptyTestSnapshot(cursor, caseID),
	}
}

func emptyTestSnapshot(cursor, caseID string) contract.CognitiveWorldSnapshot {
	return contract.CognitiveWorldSnapshot{Cursor: cursor, CaseID: caseID}
}

func TestStoreAppendLookUpAndMonotonicity(t *testing.T) {
	s := NewStore()
	if _, ok := s.Live(); ok {
		t.Fatal("an empty store must have no live revision")
	}
	if err := s.Append(rev("01AAA", "case_1")); err != nil {
		t.Fatal(err)
	}
	if err := s.Append(rev("01BBB", "case_1")); err != nil {
		t.Fatal(err)
	}
	if got := s.Head(); got != "01BBB" {
		t.Fatalf("head = %q", got)
	}
	if got := s.Oldest(); got != "01AAA" {
		t.Fatalf("oldest = %q", got)
	}
	revAt, err := s.At("01AAA")
	if err != nil || revAt.CaseID != "case_1" {
		t.Fatalf("At: %v %+v", err, revAt)
	}
	// Kernel cursors are monotonic: a non-advancing append is a relay bug and is refused rather
	// than silently reordering history.
	if err := s.Append(rev("01AAA", "case_1")); err == nil {
		t.Fatal("a repeated cursor must be refused")
	}
	if err := s.Append(rev("01AAA0-earlier", "case_1")); err == nil {
		t.Fatal("a cursor that sorts before head must be refused")
	}
	if err := s.Append(Revision{CaseID: "case_1"}); err == nil {
		t.Fatal("an empty kernel cursor must be refused: the store never mints cursors")
	}
}

func TestStoreBoundedRetentionAndDocumentedEviction(t *testing.T) {
	s := NewStoreWithRetention(2)
	for _, c := range []string{"01AAA", "01BBB", "01CCC"} {
		if err := s.Append(rev(c, "case_1")); err != nil {
			t.Fatal(err)
		}
	}
	if s.Len() != 2 {
		t.Fatalf("retention must bound the store, len = %d", s.Len())
	}
	if _, err := s.At("01AAA"); !errors.Is(err, ErrUnknownCursor) {
		t.Fatalf("an evicted cursor must answer ErrUnknownCursor, got %v", err)
	}
	if _, err := s.At("01ZZZ"); !errors.Is(err, ErrUnknownCursor) {
		t.Fatalf("an unknown cursor must answer ErrUnknownCursor, got %v", err)
	}
	if got := s.Oldest(); got != "01BBB" {
		t.Fatalf("oldest after eviction = %q", got)
	}
}

func TestStoreTrajectoryIsCaseScopedBoundedAndCloned(t *testing.T) {
	s := NewStoreWithRetention(3)
	name := "original"
	percent := 0.5
	snapshot := contract.CognitiveWorldSnapshot{
		CaseID: "case_1", Cursor: "01AAA",
		Perspective:    contract.ActorPerspective{DisplayName: &name},
		Summary:        contract.WorldSummary{ProgressPercent: &percent},
		VisibleObjects: []contract.CognitiveObject{{ID: "item", DependsOn: []string{"dep"}}},
	}
	if err := s.Append(Revision{Cursor: "01AAA", CaseID: "case_1", Snapshot: snapshot}); err != nil {
		t.Fatal(err)
	}
	if err := s.Append(rev("01BBB", "case_2")); err != nil {
		t.Fatal(err)
	}
	if err := s.Append(rev("01CCC", "case_1")); err != nil {
		t.Fatal(err)
	}
	got := s.Trajectory("case_1")
	if len(got) != 2 || got[0].Cursor != "01AAA" || got[1].Cursor != "01CCC" {
		t.Fatalf("trajectory must be ordered and case-scoped: %+v", got)
	}
	*got[0].Snapshot.Perspective.DisplayName = "mutated"
	*got[0].Snapshot.Summary.ProgressPercent = 1
	got[0].Snapshot.VisibleObjects[0].DependsOn[0] = "mutated"
	stored, err := s.At("01AAA")
	if err != nil {
		t.Fatal(err)
	}
	if *stored.Snapshot.Perspective.DisplayName != "original" || *stored.Snapshot.Summary.ProgressPercent != 0.5 || stored.Snapshot.VisibleObjects[0].DependsOn[0] != "dep" {
		t.Fatalf("trajectory query leaked mutable snapshot state: %+v", stored.Snapshot)
	}
	if empty := s.Trajectory("case_missing"); len(empty) != 0 {
		t.Fatalf("unknown case should have no retained points: %+v", empty)
	}
	if err := s.Append(rev("01DDD", "case_3")); err != nil {
		t.Fatal(err)
	}
	if points := s.Trajectory("case_1"); len(points) != 1 || points[0].Cursor != "01CCC" {
		t.Fatalf("evicted case history must be bounded honestly: %+v", points)
	}
}

func TestStoreRetainsDeepImmutableFactsForEachCursor(t *testing.T) {
	s := NewStoreWithRetention(2)
	facts := &CaseFacts{
		Record:    ports.CaseRecord{Ref: "case_1", Summary: "captured old"},
		Overview:  ports.CaseOverview{Ref: "case_1", Stages: []string{"stage"}, Settlements: []ports.SettlementNote{{Basis: []string{"basis"}}}},
		Horizon:   ports.CaseHorizon{Items: []ports.HorizonItem{{ItemID: "item", DependsOn: []string{"dep"}}}},
		Approvals: []ports.ApprovalRecord{{ApprovalID: "approval"}},
		Runs:      []ports.RunSummary{{RunID: "run"}},
	}
	if err := s.Append(Revision{Cursor: "01AAA", CaseID: "case_1", Snapshot: emptyTestSnapshot("01AAA", "case_1"), Facts: facts}); err != nil {
		t.Fatal(err)
	}
	facts.Record.Summary = "mutated source"
	facts.Overview.Stages[0] = "mutated stage"
	facts.Overview.Settlements[0].Basis[0] = "mutated basis"
	facts.Horizon.Items[0].DependsOn[0] = "mutated dep"
	facts.Approvals[0].ApprovalID = "mutated approval"
	facts.Runs[0].RunID = "mutated run"

	first, err := s.At("01AAA")
	if err != nil {
		t.Fatal(err)
	}
	if first.Facts.Record.Summary != "captured old" || first.Facts.Overview.Stages[0] != "stage" ||
		first.Facts.Overview.Settlements[0].Basis[0] != "basis" || first.Facts.Horizon.Items[0].DependsOn[0] != "dep" ||
		first.Facts.Approvals[0].ApprovalID != "approval" || first.Facts.Runs[0].RunID != "run" {
		t.Fatalf("append must own every nested fact slice: %+v", first.Facts)
	}
	first.Facts.Record.Summary = "mutated returned read"
	first.Facts.Horizon.Items[0].DependsOn[0] = "mutated returned dependency"
	second, err := s.At("01AAA")
	if err != nil {
		t.Fatal(err)
	}
	if second.Facts.Record.Summary != "captured old" || second.Facts.Horizon.Items[0].DependsOn[0] != "dep" {
		t.Fatalf("read must not expose retained facts to mutation: %+v", second.Facts)
	}
}

func TestStoreSubscribeReplayThenLive(t *testing.T) {
	s := NewStore()
	s.Append(rev("01AAA", "case_1"))
	s.Append(rev("01BBB", "case_1"))

	ch, cancel := s.Subscribe("01AAA")
	defer cancel()
	select {
	case rev, ok := <-ch:
		if !ok || rev.Cursor != "01BBB" {
			t.Fatalf("replay must deliver the revision after the requested cursor, got %v", rev.Cursor)
		}
	default:
		t.Fatal("the replay must be buffered, not dropped")
	}
	// No more replays pending; a live append arrives on the same channel in cursor order.
	s.Append(rev("01CCC", "case_1"))
	select {
	case rev := <-ch:
		if rev.Cursor != "01CCC" {
			t.Fatalf("live revision = %q", rev.Cursor)
		}
	case <-time.After(time.Second):
		t.Fatal("live revision never delivered")
	}
	if s.SubscriberCount() != 1 {
		t.Fatalf("subscriber count = %d", s.SubscriberCount())
	}
	cancel()
	cancel() // safe to call more than once
	if s.SubscriberCount() != 0 {
		t.Fatalf("cancel must deregister the subscriber, count = %d", s.SubscriberCount())
	}
}
