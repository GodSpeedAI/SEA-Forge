package projection

import (
	"errors"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
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
