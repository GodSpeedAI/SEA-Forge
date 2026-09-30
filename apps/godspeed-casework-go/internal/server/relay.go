// The kernel-frame relay: the live gateway's bridge between the T05 event subscription and the
// served surface. It consumes kernel event frames (case.submitted, case.trace.<kind> - there are
// no command-level frames on the bus; decision-log D-3-followups - so execution progress reaches
// the UI through the snapshot revisions, honestly, or not at all), and for every frame:
//
//   - advances the per-case view of the kernel event cursor (the staleness handle the intent
//     translator compares client_cursor against);
//   - rebuilds that case's CognitiveWorldSnapshot at the frame's cursor and appends it to the
//     revision store (bounded retention; the store broadcasts to SSE subscribers).
//
// Rebuild failure is never allowed to stall the relay: the cursor still advances (stale guards
// stay correct) and the revision is skipped (the store documents that history is best-effort; the
// live read path always refetches).
package server

import (
	"context"
	"log"
	"sync"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// KernelEvent is the relay's view of one kernel frame.
type KernelEvent struct {
	Cursor string // the kernel event cursor (entry_ulid)
	Kind   string // e.g. "case.submitted", "case.trace.item_activated"
	CaseID string
	At     time.Time
}

// EventFeed is the bounded, at-least-once, cursor-ordered kernel event stream (the T05
// subscription). Done is closed when the feed ends for good.
type EventFeed interface {
	Events() <-chan KernelEvent
	Done() <-chan struct{}
}

// RelayOptions tune the relay.
type RelayOptions struct {
	// DefaultActor is the relay's retained display perspective. When the source exposes captured
	// facts, authenticated reads re-render those facts for their own verified session actor.
	DefaultActor ports.ActorClaim
	// Logger receives relay diagnostics. Nil discards them.
	Logger *log.Logger
}

// Relay consumes the kernel feed and maintains the revision store and per-case cursors.
type Relay struct {
	feed   EventFeed
	source WorldSource
	store  *projection.Store
	opts   RelayOptions

	mu      sync.Mutex
	caseCur map[string]string
	waiters map[string]map[int]chan struct{}
	waitSeq int
	head    string
}

// NewRelay builds a relay over the feed, rebuilding via source into store.
func NewRelay(feed EventFeed, source WorldSource, store *projection.Store, opts RelayOptions) *Relay {
	return &Relay{
		feed:    feed,
		source:  source,
		store:   store,
		opts:    opts,
		caseCur: map[string]string{},
		waiters: map[string]map[int]chan struct{}{},
	}
}

// Run consumes the feed until ctx is done or the feed ends. It blocks; run it on its own
// goroutine for the process lifetime. The feed's channel is captured ONCE: the select must not
// re-invoke Events() per iteration (each call would spawn a fresh pump goroutine and strand
// in-flight frames in unread channels).
func (r *Relay) Run(ctx context.Context) {
	events := r.feed.Events()
	for {
		select {
		case <-ctx.Done():
			return
		case <-r.feed.Done():
			return
		case ev, ok := <-events:
			if !ok {
				return
			}
			r.accept(ctx, ev)
		}
	}
}

// accept folds one frame: cursor bookkeeping first (always, even if the rebuild fails), then the
// revision rebuild and append.
func (r *Relay) accept(ctx context.Context, ev KernelEvent) {
	r.mu.Lock()
	if ev.Cursor != "" && ev.Cursor > r.head {
		r.head = ev.Cursor
	}
	if ev.CaseID != "" && ev.Cursor != "" {
		if cur, ok := r.caseCur[ev.CaseID]; !ok || ev.Cursor > cur {
			r.caseCur[ev.CaseID] = ev.Cursor
			r.notifyCase(ev.CaseID)
		}
	}
	r.mu.Unlock()

	if ev.CaseID == "" {
		return // a frame with no case cannot be projected
	}
	var snap contract.CognitiveWorldSnapshot
	var facts *projection.CaseFacts
	var err error
	if source, ok := r.source.(interface {
		Facts(context.Context, string, ports.ActorClaim, string) (projection.CaseFacts, error)
	}); ok {
		captured, captureErr := source.Facts(ctx, ev.CaseID, r.opts.DefaultActor, ev.Cursor)
		err = captureErr
		if err == nil {
			facts = &captured
			snap = projection.Build(captured)
		}
	} else {
		snap, err = r.source.Snapshot(ctx, ev.CaseID, r.opts.DefaultActor, ev.Cursor)
	}
	if err != nil {
		// The kernel's frame is truth regardless: cursors advanced above, the revision store
		// simply has a gap here. Log honestly and keep serving.
		if r.opts.Logger != nil {
			r.opts.Logger.Printf("relay: rebuild of %s at %s failed: %v", ev.CaseID, ev.Cursor, err)
		}
		return
	}
	if err := r.store.Append(projection.Revision{
		Cursor:   ev.Cursor,
		At:       ev.At,
		CaseID:   ev.CaseID,
		Summary:  ev.Kind,
		Snapshot: snap,
		Facts:    facts,
	}); err != nil && r.opts.Logger != nil {
		r.opts.Logger.Printf("relay: append at %s refused: %v", ev.Cursor, err)
	}
}

// Head is the newest kernel cursor the relay has observed overall ("" before any frame).
func (r *Relay) Head() string {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.head
}

// CursorForCase implements intents.CursorSource.
func (r *Relay) CursorForCase(caseID string) (string, bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	cursor, ok := r.caseCur[caseID]
	return cursor, ok
}

// WaitForCaseAdvance implements intents.CursorSource: returns the case's cursor once it is set
// and strictly past `before` (an empty `before` waits for the first frame at all), or the newest
// value when ctx ends first.
func (r *Relay) WaitForCaseAdvance(ctx context.Context, caseID, before string) string {
	for {
		r.mu.Lock()
		cursor, ok := r.caseCur[caseID]
		if ok && (before == "" || cursor > before) {
			r.mu.Unlock()
			return cursor
		}
		r.waitSeq++
		id := r.waitSeq
		ch := make(chan struct{})
		if r.waiters[caseID] == nil {
			r.waiters[caseID] = map[int]chan struct{}{}
		}
		r.waiters[caseID][id] = ch
		r.mu.Unlock()

		select {
		case <-ch:
			continue // notifyCase removed us; loop to re-read
		case <-ctx.Done():
			r.mu.Lock()
			if ws := r.waiters[caseID]; ws != nil {
				delete(ws, id)
			}
			r.mu.Unlock()
			cursor, _ := r.CursorForCase(caseID)
			return cursor
		case <-r.feed.Done():
			r.mu.Lock()
			if ws := r.waiters[caseID]; ws != nil {
				delete(ws, id)
			}
			r.mu.Unlock()
			cursor, _ := r.CursorForCase(caseID)
			return cursor
		}
	}
}

func (r *Relay) notifyCase(caseID string) {
	ws := r.waiters[caseID]
	for id, ch := range ws {
		close(ch)
		delete(ws, id)
	}
}
