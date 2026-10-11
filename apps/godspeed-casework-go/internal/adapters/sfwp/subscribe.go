// The dedicated long-lived events.subscribe connection.
//
// One connection is reserved for the subscription (it is the only connection on which the server
// pushes unsolicited `{"type":"event",...}` lines). On connect it replays strictly after the last
// cursor it delivered (the server replays the durable ledger before acking, and its live task then
// skips replayed cursors), so the stream the application sees is at-least-once with monotonic
// cursors - the accepted bound when the transport drops: exact-once across a restart is not
// promised by the bus, and the client additionally deduplicates by cursor so a redelivery is
// invisible. Delivery is bounded: when the consumer falls far enough behind that the channel is
// full, the connection is deliberately dropped and re-established from the last DELIVERED cursor,
// which re-replays the gap from the durable ledger (capped at 500 frames per burst server-side).
package sfwp

import (
	"bufio"
	"context"
	"errors"
	"sync"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// Subscription is a live event stream. Events is closed when ctx (the context passed to
// Subscribe) is done. LastCursor exposure lets a consumer persist its own resume point.
type Subscription struct {
	Events chan Event

	mu   sync.Mutex
	last string
	done chan struct{}
}

// Done is closed once the stream has stopped for good (the Subscribe context was cancelled).
func (s *Subscription) Done() <-chan struct{} { return s.done }

// LastCursor reports the newest cursor delivered on the stream.
func (s *Subscription) LastCursor() string {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.last
}

// Subscribe starts (and keeps restarting) the event stream in the background until ctx is done.
// The returned Subscription is usable immediately; the first frames may be a replay burst.
func (c *Client) Subscribe(ctx context.Context) (*Subscription, error) {
	bufSize := 256
	sub := &Subscription{
		Events: make(chan Event, bufSize),
		done:   make(chan struct{}),
	}
	go c.runSubscription(ctx, sub)
	return sub, nil
}

func (c *Client) runSubscription(ctx context.Context, sub *Subscription) {
	defer close(sub.done)
	defer close(sub.Events)

	backoff := c.cfg.BackoffBase
	for {
		if ctx.Err() != nil {
			return
		}
		dropped, err := c.subscriberPass(ctx, sub)
		if ctx.Err() != nil {
			return
		}
		if err != nil {
			c.logf("sfwp: subscription pass ended: %v", err)
		}
		if dropped {
			// The consumer fell behind and the channel overflowed: reconnect NOW (no backoff) so
			// the durable replay covers the dropped frames rather than losing wall-clock time too.
			backoff = c.cfg.BackoffBase
			continue
		}
		if err := sleepCtx(ctx, backoff); err != nil {
			return
		}
		backoff = min(backoff*2, c.cfg.BackoffMax)
	}
}

// subscriberPass dials one subscription connection, subscribes from the last delivered cursor, and
// pumps frames until the connection fails, the consumer overflows, or ctx is done. It reports
// whether it ended because delivery overflowed (immediate resubscribe) and the transport error
// otherwise.
func (c *Client) subscriberPass(ctx context.Context, sub *Subscription) (dropped bool, err error) {
	dialCtx, cancel := context.WithTimeout(ctx, c.cfg.ConnectTimeout)
	nc, err := c.cfg.Dial(dialCtx, c.cfg.SocketPath)
	cancel()
	if err != nil {
		return false, apperr.Wrap(apperr.KindUnavailable, "", "subscribe",
			"cannot reach the governed authority for the event stream", err)
	}
	defer nc.Close()

	from := sub.LastCursor()

	req := NewEventsSubscribe(from)
	line, err := EncodeRequest(req)
	if err != nil {
		return false, err
	}
	// Bound the subscribe handshake; the pump below uses a rolling read deadline instead, because
	// a healthy subscription is deliberately long-lived.
	if err := nc.SetWriteDeadline(time.Now().Add(c.cfg.ConnectTimeout)); err != nil {
		return false, apperr.Wrap(apperr.KindInternal, "", "subscribe", "cannot set write deadline", err)
	}
	if _, err := nc.Write(append(line, '\n')); err != nil {
		return false, transportErr("subscribe write", err)
	}
	if err := nc.SetWriteDeadline(time.Time{}); err != nil {
		return false, apperr.Wrap(apperr.KindInternal, "", "subscribe", "cannot clear write deadline", err)
	}

	// NOTE on ordering: the server replays the durable burst BEFORE writing the subscribe ack, so
	// event frames may precede the ack on this connection. Every line is therefore classified
	// independently - exactly the discriminator the reference client uses.
	br := bufio.NewReader(nc)
	for {
		// A rolling read deadline bounds each silent stretch: a stream that delivers nothing for
		// this long is treated as dead and re-established (the cursor resume makes that lossless).
		if err := nc.SetReadDeadline(time.Now().Add(c.cfg.SubscribeIdle)); err != nil {
			return false, apperr.Wrap(apperr.KindInternal, "", "subscribe", "cannot set read deadline", err)
		}
		raw, rerr := readBoundedLine(br, c.cfg.MaxResponseLineBytes, "subscribe")
		if rerr != nil {
			var typed *apperr.Error
			if errors.As(rerr, &typed) {
				return false, typed
			}
			return false, transportErr("subscribe read", rerr)
		}
		event, isEvent, derr := DecodeEvent(raw)
		if derr != nil {
			// An unparseable line on a subscribe connection is survivable: skip it. The durable
			// ledger remains the source of truth and events.get_range can re-read any window.
			c.logf("sfwp: skipping unparseable subscription line: %v", derr)
			continue
		}
		if isEvent {
			if event.Cursor <= sub.LastCursor() {
				// Already delivered: the replay window overlapped our last cursor (at-least-once
				// bound). Skip so the consumer never sees a duplicate or non-monotonic cursor.
				continue
			}
			select {
			case <-ctx.Done():
				return false, ctx.Err()
			case sub.Events <- *event:
				sub.setLast(event.Cursor)
			default:
				// Consumer too far behind: drop the connection and replay the gap from the ledger.
				c.logf("sfwp: subscription consumer overflowed at cursor %s; resubscribing from %s",
					event.Cursor, sub.LastCursor())
				return true, nil
			}
			continue
		}
		// Non-event line: the subscribe ack (or its error). It may legitimately arrive AFTER a
		// replay burst of event frames - the server replays the durable window before acking.
		resp, derr := DecodeResponse(raw)
		if derr != nil {
			return false, derr
		}
		if resp.Err != nil {
			if resp.Err.Class == "events_unknown_cursor" {
				// Our last cursor is unknown to the (replaced) ledger: the honest repair is to
				// restart the stream from the beginning rather than silently skip the gap.
				c.logf("sfwp: subscription cursor %s unknown to the authority; restarting from head", from)
				sub.setLast("")
				return true, nil
			}
			return false, resp.Err.appErr("events_subscribe")
		}
		var ack SubscribeAckView
		if err := resp.Into(&ack); err != nil {
			return false, err
		}
		if !ack.Subscribed {
			return false, apperr.New(apperr.KindInternal, "", "events_subscribe",
				"the authority acknowledged without subscribing")
		}
	}
}

func (s *Subscription) setLast(cursor string) {
	s.mu.Lock()
	s.last = cursor
	s.mu.Unlock()
}

// EventsGetRange is the deterministic bounded ledger read: the gap-recovery primitive for
// consumers that need an exact window between two cursors.
func (c *Client) EventsGetRange(ctx context.Context, fromCursor, toCursor string, limit int) ([]Event, error) {
	resp, err := c.Do(ctx, NewEventsGetRange(fromCursor, toCursor, limit))
	if err != nil {
		return nil, err
	}
	var view struct {
		Events []Event `json:"events"`
	}
	if err := resp.Into(&view); err != nil {
		return nil, err
	}
	return view.Events, nil
}
