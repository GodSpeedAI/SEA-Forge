package projection

import (
	"errors"
	"testing"
)

func TestStoreRetentionOneKeepsLatestAtAndEvictsPrevious(t *testing.T) {
	s := NewStoreWithRetention(1)
	for _, cursor := range []string{"01AAA", "01BBB"} {
		if err := s.Append(rev(cursor, "case_1")); err != nil {
			t.Fatal(err)
		}
	}

	if s.Len() != 1 || s.Oldest() != "01BBB" || s.Head() != "01BBB" {
		t.Fatalf("retention one must keep only the latest revision: len=%d oldest=%q head=%q", s.Len(), s.Oldest(), s.Head())
	}
	if _, err := s.At("01AAA"); !errors.Is(err, ErrUnknownCursor) {
		t.Fatalf("evicted cursor must return ErrUnknownCursor, got %v", err)
	}
	assertRetainedCursor(t, s, "01BBB")
	assertStoreCursorIndex(t, s, []string{"01BBB"})
}

func TestStoreRetentionTwoKeepsEveryRetainedIndexAndReplaysInOrder(t *testing.T) {
	s := NewStoreWithRetention(2)
	cursors := []string{"01AAA", "01BBB", "01CCC", "01DDD", "01EEE"}
	for i, cursor := range cursors {
		if err := s.Append(rev(cursor, "case_1")); err != nil {
			t.Fatal(err)
		}

		firstRetained := i - 1
		if firstRetained < 0 {
			firstRetained = 0
		}
		for j, candidate := range cursors[:i+1] {
			if j < firstRetained {
				if _, err := s.At(candidate); !errors.Is(err, ErrUnknownCursor) {
					t.Fatalf("after appending %s, evicted cursor %s must return ErrUnknownCursor, got %v", cursor, candidate, err)
				}
				continue
			}
			assertRetainedCursor(t, s, candidate)
		}
		assertStoreCursorIndex(t, s, cursors[firstRetained:i+1])
	}

	// Resume from an evicted cursor returns only retained revisions newer than it, in cursor
	// order. The HTTP layer is responsible for its separate resync_required control event.
	ch, cancel := s.Subscribe("01BBB")
	defer cancel()
	for _, want := range []string{"01DDD", "01EEE"} {
		select {
		case got, ok := <-ch:
			if !ok || got.Cursor != want || got.Snapshot.Cursor != want {
				t.Fatalf("replay from evicted cursor returned %+v (open=%t), want %s", got, ok, want)
			}
		default:
			t.Fatalf("replay from evicted cursor omitted retained cursor %s", want)
		}
	}
	select {
	case got := <-ch:
		t.Fatalf("replay returned an extra revision after the retained head: %+v", got)
	default:
	}
}

func assertRetainedCursor(t *testing.T, s *Store, cursor string) {
	t.Helper()
	got, err := s.At(cursor)
	if err != nil {
		t.Fatalf("At(%s) for retained revision: %v", cursor, err)
	}
	if got.Cursor != cursor || got.Snapshot.Cursor != cursor || got.CaseID != "case_1" || got.Snapshot.CaseID != "case_1" {
		t.Fatalf("At(%s) returned a different retained revision: %+v", cursor, got)
	}
}

func assertStoreCursorIndex(t *testing.T, s *Store, cursors []string) {
	t.Helper()
	s.mu.Lock()
	defer s.mu.Unlock()
	if len(s.revisions) != len(cursors) || len(s.byCursor) != len(cursors) {
		t.Fatalf("retention indexes are not bounded to %d entries: revisions=%d index=%d", len(cursors), len(s.revisions), len(s.byCursor))
	}
	for i, cursor := range cursors {
		if s.revisions[i].Cursor != cursor {
			t.Fatalf("retained revision[%d] = %q, want %q", i, s.revisions[i].Cursor, cursor)
		}
		if index, ok := s.byCursor[cursor]; !ok || index != i {
			t.Fatalf("cursor index for %s = %d (present=%t), want %d", cursor, index, ok, i)
		}
	}
}
