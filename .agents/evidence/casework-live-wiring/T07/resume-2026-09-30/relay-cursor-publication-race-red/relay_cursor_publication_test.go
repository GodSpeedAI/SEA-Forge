package server

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// firstCaptureGate pauses the first relay projection after cursor observation. It lets these
// tests distinguish an observed kernel cursor (used for stale-intent checks) from a cursor whose
// revision has actually been retained and can be returned as a mutation receipt.
type firstCaptureGate struct {
	entered chan struct{}
	release chan struct{}
	err     error
	calls   int
}

func (s *firstCaptureGate) Facts(ctx context.Context, _ string, actor ports.ActorClaim, cursor string) (projection.CaseFacts, error) {
	s.calls++
	if s.calls == 1 {
		close(s.entered)
		select {
		case <-s.release:
		case <-ctx.Done():
			return projection.CaseFacts{}, ctx.Err()
		}
		if s.err != nil {
			return projection.CaseFacts{}, s.err
		}
	}
	facts := projection.CloneFacts(retainedFacts(cursor, "case_1", "completed"))
	facts.Actor = actor
	facts.Cursor = cursor
	return *facts, nil
}

func (*firstCaptureGate) Snapshot(context.Context, string, ports.ActorClaim, string) (contract.CognitiveWorldSnapshot, error) {
	return contract.CognitiveWorldSnapshot{}, errors.New("Facts path expected")
}

func (*firstCaptureGate) NewestCaseID(context.Context) (string, error) { return "case_1", nil }

func newPublicationRaceRelay(source *firstCaptureGate, store *projection.Store) *Relay {
	return NewRelay(newFakeFeed(), source, store, RelayOptions{
		DefaultActor: ports.ActorClaim{ActorID: "operator_local", Role: "operator"},
	})
}

func waitForRegisteredCaseWaiter(t *testing.T, relay *Relay, caseID string) {
	t.Helper()
	deadline := time.Now().Add(time.Second)
	for time.Now().Before(deadline) {
		relay.mu.Lock()
		registered := len(relay.waiters[caseID]) != 0
		relay.mu.Unlock()
		if registered {
			return
		}
		time.Sleep(time.Millisecond)
	}
	t.Fatalf("WaitForCaseAdvance waiter for %s was not registered", caseID)
}

func requireWaitStillBlocked(t *testing.T, result <-chan string) {
	t.Helper()
	select {
	case cursor := <-result:
		t.Fatalf("WaitForCaseAdvance returned observed cursor %q before retained publication", cursor)
	case <-time.After(25 * time.Millisecond):
	}
}

func TestWaitForCaseAdvanceReturnsOnlyAfterRevisionIsQueryable(t *testing.T) {
	source := &firstCaptureGate{entered: make(chan struct{}), release: make(chan struct{})}
	store := projection.NewStoreWithRetention(16)
	relay := newPublicationRaceRelay(source, store)
	waitResult := make(chan string, 1)
	go func() { waitResult <- relay.WaitForCaseAdvance(context.Background(), "case_1", "01AAA") }()
	waitForRegisteredCaseWaiter(t, relay, "case_1")

	acceptDone := make(chan struct{})
	go func() {
		relay.accept(context.Background(), KernelEvent{Cursor: "01BBB", CaseID: "case_1", Kind: "case.trace.item_completed", At: time.Now()})
		close(acceptDone)
	}()
	<-source.entered
	if got, ok := relay.CursorForCase("case_1"); !ok || got != "01BBB" {
		t.Fatalf("observed cursor must advance while capture is pending, got %q (%v)", got, ok)
	}
	if got := relay.Head(); got != "01BBB" {
		t.Fatalf("kernel head must reflect the observed frame during capture, got %q", got)
	}
	requireWaitStillBlocked(t, waitResult)

	close(source.release)
	<-acceptDone
	select {
	case cursor := <-waitResult:
		if cursor != "01BBB" {
			t.Fatalf("mutation receipt cursor = %q, want 01BBB", cursor)
		}
	case <-time.After(time.Second):
		t.Fatal("WaitForCaseAdvance did not return after the revision was appended")
	}
	rev, err := store.At("01BBB")
	if err != nil || rev.CaseID != "case_1" {
		t.Fatalf("returned mutation receipt must already be queryable at Store.At: revision=%+v err=%v", rev, err)
	}
	trajectory := store.Trajectory("case_1")
	if len(trajectory) != 1 || trajectory[0].Cursor != "01BBB" {
		t.Fatalf("returned mutation receipt must already be queryable in the case trajectory: %+v", trajectory)
	}
}

func TestWaitForCaseAdvanceSkipsCaptureAndAppendFailures(t *testing.T) {
	tests := []struct {
		name        string
		captureErr  error
		refuseStore bool
	}{
		{name: "capture error", captureErr: errors.New("capture unavailable")},
		{name: "store append refusal", refuseStore: true},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			source := &firstCaptureGate{entered: make(chan struct{}), release: make(chan struct{}), err: tc.captureErr}
			store := projection.NewStoreWithRetention(16)
			if tc.refuseStore {
				if err := store.Append(projection.Revision{Cursor: "01BBB", CaseID: "case_other"}); err != nil {
					t.Fatal(err)
				}
			}
			relay := newPublicationRaceRelay(source, store)
			waitResult := make(chan string, 1)
			go func() { waitResult <- relay.WaitForCaseAdvance(context.Background(), "case_1", "01AAA") }()
			waitForRegisteredCaseWaiter(t, relay, "case_1")

			acceptDone := make(chan struct{})
			go func() {
				relay.accept(context.Background(), KernelEvent{Cursor: "01BBB", CaseID: "case_1", Kind: "case.trace.item_completed", At: time.Now()})
				close(acceptDone)
			}()
			<-source.entered
			if got, ok := relay.CursorForCase("case_1"); !ok || got != "01BBB" {
				t.Fatalf("observed cursor must advance despite later projection failure, got %q (%v)", got, ok)
			}
			if got := relay.Head(); got != "01BBB" {
				t.Fatalf("kernel head must preserve observed frame despite projection failure, got %q", got)
			}
			close(source.release)
			<-acceptDone
			requireWaitStillBlocked(t, waitResult)
			if got := store.Trajectory("case_1"); len(got) != 0 {
				t.Fatalf("failed cursor must not appear as a retained case revision: %+v", got)
			}

			relay.accept(context.Background(), KernelEvent{Cursor: "01CCC", CaseID: "case_1", Kind: "case.trace.item_completed", At: time.Now()})
			select {
			case cursor := <-waitResult:
				if cursor != "01CCC" {
					t.Fatalf("WaitForCaseAdvance returned %q; want the next successfully retained cursor 01CCC", cursor)
				}
			case <-time.After(time.Second):
				t.Fatal("WaitForCaseAdvance did not return after a later revision was retained")
			}
			if rev, err := store.At("01CCC"); err != nil || rev.CaseID != "case_1" {
				t.Fatalf("successful cursor must be queryable: revision=%+v err=%v", rev, err)
			}
		})
	}
}
