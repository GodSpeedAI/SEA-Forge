package sfwp

import (
	"context"
	"sync"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

const runGetAdmissionCooldown = time.Second

// RunGetAdmission acquires one permit for one physical run_get attempt.
// A logical Client.Do retry must acquire another permit.
type RunGetAdmission interface {
	AcquireRunGet(ctx context.Context, runID string) (RunGetPermit, error)
}

// RunGetPermit records the physical write interval and remains owned through
// connection cleanup. Release is expected to be idempotent.
type RunGetPermit interface {
	WriteAttemptStarted()
	WriteAttemptFinished()
	Release()
}

// RunGetLimiter is a bounded process-shared owner for physical run_get attempts.
type RunGetLimiter struct {
	mu       sync.Mutex
	slots    [2]runGetSlot
	cooldown time.Duration
	now      func() time.Time
	newTimer func(time.Duration) runGetAdmissionTimer
	changed  chan struct{}
}

type runGetSlot struct {
	runID        string
	busy         bool
	nextEligible time.Time
}

// runGetAdmissionTimer and its factory are package-private seams for controlled
// deadline-wait tests.
type runGetAdmissionTimer interface {
	C() <-chan time.Time
	Stop() bool
}

// NewRunGetLimiter fixes production storage at two records and the cooldown at
// one second. There is deliberately no production timing override.
func NewRunGetLimiter() *RunGetLimiter {
	return newRunGetLimiter(runGetAdmissionCooldown, time.Now, func(d time.Duration) runGetAdmissionTimer {
		return realRunGetAdmissionTimer{timer: time.NewTimer(d)}
	})
}

func newRunGetLimiter(cooldown time.Duration, now func() time.Time, timer func(time.Duration) runGetAdmissionTimer) *RunGetLimiter {
	if now == nil {
		now = time.Now
	}
	if timer == nil {
		timer = func(d time.Duration) runGetAdmissionTimer {
			return realRunGetAdmissionTimer{timer: time.NewTimer(d)}
		}
	}
	return &RunGetLimiter{
		cooldown: cooldown,
		now:      now,
		newTimer: timer,
		changed:  make(chan struct{}),
	}
}

type realRunGetAdmissionTimer struct{ timer *time.Timer }

func (t realRunGetAdmissionTimer) C() <-chan time.Time { return t.timer.C }
func (t realRunGetAdmissionTimer) Stop() bool          { return t.timer.Stop() }

func (l *RunGetLimiter) AcquireRunGet(ctx context.Context, runID string) (RunGetPermit, error) {
	for {
		if err := ctx.Err(); err != nil {
			return nil, runGetAdmissionContextError(err)
		}

		l.mu.Lock()
		if err := ctx.Err(); err != nil {
			l.mu.Unlock()
			return nil, runGetAdmissionContextError(err)
		}

		now := l.now()
		matching := -1
		for i := range l.slots {
			if l.slots[i].runID == runID && !runGetSlotEmpty(l.slots[i]) {
				matching = i
				break
			}
		}

		if matching >= 0 {
			slot := &l.slots[matching]
			if !slot.busy && !now.Before(slot.nextEligible) {
				permit := l.acquireSlotLocked(matching, runID)
				l.mu.Unlock()
				return permit, nil
			}
			changed := l.changed
			var deadline time.Time
			if !slot.busy {
				deadline = slot.nextEligible
			}
			l.mu.Unlock()
			l.wait(ctx, changed, deadline)
			continue
		}

		selected := -1
		for i := range l.slots {
			if runGetSlotEmpty(l.slots[i]) {
				selected = i
				break
			}
		}
		if selected < 0 {
			for i := range l.slots {
				slot := &l.slots[i]
				if slot.busy || now.Before(slot.nextEligible) {
					continue
				}
				if selected < 0 || slot.nextEligible.Before(l.slots[selected].nextEligible) {
					selected = i
				}
			}
		}
		if selected >= 0 {
			permit := l.acquireSlotLocked(selected, runID)
			l.mu.Unlock()
			return permit, nil
		}

		changed := l.changed
		var deadline time.Time
		for i := range l.slots {
			slot := &l.slots[i]
			if slot.busy || slot.nextEligible.IsZero() || !now.Before(slot.nextEligible) {
				continue
			}
			if deadline.IsZero() || slot.nextEligible.Before(deadline) {
				deadline = slot.nextEligible
			}
		}
		l.mu.Unlock()
		l.wait(ctx, changed, deadline)
	}
}

func runGetSlotEmpty(slot runGetSlot) bool {
	return slot.runID == "" && !slot.busy && slot.nextEligible.IsZero()
}

func (l *RunGetLimiter) acquireSlotLocked(index int, runID string) RunGetPermit {
	l.slots[index] = runGetSlot{runID: runID, busy: true}
	return &runGetPermit{limiter: l, slot: index}
}

func (l *RunGetLimiter) wait(ctx context.Context, changed <-chan struct{}, deadline time.Time) {
	if deadline.IsZero() {
		select {
		case <-ctx.Done():
		case <-changed:
		}
		return
	}
	d := deadline.Sub(l.now())
	if d <= 0 {
		return
	}
	timer := l.newTimer(d)
	defer timer.Stop()
	select {
	case <-ctx.Done():
	case <-changed:
	case <-timer.C():
	}
}

func (l *RunGetLimiter) signalLocked() {
	close(l.changed)
	l.changed = make(chan struct{})
}

type runGetPermit struct {
	limiter  *RunGetLimiter
	slot     int
	started  bool
	finished bool
	released bool
}

func (p *runGetPermit) WriteAttemptStarted() {
	l := p.limiter
	l.mu.Lock()
	defer l.mu.Unlock()
	if !p.released && !p.started {
		p.started = true
	}
}

func (p *runGetPermit) WriteAttemptFinished() {
	l := p.limiter
	l.mu.Lock()
	defer l.mu.Unlock()
	if !p.released && p.started && !p.finished {
		l.slots[p.slot].nextEligible = l.now().Add(l.cooldown)
		p.finished = true
		l.signalLocked()
	}
}

func (p *runGetPermit) Release() {
	l := p.limiter
	l.mu.Lock()
	defer l.mu.Unlock()
	if p.released {
		return
	}
	if p.started && !p.finished {
		l.slots[p.slot].nextEligible = l.now().Add(l.cooldown)
		p.finished = true
	}
	l.slots[p.slot].busy = false
	p.released = true
	l.signalLocked()
}

func runGetAdmissionContextError(err error) error {
	return apperr.Wrap(apperr.KindUnavailable, "", "run_get_admission",
		"run_get admission was canceled before a slot became available", err)
}
