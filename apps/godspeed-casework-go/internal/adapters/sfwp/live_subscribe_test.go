//go:build live

// Subscription resume across a server restart: subscribe, receive a mutation's frames, kill the
// server, restart it, and prove the client resubscribes from its last cursor with no gap and no
// duplicate for a mutation committed across the restart boundary.
//
// Honest bound (per the task): the bus retention makes exact-once across a restart impossible to
// promise in general; at-least-once with monotonic cursors is the accepted contract. The client
// deduplicates by cursor, so what the consumer observes is exactly-once delivery on this client.
package sfwp

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// commitCaseResolvingRestart commits one case, honouring the client's
// unresolved-outcome contract across a server restart: a typed unavailable
// recovery error ("the authority has no record of request ..." - the request
// never crossed admission) is retried exactly once, with a NEW request id,
// which is precisely what the contract tells the caller to do. Anything else
// is surfaced.
func commitCaseResolvingRestart(ctx context.Context, t *testing.T, cl *Client) (CommitView, error) {
	t.Helper()
	send := func() (CommitView, error) {
		req, err := NewCaseCommit(sentryChainRef, sentryChainParams(), policyRef, "sfwp_live_tests", 60,
			cl.NewRequestID("case_commit"), nil, operatorGov())
		if err != nil {
			return CommitView{}, err
		}
		resp, err := cl.Do(ctx, req)
		if err != nil {
			return CommitView{}, err
		}
		var view CommitView
		if err := resp.Into(&view); err != nil {
			return CommitView{}, err
		}
		return view, nil
	}
	view, err := send()
	if err == nil {
		return view, nil
	}
	var typed *apperr.Error
	if errors.As(err, &typed) && typed.Kind == apperr.KindUnavailable &&
		typed.Op == "recover" && strings.Contains(typed.Message, "no record of request") {
		t.Logf("commit outcome unresolved across the restart (%v); retrying with a new request id", err)
		return send()
	}
	return view, err
}

func TestLiveSubscriptionResumeAcrossRestart(t *testing.T) {
	cell := newLiveCell(t)
	cl := cell.client()

	ctx, cancel := context.WithTimeout(context.Background(), 120*time.Second)
	defer cancel()

	sub, err := cl.Subscribe(ctx)
	if err != nil {
		t.Fatal(err)
	}

	// Drain in the background, tracking cursors.
	type frame struct {
		cursor string
		kind   string
		caseID string
	}
	frames := make(chan frame, 1024)
	seen := map[string]bool{}
	dupes := 0
	go func() {
		defer close(frames)
		for ev := range sub.Events {
			if seen[ev.Cursor] {
				dupes++
				continue
			}
			seen[ev.Cursor] = true
			frames <- frame{cursor: ev.Cursor, kind: ev.Kind, caseID: ev.CaseID}
		}
	}()

	waitFor := func(what string, pred func(f frame) bool) frame {
		t.Helper()
		deadline := time.After(30 * time.Second)
		for {
			select {
			case f, ok := <-frames:
				if !ok {
					t.Fatalf("stream ended while waiting for %s", what)
				}
				if pred(f) {
					return f
				}
			case <-deadline:
				t.Fatalf("timed out waiting for %s", what)
			}
		}
	}

	// Mutation A: before the restart. Its case.submitted frame must arrive live.
	a := commitCase(ctx, t, cl, cl.NewRequestID("case_commit"))
	waitFor("case.submitted for case A", func(f frame) bool {
		return f.kind == "case.submitted" && f.caseID == a.CaseID
	})
	t.Logf("case A (%s) frames streaming; last cursor %s", a.CaseID, sub.LastCursor())

	// Kill and restart. The subscription connection dies with the server.
	cell.kill()
	t.Log("server killed under an active subscription; restarting")
	time.Sleep(200 * time.Millisecond)
	cell.start()

	// Mutation B: AFTER the restart, while the subscriber is reconnecting. Its frames must arrive
	// without a gap - either from the resubscribe replay (from_cursor = last delivered) or from
	// the live stream - and never twice.
	//
	// The client may hand the commit a pooled connection that died with the old server. Its
	// documented discipline then reports the outcome unresolved (the request never crossed the
	// authority's admission boundary) and leaves the retry to the caller - with a NEW request id.
	// That is exactly the recovery contract T06's coordinator implements, so the test follows it.
	b, err := commitCaseResolvingRestart(ctx, t, cl)
	if err != nil {
		t.Fatalf("case.commit after restart: %v", err)
	}
	waitFor("case.submitted for case B across the restart boundary", func(f frame) bool {
		return f.kind == "case.submitted" && f.caseID == b.CaseID
	})
	t.Logf("case B (%s) frames delivered across the restart boundary; last cursor %s", b.CaseID, sub.LastCursor())

	// A mutation on a case also publishes per-append case.trace frames (T04): driving one and
	// observing its trace frames proves the stream carries more than submit markers. NOTE the
	// real wire spelling: the server's trace_kind_snake helper lowercases the Debug format
	// WITHOUT snaking it, so ItemActivated publishes as "case.trace.itemactivated" (its doc
	// comment claims "item_activated" - a server-side doc/naming inconsistency recorded in the
	// T05 report). Both spellings are accepted here so the test tracks the contract, not a bug.
	advReqID := cl.NewRequestID("item_execute")
	execReq, err := NewItemExecute(a.CaseID, "task_prepare", policyRef, 60, advReqID, operatorGov())
	if err != nil {
		t.Fatal(err)
	}
	if _, err := cl.Do(ctx, execReq); err != nil {
		t.Fatalf("item.execute after restart: %v", err)
	}
	waitFor("case.trace item_activated frames for case A after the restart", func(f frame) bool {
		return (f.kind == "case.trace.itemactivated" || f.kind == "case.trace.item_activated") &&
			f.caseID == a.CaseID
	})

	cancel() // stop the subscription
	if dupes != 0 {
		t.Fatalf("the consumer observed %d duplicate cursors (dedupe failed)", dupes)
	}

	// Deterministic cross-check: events.get_range from the last delivered cursor returns only
	// newer frames, proving the cursor the client tracked is a real durable position.
	rangeCtx, rangeCancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer rangeCancel()
	after, err := cl.EventsGetRange(rangeCtx, sub.LastCursor(), "", 10)
	if err != nil {
		t.Fatalf("events.get_range: %v", err)
	}
	for _, ev := range after {
		if seen[ev.Cursor] {
			t.Fatalf("events.get_range after the last cursor returned an already-delivered frame %s", ev.Cursor)
		}
	}
	if len(after) == 0 && sub.LastCursor() != "" {
		// Not fatal: nothing new was committed after the last delivered frame. But we did commit
		// nothing after the trace frames, so expect exactly that.
		t.Log("no frames after the last delivered cursor (expected: none were committed)")
	}
}
