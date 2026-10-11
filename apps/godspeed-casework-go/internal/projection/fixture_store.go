//go:build casework_fixture

// FixtureStore: the in-memory revision log behind the projection API.
package projection

import (
	"errors"
	"fmt"
	"sync"
)

// ErrUnknownFixtureCursor is returned by At for a cursor no revision was recorded at.
var ErrUnknownFixtureCursor = errors.New("projection: unknown cursor")

// fixtureSubscriberBuffer is the per-subscriber channel capacity. It is deliberately larger than any
// replay the fixture can produce; a subscriber that still falls behind is evicted (its channel is
// closed) rather than allowed to stall appends for everyone. An evicted SSE client reconnects
// with ?last= and replays.
const fixtureSubscriberBuffer = 256

// FixtureStore holds the world's revision history and serves projections of it. All methods are safe for
// concurrent use; every Snapshot handed out is a deep copy, so callers may mutate freely without
// touching stored state.
type FixtureStore struct {
	mu        sync.Mutex
	revisions []FixtureRevision // ordered by strictly increasing cursor
	byCursor  map[int64]int
	subs      map[int]*storeSub
	subSeq    int
}

type storeSub struct {
	ch chan Snapshot
}

// NewFixtureStore builds a store from a dataset and validates its history: at least one revision,
// strictly increasing cursors, and a declared live cursor that matches the last revision.
func NewFixtureStore(ds Dataset) (*FixtureStore, error) {
	if len(ds.FixtureRevisions) == 0 {
		return nil, fmt.Errorf("projection: dataset has no revisions")
	}
	s := &FixtureStore{byCursor: map[int64]int{}, subs: map[int]*storeSub{}}
	for i, rev := range ds.FixtureRevisions {
		if i > 0 && rev.Cursor <= ds.FixtureRevisions[i-1].Cursor {
			return nil, fmt.Errorf("projection: dataset cursor %d does not strictly advance past %d", rev.Cursor, ds.FixtureRevisions[i-1].Cursor)
		}
		s.revisions = append(s.revisions, cloneFixtureRevision(rev))
		s.byCursor[rev.Cursor] = i
	}
	last := s.revisions[len(s.revisions)-1].Cursor
	if ds.LiveCursor != last {
		return nil, fmt.Errorf("projection: dataset liveCursor %d does not match last revision cursor %d", ds.LiveCursor, last)
	}
	return s, nil
}

// LiveCursor is the cursor of the latest revision.
func (s *FixtureStore) LiveCursor() int64 {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.revisions[len(s.revisions)-1].Cursor
}

// Live returns the latest revision as a labelled snapshot (deep copy).
func (s *FixtureStore) Live() Snapshot {
	s.mu.Lock()
	defer s.mu.Unlock()
	return SnapshotOf(s.revisions[len(s.revisions)-1])
}

// At returns the revision recorded at cursor as a labelled snapshot (deep copy), or
// ErrUnknownFixtureCursor when no revision was recorded there.
func (s *FixtureStore) At(cursor int64) (Snapshot, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	idx, ok := s.byCursor[cursor]
	if !ok {
		return Snapshot{}, fmt.Errorf("%w: %d", ErrUnknownFixtureCursor, cursor)
	}
	return SnapshotOf(s.revisions[idx]), nil
}

// Window returns every revision's position, oldest first.
func (s *FixtureStore) Window() []WindowEntry {
	s.mu.Lock()
	defer s.mu.Unlock()
	out := make([]WindowEntry, 0, len(s.revisions))
	for _, rev := range s.revisions {
		out = append(out, WindowEntry{Cursor: rev.Cursor, At: rev.At, Summary: rev.Summary})
	}
	return out
}

// Append records a new revision. The cursor must strictly advance past the live cursor; anything
// else is refused so the revision log stays strictly monotonic. The revision is deep-copied on
// entry, and every current subscriber is handed a labelled snapshot of it.
func (s *FixtureStore) Append(rev FixtureRevision) error {
	rev = cloneFixtureRevision(rev)
	s.mu.Lock()
	defer s.mu.Unlock()
	last := s.revisions[len(s.revisions)-1]
	if rev.Cursor <= last.Cursor {
		return fmt.Errorf("projection: append cursor %d does not strictly advance past live cursor %d", rev.Cursor, last.Cursor)
	}
	s.revisions = append(s.revisions, rev)
	s.byCursor[rev.Cursor] = len(s.revisions) - 1
	snapshot := SnapshotOf(rev)
	for id, sub := range s.subs {
		select {
		case sub.ch <- snapshot:
		default:
			// A stalled subscriber must never stall the log: evict it. Its channel close tells the
			// streaming side to end; the client reconnects and replays from ?last=.
			close(sub.ch)
			delete(s.subs, id)
		}
	}
	return nil
}

// Subscribe streams every revision with a cursor strictly after `after` (the replay), then every
// live revision as it is appended. The returned cancel removes the subscription and is safe to
// call more than once. FixtureRevisions arrive in cursor order.
func (s *FixtureStore) Subscribe(after int64) (<-chan Snapshot, func()) {
	s.mu.Lock()
	replay := make([]Snapshot, 0, len(s.revisions))
	for _, rev := range s.revisions {
		if rev.Cursor > after {
			replay = append(replay, SnapshotOf(rev))
		}
	}
	// The replay is sent while holding the lock, into a channel whose capacity covers the whole
	// replay by construction, so this cannot block and no live append can overtake the replay in
	// cursor order.
	ch := make(chan Snapshot, len(replay)+fixtureSubscriberBuffer)
	for _, snap := range replay {
		ch <- snap
	}
	s.subSeq++
	id := s.subSeq
	s.subs[id] = &storeSub{ch: ch}
	s.mu.Unlock()

	var once sync.Once
	cancel := func() {
		once.Do(func() {
			s.mu.Lock()
			if sub, ok := s.subs[id]; ok {
				delete(s.subs, id)
				close(sub.ch)
			}
			s.mu.Unlock()
		})
	}
	return ch, cancel
}

// SubscriberCount reports the number of active revision subscriptions. It exists for tests and
// operational honesty about goroutine cleanup, not for application logic.
func (s *FixtureStore) SubscriberCount() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return len(s.subs)
}
