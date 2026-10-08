// The EventFeed adapter over the T05 sfwp.Subscription: the relay consumes kernel frames through
// this interface so its logic stays testable without a kernel.
package server

import (
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/adapters/sfwp"
)

// sfwpFeed adapts sfwp.Subscription's Event channel to the relay's EventFeed.
type sfwpFeed struct {
	sub *sfwp.Subscription
}

// NewSFWPFeed wraps a live subscription as the relay's feed.
func NewSFWPFeed(sub *sfwp.Subscription) EventFeed {
	return &sfwpFeed{sub: sub}
}

// Events streams kernel frames. The cursor is the kernel's own event cursor (entry_ulid); kind is
// the frame's spelling ("case.submitted", "case.trace.<snake_kind>").
func (f *sfwpFeed) Events() <-chan KernelEvent {
	out := make(chan KernelEvent, 256)
	go func() {
		defer close(out)
		for ev := range f.sub.Events {
			at := time.Time{}
			if t, err := time.Parse(time.RFC3339Nano, ev.CommittedAt); err == nil {
				at = t
			} else {
				at = time.Now()
			}
			out <- KernelEvent{
				Cursor: ev.Cursor,
				Kind:   ev.Kind,
				CaseID: ev.CaseID,
				At:     at,
			}
		}
	}()
	return out
}

// Done closes when the subscription ends for good.
func (f *sfwpFeed) Done() <-chan struct{} { return f.sub.Done() }
