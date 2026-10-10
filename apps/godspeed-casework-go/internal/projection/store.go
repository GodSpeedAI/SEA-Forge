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
	"sort"
	"sync"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
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
	// Facts are the immutable authority view captured for this cursor. They are kept out of
	// the wire representation and allow a verified session to render the same historical
	// state in its own role perspective without refetching current authority state.
	Facts *CaseFacts `json:"-"`
}

// Store holds the world's revision history. All methods are safe for concurrent use. Stored
// revisions are IMMUTABLE once appended: the relay builds a fresh snapshot per revision, and the
// store deep-copies snapshots on append and when returning revisions to callers.
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

// Trajectory returns the retained revisions for one case in kernel-cursor order. The result is
// bounded by the store's global retention limit and owns a deep copy of every snapshot, so a
// caller cannot mutate retained history through returned slices or pointers. An empty result
// means the process has no retained revisions for that case; this can mean an unknown case, a
// cold store before relay replay, or that the case's revisions were evicted.
func (s *Store) Trajectory(caseID string) []Revision {
	s.mu.Lock()
	defer s.mu.Unlock()
	if caseID == "" || len(s.revisions) == 0 {
		return []Revision{}
	}
	out := make([]Revision, 0, len(s.revisions))
	for _, rev := range s.revisions {
		if rev.CaseID == caseID {
			out = append(out, cloneRevision(rev))
		}
	}
	return out
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
	evicted := false
	for len(s.revisions) > s.max {
		// Bounded retention: drop the oldest revision and its index entry. A client holding the
		// evicted cursor gets the documented 404 / resync_required, never a silently rewritten
		// history.
		delete(s.byCursor, s.revisions[0].Cursor)
		s.revisions = s.revisions[1:]
		evicted = true
	}
	if evicted {
		// Trimming shifts every survivor's slice index. Repair the cursor lookup once after all
		// evictions so At continues to resolve each retained revision to its own snapshot.
		for i, retained := range s.revisions {
			s.byCursor[retained.Cursor] = i
		}
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

// CountAfter reports how many retained revisions have a cursor strictly after `cursor` (the
// backlog an event-stream client positioned at `cursor` has not been sent). Used by metrics.
func (s *Store) CountAfter(cursor string) int {
	s.mu.Lock()
	defer s.mu.Unlock()
	i := sort.Search(len(s.revisions), func(i int) bool { return s.revisions[i].Cursor > cursor })
	return len(s.revisions) - i
}

// SubscriberCount reports the number of active subscriptions (tests and operational honesty).
func (s *Store) SubscriberCount() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return len(s.subs)
}

func cloneRevision(rev Revision) Revision {
	rev.Snapshot = cloneSnapshot(rev.Snapshot)
	rev.Facts = cloneFacts(rev.Facts)
	return rev
}

// CloneFacts returns an owned deep copy of the input authority view.
func CloneFacts(in *CaseFacts) *CaseFacts {
	return cloneFacts(in)
}

func cloneFacts(in *CaseFacts) *CaseFacts {
	if in == nil {
		return nil
	}
	out := *in
	out.Overview.Stages = append([]string(nil), in.Overview.Stages...)
	out.Overview.RunIDs = append([]ports.RunRef(nil), in.Overview.RunIDs...)
	if in.Overview.Settlements != nil {
		out.Overview.Settlements = make([]ports.SettlementNote, len(in.Overview.Settlements))
		for i, item := range in.Overview.Settlements {
			out.Overview.Settlements[i] = item
			out.Overview.Settlements[i].Basis = append([]string(nil), item.Basis...)
		}
	}
	if in.Horizon.Items != nil {
		out.Horizon.Items = make([]ports.HorizonItem, len(in.Horizon.Items))
		for i, item := range in.Horizon.Items {
			out.Horizon.Items[i] = item
			out.Horizon.Items[i].DependsOn = append([]string(nil), item.DependsOn...)
			out.Horizon.Items[i].RunIDs = append([]ports.RunRef(nil), item.RunIDs...)
		}
	}
	out.Approvals = append([]ports.ApprovalRecord(nil), in.Approvals...)
	out.Runs = append([]ports.RunSummary(nil), in.Runs...)
	out.UnreadableRunIDs = append([]string(nil), in.UnreadableRunIDs...)
	return &out
}

func cloneSnapshot(in contract.CognitiveWorldSnapshot) contract.CognitiveWorldSnapshot {
	out := in
	out.Perspective.DisplayName = cloneString(in.Perspective.DisplayName)
	out.Summary.ProgressPercent = cloneFloat(in.Summary.ProgressPercent)
	if in.VisibleObjects != nil {
		out.VisibleObjects = make([]contract.CognitiveObject, len(in.VisibleObjects))
		for i, obj := range in.VisibleObjects {
			cloned := obj
			cloned.Explanation = cloneString(obj.Explanation)
			cloned.ParentID = cloneString(obj.ParentID)
			if obj.DependsOn != nil {
				cloned.DependsOn = append([]string{}, obj.DependsOn...)
			}
			if obj.SpatialLayout != nil {
				layout := *obj.SpatialLayout
				layout.Radius = cloneFloat(obj.SpatialLayout.Radius)
				cloned.SpatialLayout = &layout
			}
			cloned.Actions = cloneActions(obj.Actions)
			out.VisibleObjects[i] = cloned
		}
	}
	out.AvailableActions = cloneActions(in.AvailableActions)
	if in.AttentionFocus.SalienceRank != nil {
		out.AttentionFocus.SalienceRank = append([]string{}, in.AttentionFocus.SalienceRank...)
	}
	out.AttentionFocus.Narration = cloneString(in.AttentionFocus.Narration)
	return out
}

func cloneActions(in []contract.ActionDescriptor) []contract.ActionDescriptor {
	if in == nil {
		return nil
	}
	out := make([]contract.ActionDescriptor, len(in))
	for i, action := range in {
		out[i] = action
		if action.Variant != nil {
			value := *action.Variant
			out[i].Variant = &value
		}
		if action.RequiresJustification != nil {
			value := *action.RequiresJustification
			out[i].RequiresJustification = &value
		}
	}
	return out
}

func cloneString(in *string) *string {
	if in == nil {
		return nil
	}
	out := *in
	return &out
}

func cloneFloat(in *float64) *float64 {
	if in == nil {
		return nil
	}
	out := *in
	return &out
}
