// The revision store behind the live /api/world history and the SSE replay.
//
// Revisions are keyed by the KERNEL's event cursor (the events-ledger entry_ulid the event bus
// hands out: opaque, monotonic in append order, durable, replay-addressable - see
// crates/sea-forge-server/src/sfwp/events.rs). The gateway never mints cursors of its own: a
// revision exists at the kernel cursor of the frame that produced it, so /api/world?cursor=...
// serves true kernel history, an SSE event's id IS a kernel cursor, and Last-Event-ID resume maps
// onto the same address space.
//
// Retention is bounded: the store keeps the most recent maxRevisions and evicts the oldest. A
// cursor evicted (or never stored) answers ErrUnknownCursor, which the server renders as the
// documented 404; the SSE side turns "last is older than the oldest retained" into the contract's
// resync_required event so the client refetches instead of assuming a gap-free replay.
package projection

import (
	"errors"
	"sync"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
)

// ErrUnknownCursor is returned by At for a cursor that is evicted or was never stored. The HTTP
// surface renders it as 404 with an explicit note; it is deliberately one error for both "never
// existed" and "evicted", because a client cannot act on the difference - it refetches the live
// world either way.
var ErrUnknownCursor = errors.New("projection: unknown or evicted kernel cursor")

// DefaultRetention is the store's default revision budget. Each revision holds a full snapshot,
// so the budget bounds gateway memory regardless of how busy the cell is (plan T11 owns
// shared-projection optimisation if the budget ever shows up in profiles).
const DefaultRetention = 512

// subscriberBuffer is the per-subscriber channel capacity for live revisions. A subscriber that
// falls this far behind is evicted (its channel closed) rather than allowed to stall the relay;
// it reconnects with Last-Event-ID and replays.
const subscriberBuffer = 256

// Revision is one position of a case's served history: the snapshot as it existed at the kernel
// cursor that produced it.
type Revision struct {
	Cursor   string                          `json:"cursor"` // kernel event cursor (entry_ulid)
	At       time.Time                       `json:"at"`     // when the gateway recorded it
	CaseID   string                          `json:"case_id"`
	Summary  string                          `json:"summary"` // the kernel frame kind that produced it
	Snapshot contract.CognitiveWorldSnapshot `json:"snapshot"`
}

// Store holds the world's revision history. All methods are safe for concurrent use. Stored
// revisions are IMMUTABLE once appended: the relay builds a fresh snapshot per revision and never
// mutates it afterwards, so handing out struct copies that share the snapshot's slices is safe.
// Callers must treat a received Revision as read-only.
type Store struct {
	mu        sync.Mutex
	max       int
	revisions []Revision // in kernel-cursor order, strictly advancing
	byCursor  map[string]int
	subs      map[int]chan Revision
	subSeq    int
}

// NewStore builds an empty store with the default retention budget.
func NewStore() *Store {
	return NewStoreWithRetention(DefaultRetention)
}

// NewStoreWithRetention builds an empty store with an explicit budget (tests use small ones).
func NewStoreWithRetention(max int) *Store {
	if max <= 0 {
		max = DefaultRetention
	}
	return &Store{max: max, byCursor: map[string]int{}, subs: map[int]chan Revision{}}
}

// Live returns the newest revision, or false when nothing has been recorded yet.
func (s *Store) Live() (Revision, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if len(s.revisions) == 0 {
		return Revision{}, false
	}
	return cloneRevision(s.revisions[len(s.revisions)-1]), true
}

// Head is the newest stored cursor ("" when the store is empty).
func (s *Store) Head() string {
	s.mu.Lock()
	defer s.mu.Unlock()
	if len(s.revisions) == 0 {
		return ""
	}
	return s.revisions[len(s.revisions)-1].Cursor
}

// Oldest is the oldest retained cursor ("" when the store is empty). A resume request older than
// this cannot be replayed gap-free: the caller sends resync_required.
func (s *Store) Oldest() string {
	s.mu.Lock()
	defer s.mu.Unlock()
	if len(s.revisions) == 0 {
		return ""
	}
	return s.revisions[0].Cursor
}

// At returns the revision recorded at a kernel cursor, or ErrUnknownCursor.
func (s *Store) At(cursor string) (Revision, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	idx, ok := s.byCursor[cursor]
	if !ok {
		return Revision{}, ErrUnknownCursor
	}
	return cloneRevision(s.revisions[idx]), nil
}

// Len reports the number of stored revisions (operational honesty and tests).
func (s *Store) Len() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return len(s.revisions)
}

// Append records a revision produced by a kernel frame. The cursor must strictly advance past
// the newest stored cursor (kernel cursors are monotonic; the relay deduplicates and orders, so
// a violation is a relay bug and is refused rather than silently reordering history). The append
// is broadcast to every current subscriber.
func (s *Store) Append(rev Revision) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if rev.Cursor == "" {
		return errors.New("projection: refusing a revision with an empty kernel cursor")
	}
	if n := len(s.revisions); n > 0 && rev.Cursor <= s.revisions[n-1].Cursor {
		return errors.New("projection: kernel cursor " + rev.Cursor + " does not advance past " + s.revisions[n-1].Cursor)
	}
	s.revisions = append(s.revisions, cloneRevision(rev))
	s.byCursor[rev.Cursor] = len(s.revisions) - 1
	for len(s.revisions) > s.max {
		// Bounded retention: drop the oldest revision and its index entry. A client holding the
		// evicted cursor gets the documented 404 / resync_required, never a silently rewritten
		// history.
		delete(s.byCursor, s.revisions[0].Cursor)
		s.revisions = s.revisions[1:]
	}
	for id, ch := range s.subs {
		select {
		case ch <- cloneRevision(rev):
		default:
			// A stalled subscriber must never stall the log: evict it. The closed channel ends
			// its stream; the client reconnects with Last-Event-ID and replays.
			close(ch)
			delete(s.subs, id)
		}
	}
	return nil
}

// Subscribe streams every stored revision with a cursor strictly after `after` (the replay), then
// every live revision as it is appended. The returned cancel removes the subscription and is safe
// to call more than once.
func (s *Store) Subscribe(after string) (<-chan Revision, func()) {
	s.mu.Lock()
	var replay []Revision
	for _, rev := range s.revisions {
		if rev.Cursor > after {
			replay = append(replay, cloneRevision(rev))
		}
	}
	// The replay is sent while holding the lock into a channel sized to hold it by construction,
	// so it cannot block and no live append can overtake the replay's cursor order.
	ch := make(chan Revision, len(replay)+subscriberBuffer)
	for _, rev := range replay {
		ch <- rev
	}
	s.subSeq++
	id := s.subSeq
	s.subs[id] = ch
	s.mu.Unlock()

	var once sync.Once
	cancel := func() {
		once.Do(func() {
			s.mu.Lock()
			if sub, ok := s.subs[id]; ok {
				delete(s.subs, id)
				close(sub)
			}
			s.mu.Unlock()
		})
	}
	return ch, cancel
}

// SubscriberCount reports the number of active subscriptions (tests and operational honesty).
func (s *Store) SubscriberCount() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return len(s.subs)
}

func cloneRevision(rev Revision) Revision {
	// Revisions are immutable once stored (see the Store contract); the struct copy is enough.
	return rev
}
