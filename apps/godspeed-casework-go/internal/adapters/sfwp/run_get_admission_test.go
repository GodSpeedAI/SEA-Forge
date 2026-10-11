package sfwp

import (
	"context"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// These contract fixtures intentionally exercise the proposed limiter API while
// its implementation is still a typed-unavailable stub. They are expected to
// fail assertions until a separately assigned implementation lands; they do
// not implement or simulate the limiter algorithm.
func TestRunGetLimiterContract(t *testing.T) {
	t.Run("production constructor fixes storage and cooldown", func(t *testing.T) {
		limiter := NewRunGetLimiter()
		if limiter.cooldown != time.Second {
			t.Fatalf("production cooldown=%s, want 1s", limiter.cooldown)
		}
		if got := len(limiter.slots); got != 2 {
			t.Fatalf("production records=%d, want exactly 2", got)
		}
	})

	t.Run("same run remains excluded while a permit is owned", func(t *testing.T) {
		limiter := newRunGetLimiter(time.Second, time.Now, nil)
		first, err := limiter.AcquireRunGet(boundedAdmissionContext(t), "run-a")
		if err != nil {
			t.Fatalf("first same-run admission: %v", err)
		}
		defer first.Release()
		waiter := newQueuedRunGetWaiter(t, limiter, "run-a")
		waiter.waitUntilParked(t, "same-run waiter did not reach bounded wait", "same run did not wait for its owned permit")
		first.Release()
		if err := waiter.join(t, "same-run waiter did not wake after release"); err != nil {
			t.Fatalf("same-run admission after release: %v", err)
		}
	})

	t.Run("matching run reuses its eligible slot without falling through", func(t *testing.T) {
		limiter := newRunGetLimiter(time.Second, time.Now, nil)
		limiter.slots[0] = runGetSlot{runID: "run-a", nextEligible: time.Time{}}
		limiter.slots[1] = runGetSlot{runID: "run-b", busy: true}
		permit, err := limiter.AcquireRunGet(boundedAdmissionContext(t), "run-a")
		if err != nil {
			t.Fatalf("eligible matching slot was not acquired: %v", err)
		}
		permit.Release()
		if limiter.slots[0].runID != "run-a" || limiter.slots[1].runID != "run-b" || !limiter.slots[1].busy {
			t.Fatalf("matching acquisition changed wrong slot: %+v", limiter.slots)
		}
	})

	t.Run("matching cooling slot does not fall through to another eligible slot", func(t *testing.T) {
		now := time.Unix(800, 0)
		limiter := newRunGetLimiter(time.Second, func() time.Time { return now }, nil)
		limiter.slots[0] = runGetSlot{runID: "run-a", nextEligible: now.Add(time.Minute)}
		limiter.slots[1] = runGetSlot{runID: "run-b"}
		waiter := newQueuedRunGetWaiter(t, limiter, "run-a")
		waiter.waitUntilParked(t, "matching-run waiter did not reach bounded wait", "matching run bypassed its cooling slot")
		slots := slotSnapshot(limiter)
		if slots[0].runID != "run-a" || slots[1].runID != "run-b" {
			t.Fatalf("matching cooldown wait changed slots: %+v", slots)
		}
		waiter.cancel()
		if err := waiter.join(t, "matching-run waiter did not observe cancellation"); err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
			t.Fatalf("queued cancellation error=%v", err)
		}
	})

	t.Run("unexpired idle and busy slots are not evicted", func(t *testing.T) {
		limiter := newRunGetLimiter(time.Second, time.Now, nil)
		future := time.Now().Add(time.Hour)
		limiter.slots[0] = runGetSlot{runID: "run-a", nextEligible: future}
		limiter.slots[1] = runGetSlot{runID: "run-b", busy: true, nextEligible: future}
		waiter := newQueuedRunGetWaiter(t, limiter, "run-c")
		waiter.waitUntilParked(t, "third-run waiter did not reach bounded wait", "third run returned instead of waiting for eligible record")
		slots := slotSnapshot(limiter)
		if slots[0].runID != "run-a" || slots[1].runID != "run-b" {
			t.Fatalf("blocked acquisition evicted state: %+v", slots)
		}
		waiter.cancel()
		if err := waiter.join(t, "third-run waiter did not observe cancellation"); err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
			t.Fatalf("queued cancellation error=%v", err)
		}
	})

	t.Run("different runs have at most two simultaneous owners", func(t *testing.T) {
		limiter := newRunGetLimiter(time.Second, time.Now, nil)
		first, err := limiter.AcquireRunGet(boundedAdmissionContext(t), "run-a")
		if err != nil {
			t.Fatalf("first distinct run: %v", err)
		}
		defer first.Release()
		second, err := limiter.AcquireRunGet(boundedAdmissionContext(t), "run-b")
		if err != nil {
			t.Fatalf("second distinct run: %v", err)
		}
		defer second.Release()
		waiter := newQueuedRunGetWaiter(t, limiter, "run-c")
		waiter.waitUntilParked(t, "third-run waiter did not reach bounded wait", "third run bypassed two owned records")
		first.Release()
		if err := waiter.join(t, "third run did not acquire after a slot became eligible"); err != nil {
			t.Fatalf("third run after one release: %v", err)
		}
	})

	t.Run("queued cancellation returns typed unavailable", func(t *testing.T) {
		limiter := newRunGetLimiter(time.Second, time.Now, nil)
		ctx, cancel := context.WithCancel(context.Background())
		cancel()
		_, err := limiter.AcquireRunGet(ctx, "run-a")
		if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
			t.Fatalf("canceled queued acquisition error=%v, want typed unavailable", err)
		}
	})

	t.Run("queued deadline returns typed unavailable", func(t *testing.T) {
		limiter := newRunGetLimiter(time.Second, time.Now, nil)
		limiter.slots[0] = runGetSlot{runID: "run-a", busy: true}
		deadline := newManualDeadlineContext()
		defer deadline.expire()
		waiter := newQueuedRunGetWaiterWithContext(t, limiter, "run-a", deadline, deadline.expire)
		waiter.waitUntilParked(t, "queued acquisition did not reach bounded deadline wait", "queued acquisition returned before deadline wait")
		deadline.expire()
		if err := waiter.join(t, "queued acquisition did not observe deadline"); err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
			t.Fatalf("deadline error=%v, want typed unavailable", err)
		}
	})

	t.Run("release before any write leaves cooldown clear", func(t *testing.T) {
		limiter := newRunGetLimiter(time.Second, time.Now, nil)
		permit, err := limiter.AcquireRunGet(boundedAdmissionContext(t), "run-a")
		if err != nil {
			t.Fatalf("prewrite admission: %v", err)
		}
		permit.Release()
		if !limiter.slots[0].nextEligible.IsZero() {
			t.Fatalf("prewrite release set cooldown to %s", limiter.slots[0].nextEligible)
		}
	})

	t.Run("cooldown begins at final attempted write completion", func(t *testing.T) {
		finish := time.Unix(500, 0)
		now := finish
		limiter := newRunGetLimiter(time.Second, func() time.Time { return now }, nil)
		permit, err := limiter.AcquireRunGet(boundedAdmissionContext(t), "run-a")
		if err != nil {
			t.Fatalf("acquire run_get: %v", err)
		}
		permit.WriteAttemptStarted()
		now = finish.Add(7 * time.Second) // controlled blocked/partial payload+LF completion
		permit.WriteAttemptFinished()
		permit.Release()
		if got, want := limiter.slots[0].nextEligible, now.Add(time.Second); !got.Equal(want) {
			t.Fatalf("cooldown eligibility=%s, want final write completion + 1s (%s)", got, want)
		}
	})

	t.Run("busy same-run wait ignores expired unrelated deadlines and wakes on permit release", func(t *testing.T) {
		// Both permits are acquired and finish writes under a controlled clock.
		// run-b is released and its cooldown expires before either waiter starts;
		// run-a remains owned past its cooldown. This proves the normal Release
		// contract and the no-timer busy-match wait without fabricating a signal.
		var timerCalls atomic.Int32
		clock := newObservedAdmissionClock(time.Unix(900, 0))
		limiter := newRunGetLimiter(time.Second, clock.Now, func(d time.Duration) runGetAdmissionTimer {
			timerCalls.Add(1)
			return inertRunGetTimer{ch: make(chan time.Time)}
		})
		permitA, err := limiter.AcquireRunGet(boundedAdmissionContext(t), "run-a")
		if err != nil {
			t.Fatalf("acquire run-a owner: %v", err)
		}
		permitA.WriteAttemptStarted()
		permitA.WriteAttemptFinished()
		permitB, err := limiter.AcquireRunGet(boundedAdmissionContext(t), "run-b")
		if err != nil {
			t.Fatalf("acquire run-b owner: %v", err)
		}
		permitB.WriteAttemptStarted()
		permitB.WriteAttemptFinished()
		permitB.Release()
		clock.Set(time.Unix(902, 0)) // both one-second cooldowns are now expired
		clock.resetNotifications()

		firstWaiter := newQueuedRunGetWaiter(t, limiter, "run-a")
		firstWaiter.waitUntilParked(t, "busy same-run waiter did not park", "busy same-run admission returned before cancellation or release")
		assertNoAdmissionReevaluation(t, clock, "canceled busy same-run wait")
		firstWaiter.cancel()
		if err := firstWaiter.join(t, "same-run waiter did not observe cancellation"); err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
			t.Fatalf("canceled no-spin waiter error=%v, want typed unavailable", err)
		}

		secondWaiter := newQueuedRunGetWaiter(t, limiter, "run-a")
		secondWaiter.waitUntilParked(t, "second same-run waiter did not park", "same-run waiter returned before permit release")
		assertNoAdmissionReevaluation(t, clock, "released busy same-run wait")
		permitA.Release()
		if err := secondWaiter.join(t, "same-run waiter did not acquire after real permit release"); err != nil {
			t.Fatalf("same-run acquisition after permit release: %v", err)
		}
		if got := timerCalls.Load(); got != 0 {
			t.Fatalf("same-run busy wait created %d timers over both waiter lifetimes", got)
		}
	})
}

type queuedRunGetWaiter struct {
	ctx      *observedDoneContext
	cancelFn context.CancelFunc
	result   chan error
	joined   bool
}

func newQueuedRunGetWaiter(t *testing.T, limiter *RunGetLimiter, runID string) *queuedRunGetWaiter {
	t.Helper()
	base, cancel := context.WithCancel(context.Background())
	return newQueuedRunGetWaiterWithContext(t, limiter, runID, base, cancel)
}

func newQueuedRunGetWaiterWithContext(t *testing.T, limiter *RunGetLimiter, runID string, base context.Context, cancel context.CancelFunc) *queuedRunGetWaiter {
	t.Helper()
	w := &queuedRunGetWaiter{
		ctx:      &observedDoneContext{Context: base, observed: make(chan struct{})},
		cancelFn: cancel,
		result:   make(chan error, 1),
	}
	go func() {
		permit, err := limiter.AcquireRunGet(w.ctx, runID)
		if permit != nil {
			permit.Release()
		}
		w.result <- err
	}()
	t.Cleanup(func() { w.cleanup(t) })
	return w
}

func slotSnapshot(limiter *RunGetLimiter) [2]runGetSlot {
	limiter.mu.Lock()
	defer limiter.mu.Unlock()
	return limiter.slots
}

func (w *queuedRunGetWaiter) waitUntilParked(t *testing.T, timeoutMessage, earlyMessage string) {
	t.Helper()
	select {
	case <-w.ctx.observed:
	case err := <-w.result:
		w.joined = true
		t.Fatalf("%s: %v", earlyMessage, err)
	case <-time.After(time.Second):
		t.Fatal(timeoutMessage)
	}
}

func (w *queuedRunGetWaiter) cancel() { w.cancelFn() }

func (w *queuedRunGetWaiter) join(t *testing.T, timeoutMessage string) error {
	t.Helper()
	select {
	case err := <-w.result:
		w.joined = true
		return err
	case <-time.After(time.Second):
		t.Fatal(timeoutMessage)
		return nil
	}
}

func (w *queuedRunGetWaiter) cleanup(t *testing.T) {
	w.cancelFn()
	if w.joined {
		return
	}
	select {
	case <-w.result:
		w.joined = true
	case <-time.After(time.Second):
		t.Errorf("queued run_get waiter did not join after cleanup cancellation")
	}
}

type observedAdmissionClock struct {
	nanos  atomic.Int64
	calls  atomic.Int64
	called chan struct{}
}

func newObservedAdmissionClock(now time.Time) *observedAdmissionClock {
	clock := &observedAdmissionClock{called: make(chan struct{}, 128)}
	clock.Set(now)
	return clock
}

func (c *observedAdmissionClock) Now() time.Time {
	c.calls.Add(1)
	select {
	case c.called <- struct{}{}:
	default:
	}
	return time.Unix(0, c.nanos.Load())
}

func (c *observedAdmissionClock) Set(now time.Time) { c.nanos.Store(now.UnixNano()) }

func (c *observedAdmissionClock) resetNotifications() {
	for {
		select {
		case <-c.called:
		default:
			return
		}
	}
}

func assertNoAdmissionReevaluation(t *testing.T, clock *observedAdmissionClock, label string) {
	t.Helper()
	// This bounded controlled-clock probe catches repeated re-evaluation while
	// the wait set has no event. It does not claim to prove the absence of every
	// possible CPU-spin implementation; source review must verify the wait path.
	clock.resetNotifications()
	baseline := clock.calls.Load()
	select {
	case <-clock.called:
		t.Fatalf("%s repeatedly re-evaluated the controlled clock", label)
	case <-time.After(20 * time.Millisecond):
		if got := clock.calls.Load(); got != baseline {
			t.Fatalf("%s clock calls advanced from %d to %d while no state changed", label, baseline, got)
		}
	}
}

func boundedAdmissionContext(t *testing.T) context.Context {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	t.Cleanup(cancel)
	return ctx
}

type inertRunGetTimer struct{ ch <-chan time.Time }

func (t inertRunGetTimer) C() <-chan time.Time { return t.ch }
func (inertRunGetTimer) Stop() bool            { return false }

type observedDoneContext struct {
	context.Context
	observed chan struct{}
	once     atomic.Bool
}

type manualDeadlineContext struct {
	done chan struct{}
	once sync.Once
}

func newManualDeadlineContext() *manualDeadlineContext {
	return &manualDeadlineContext{done: make(chan struct{})}
}

func (c *manualDeadlineContext) Deadline() (time.Time, bool) { return time.Now().Add(time.Hour), true }
func (c *manualDeadlineContext) Done() <-chan struct{}       { return c.done }
func (c *manualDeadlineContext) Err() error {
	select {
	case <-c.done:
		return context.DeadlineExceeded
	default:
		return nil
	}
}
func (*manualDeadlineContext) Value(any) any { return nil }
func (c *manualDeadlineContext) expire()     { c.once.Do(func() { close(c.done) }) }

func (c *observedDoneContext) Done() <-chan struct{} {
	if c.once.CompareAndSwap(false, true) {
		close(c.observed)
	}
	return c.Context.Done()
}
