package auth

import (
	"sync"
	"testing"
	"time"
)

type observationTestClock struct {
	mu  sync.RWMutex
	now time.Time
}

func newObservationTestClock() *observationTestClock {
	return &observationTestClock{now: time.Date(2026, 10, 5, 12, 0, 0, 0, time.UTC)}
}

func (c *observationTestClock) Now() time.Time {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.now
}

func (c *observationTestClock) Set(now time.Time) {
	c.mu.Lock()
	c.now = now
	c.mu.Unlock()
}

func (c *observationTestClock) Advance(by time.Duration) {
	c.mu.Lock()
	c.now = c.now.Add(by)
	c.mu.Unlock()
}

func newObservationSessionStore(max int, idleTTL, absoluteTTL time.Duration) (*observationTestClock, *SessionStore) {
	clock := newObservationTestClock()
	return clock, NewSessionStore(idleTTL, absoluteTTL, max, clock.Now)
}

func observationIdentity() Identity {
	return Identity{ActorID: "operator_local", Role: "operator"}
}

func requireCurrentSession(t *testing.T, store *SessionStore, session *Session) CurrentSessionState {
	t.Helper()
	state, ok := store.Current(session.ID)
	if !ok {
		t.Fatal("Current must return the live session state")
	}
	if state.Revoked == nil {
		t.Fatal("Current must return the session revocation signal")
	}
	return state
}

func requireRevoked(t *testing.T, revoked <-chan struct{}) {
	t.Helper()
	select {
	case <-revoked:
	default:
		t.Fatal("removed session revocation signal must be closed")
	}
}

func requireNotRevoked(t *testing.T, revoked <-chan struct{}) {
	t.Helper()
	select {
	case <-revoked:
		t.Fatal("unrelated live session was revoked")
	default:
	}
}

func TestSessionStoreCurrentIsDetachedAndDoesNotSlide(t *testing.T) {
	clock, store := newObservationSessionStore(4, time.Minute, time.Hour)
	startedAt := clock.Now()
	session := store.Start(observationIdentity())
	wantDeadline := startedAt.Add(time.Minute)

	first := requireCurrentSession(t, store, session)
	if !first.ValidUntil.Equal(wantDeadline) {
		t.Fatal("Current returned the wrong idle deadline")
	}
	first.Claim.ActorID = "changed_actor"
	first.Claim.Role = "changed_role"
	clock.Set(startedAt.Add(30 * time.Second))
	second := requireCurrentSession(t, store, session)
	if second.Claim.ActorID != "operator_local" || second.Claim.Role != "operator" {
		t.Fatal("mutating the detached claim changed stored session identity")
	}
	if !second.ValidUntil.Equal(wantDeadline) || !session.LastSeen.Equal(startedAt) {
		t.Fatal("Current slid idle expiry or changed the session deadline")
	}
	if first.Revoked != second.Revoked {
		t.Fatal("Current did not return the stable session revocation signal")
	}
}

func TestSessionStoreCurrentUsesStrictIdleAndAbsoluteBoundaries(t *testing.T) {
	for _, tc := range []struct {
		name                 string
		idleTTL, absoluteTTL time.Duration
		deadline             time.Duration
	}{
		{name: "idle", idleTTL: time.Minute, absoluteTTL: time.Hour, deadline: time.Minute},
		{name: "absolute", idleTTL: time.Hour, absoluteTTL: time.Minute, deadline: time.Minute},
	} {
		t.Run(tc.name, func(t *testing.T) {
			clock, store := newObservationSessionStore(2, tc.idleTTL, tc.absoluteTTL)
			startedAt := clock.Now()
			session := store.Start(observationIdentity())
			state := requireCurrentSession(t, store, session)
			clock.Set(startedAt.Add(tc.deadline))
			atBoundary := requireCurrentSession(t, store, session)
			if atBoundary.Revoked != state.Revoked || !atBoundary.ValidUntil.Equal(startedAt.Add(tc.deadline)) {
				t.Fatal("session at the exact expiry boundary must remain valid")
			}
			clock.Set(startedAt.Add(tc.deadline + time.Nanosecond))
			if _, ok := store.Current(session.ID); ok {
				t.Fatal("session just after the strict expiry boundary must be absent")
			}
			requireRevoked(t, state.Revoked)
		})
	}
}

func TestSessionStoreResolveSlidesIdleButNotAbsoluteDeadline(t *testing.T) {
	clock, store := newObservationSessionStore(2, time.Minute, 2*time.Minute)
	startedAt := clock.Now()
	session := store.Start(observationIdentity())
	state := requireCurrentSession(t, store, session)

	clock.Set(startedAt.Add(50 * time.Second))
	if _, ok := store.Resolve(session.ID); !ok {
		t.Fatal("Resolve must preserve an active session")
	}
	afterIdleTouch := requireCurrentSession(t, store, session)
	if !afterIdleTouch.ValidUntil.Equal(startedAt.Add(110 * time.Second)) {
		t.Fatal("Resolve did not slide only the idle deadline")
	}

	clock.Set(startedAt.Add(100 * time.Second))
	if _, ok := store.Resolve(session.ID); !ok {
		t.Fatal("Resolve must preserve a session before its fixed absolute deadline")
	}
	beforeAbsolute := requireCurrentSession(t, store, session)
	absoluteDeadline := startedAt.Add(2 * time.Minute)
	if !beforeAbsolute.ValidUntil.Equal(absoluteDeadline) || !session.ExpiresAt.Equal(absoluteDeadline) {
		t.Fatal("Resolve extended the fixed absolute deadline")
	}
	if beforeAbsolute.Revoked != state.Revoked {
		t.Fatal("Resolve replaced the stable revocation signal")
	}
	clock.Set(absoluteDeadline)
	requireCurrentSession(t, store, session)
	clock.Set(absoluteDeadline.Add(time.Nanosecond))
	if _, ok := store.Resolve(session.ID); ok {
		t.Fatal("Resolve must expire the session just after the absolute deadline")
	}
	requireRevoked(t, state.Revoked)
}

func TestSessionStoreRemovalPathsCloseStableRevocationSignal(t *testing.T) {
	for _, tc := range []struct {
		name    string
		idleTTL time.Duration
		absTTL  time.Duration
		remove  func(*SessionStore, *observationTestClock, *Session)
	}{
		{
			name:    "destroy",
			idleTTL: time.Minute, absTTL: time.Hour,
			remove: func(store *SessionStore, _ *observationTestClock, session *Session) {
				store.Destroy(session.ID)
				store.Destroy(session.ID)
			},
		},
		{
			name:    "resolve idle expiry",
			idleTTL: time.Minute, absTTL: time.Hour,
			remove: func(store *SessionStore, clock *observationTestClock, session *Session) {
				clock.Advance(time.Minute + time.Nanosecond)
				store.Resolve(session.ID)
				store.Resolve(session.ID)
			},
		},
		{
			name:    "current idle expiry",
			idleTTL: time.Minute, absTTL: time.Hour,
			remove: func(store *SessionStore, clock *observationTestClock, session *Session) {
				clock.Advance(time.Minute + time.Nanosecond)
				store.Current(session.ID)
				store.Current(session.ID)
			},
		},
		{
			name:    "sweep absolute expiry",
			idleTTL: time.Hour, absTTL: time.Minute,
			remove: func(store *SessionStore, clock *observationTestClock, _ *Session) {
				clock.Advance(time.Minute + time.Nanosecond)
				if store.Sweep() != 1 || store.Sweep() != 0 {
					t.Fatal("Sweep must remove an expired session once")
				}
			},
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			clock, store := newObservationSessionStore(2, tc.idleTTL, tc.absTTL)
			session := store.Start(observationIdentity())
			state := requireCurrentSession(t, store, session)
			tc.remove(store, clock, session)
			requireRevoked(t, state.Revoked)
			if store.Len() != 0 {
				t.Fatal("removed session remains in the store")
			}
		})
	}
}

func TestSessionStoreCapacityEvictionIsBoundedAndDoesNotRevokeOthers(t *testing.T) {
	clock, store := newObservationSessionStore(2, time.Hour, 2*time.Hour)
	first := store.Start(observationIdentity())
	firstState := requireCurrentSession(t, store, first)
	clock.Advance(time.Second)
	second := store.Start(observationIdentity())
	secondState := requireCurrentSession(t, store, second)
	clock.Advance(time.Second)
	third := store.Start(observationIdentity())
	thirdState := requireCurrentSession(t, store, third)

	if store.Len() != 2 {
		t.Fatal("session store exceeded its configured capacity")
	}
	requireRevoked(t, firstState.Revoked)
	requireNotRevoked(t, secondState.Revoked)
	requireNotRevoked(t, thirdState.Revoked)
	store.Destroy(first.ID)
	store.Destroy(first.ID)
	requireNotRevoked(t, secondState.Revoked)
	requireNotRevoked(t, thirdState.Revoked)
	for i := 0; i < 8; i++ {
		clock.Advance(time.Second)
		store.Start(observationIdentity())
		if store.Len() > 2 {
			t.Fatal("session store exceeded its configured capacity")
		}
	}
}

func TestSessionStoreCurrentRacingDestroyNeverMissesRevocation(t *testing.T) {
	timer := time.NewTimer(5 * time.Second)
	defer timer.Stop()

	for i := 0; i < 16; i++ {
		_, store := newObservationSessionStore(2, time.Hour, 2*time.Hour)
		session := store.Start(observationIdentity())
		initial := requireCurrentSession(t, store, session)
		start := make(chan struct{})
		completed := make(chan struct{}, 2)
		var got CurrentSessionState
		var found bool
		go func() {
			<-start
			got, found = store.Current(session.ID)
			completed <- struct{}{}
		}()
		go func() {
			<-start
			store.Destroy(session.ID)
			completed <- struct{}{}
		}()
		close(start)
		for worker := 0; worker < 2; worker++ {
			select {
			case <-completed:
			case <-timer.C:
				t.Fatal("racing Current and Destroy workers did not complete within five seconds")
			}
		}
		if found {
			if got.Revoked != initial.Revoked {
				t.Fatal("racing Current returned a different revocation signal")
			}
			requireRevoked(t, got.Revoked)
		}
		requireRevoked(t, initial.Revoked)
		if _, ok := store.Current(session.ID); ok {
			t.Fatal("destroyed session remained observable")
		}
	}
}
