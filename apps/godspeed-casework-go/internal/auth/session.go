// Server-side sessions (T07). The store is in-memory and single-process BY DESIGN for this
// gateway: the plan's redesign_trigger documents that a stateless-replica deployment would move
// sessions (with leases and idempotency) to shared storage. Every session carries
//
//   - a 256-bit random id (the only thing the cookie holds; no user data rides in cookies),
//   - the authenticated Identity (the kernel actor + role the session acts as - fixed at login,
//     never client-settable),
//   - the CSRF synchronizer token (256-bit random, minted at session start),
//   - a sliding idle expiry and an absolute expiry, both bounded.
//
// The store is bounded (maxSessions; the oldest idle session is evicted when full) so an
// unbounded login flood cannot grow memory without limit.
package auth

import (
	"crypto/rand"
	"encoding/hex"
	"sync"
	"time"
)

// Session is one authenticated browser session.
type Session struct {
	ID        string   // 256-bit random hex; the cookie's only content
	Identity  Identity // fixed at login; the kernel standing this session acts as
	CSRFToken string   // 256-bit random hex; the synchronizer token every state-changing POST must present

	CreatedAt time.Time
	LastSeen  time.Time
	ExpiresAt time.Time // the FIXED absolute deadline (never extended; idle expiry rides on LastSeen)
}

// Idle reports whether the session's sliding idle window has passed.
func (s *Session) Idle(now time.Time, idle time.Duration) bool {
	return now.Sub(s.LastSeen) > idle
}

// SessionStore is the bounded in-memory session store. Safe for concurrent use.
type SessionStore struct {
	idleTTL     time.Duration
	absoluteTTL time.Duration
	maxSessions int
	now         func() time.Time

	mu       sync.Mutex
	sessions map[string]*Session
}

// Session lifetimes: documented defaults overridable from config (auth.session_idle_minutes,
// auth.session_absolute_hours).
const (
	DefaultIdleTTL     = 30 * time.Minute
	DefaultAbsoluteTTL = 12 * time.Hour
	// MaxSessions bounds the store (a single-operator gateway never approaches this; the bound
	// exists so a login flood cannot grow memory without limit).
	MaxSessions = 1024
)

// NewSessionStore builds the store. Non-positive TTLs or max take the documented defaults.
func NewSessionStore(idle, absolute time.Duration, max int, now func() time.Time) *SessionStore {
	if idle <= 0 {
		idle = DefaultIdleTTL
	}
	if absolute <= 0 {
		absolute = DefaultAbsoluteTTL
	}
	if max <= 0 {
		max = MaxSessions
	}
	if now == nil {
		now = time.Now
	}
	return &SessionStore{
		idleTTL:     idle,
		absoluteTTL: absolute,
		maxSessions: max,
		now:         now,
		sessions:    map[string]*Session{},
	}
}

// Start creates a session for the identity and returns it.
func (st *SessionStore) Start(identity Identity) *Session {
	id, err := randomToken()
	if err != nil {
		// A broken CSPRNG is not survivable for a session store: the caller's login fails.
		panic("auth: cannot mint session id: " + err.Error())
	}
	csrf, err := randomToken()
	if err != nil {
		panic("auth: cannot mint session csrf token: " + err.Error())
	}
	now := st.now()
	sess := &Session{
		ID:        id,
		Identity:  identity,
		CSRFToken: csrf,
		CreatedAt: now,
		LastSeen:  now,
		ExpiresAt: now.Add(st.absoluteTTL),
	}
	st.mu.Lock()
	defer st.mu.Unlock()
	if len(st.sessions) >= st.maxSessions {
		st.evictOldestLocked()
	}
	st.sessions[sess.ID] = sess
	return sess
}

// Resolve returns the live session for the given id, sliding its idle window (LastSeen). The
// absolute expiry is FIXED at creation and never extended; an expired session (idle or absolute)
// is destroyed and reported absent: expiry is enforced, never extended.
func (st *SessionStore) Resolve(id string) (*Session, bool) {
	if id == "" {
		return nil, false
	}
	st.mu.Lock()
	defer st.mu.Unlock()
	sess, ok := st.sessions[id]
	if !ok {
		return nil, false
	}
	now := st.now()
	if now.After(sess.ExpiresAt) || sess.Idle(now, st.idleTTL) {
		delete(st.sessions, id)
		return nil, false
	}
	sess.LastSeen = now
	return sess, true
}

// Destroy removes the session (logout).
func (st *SessionStore) Destroy(id string) {
	st.mu.Lock()
	defer st.mu.Unlock()
	delete(st.sessions, id)
}

// Len reports the number of live sessions (diagnostics).
func (st *SessionStore) Len() int {
	st.mu.Lock()
	defer st.mu.Unlock()
	return len(st.sessions)
}

// Sweep destroys expired sessions; the server runs it on an interval so idle sessions do not
// linger between touches. Returns the number swept.
func (st *SessionStore) Sweep() int {
	st.mu.Lock()
	defer st.mu.Unlock()
	now := st.now()
	swept := 0
	for id, sess := range st.sessions {
		if now.After(sess.ExpiresAt) || sess.Idle(now, st.idleTTL) {
			delete(st.sessions, id)
			swept++
		}
	}
	return swept
}

// evictOldestLocked drops the least-recently-seen session. Called with the lock held.
func (st *SessionStore) evictOldestLocked() {
	oldestID := ""
	var oldest time.Time
	for id, sess := range st.sessions {
		if oldestID == "" || sess.LastSeen.Before(oldest) {
			oldestID, oldest = id, sess.LastSeen
		}
	}
	if oldestID != "" {
		delete(st.sessions, oldestID)
	}
}

// randomToken returns 256 bits of CSPRNG output as hex.
func randomToken() (string, error) {
	var buf [32]byte
	if _, err := rand.Read(buf[:]); err != nil {
		return "", err
	}
	return hex.EncodeToString(buf[:]), nil
}
