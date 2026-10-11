// LocalUserStore and SessionStore tests: the credential contract and the session lifetime
// contract (bounded TTL, absolute expiry, bounded store).
package auth

import (
	"errors"
	"testing"
	"time"
)

func testUsers(t *testing.T) []User {
	t.Helper()
	h, err := HashPassword("op-password")
	if err != nil {
		t.Fatal(err)
	}
	rh, err := HashPassword("rso-password")
	if err != nil {
		t.Fatal(err)
	}
	return []User{
		{Username: "operator", DisplayName: "Ops", PasswordHash: h, ActorID: "operator_local", Role: "operator"},
		{Username: "rso", PasswordHash: rh, ActorID: "rso_local", Role: "R-SO"},
	}
}

func newLocalStore(t *testing.T, mode string) *LocalUserStore {
	t.Helper()
	s, err := NewLocalUserStore(mode, testUsers(t))
	if err != nil {
		t.Fatal(err)
	}
	return s
}

func TestLocalLoginVerifiesPasswords(t *testing.T) {
	s := newLocalStore(t, ModeLocal)
	id, err := s.Login(nil, "operator", "op-password")
	if err != nil {
		t.Fatal(err)
	}
	if id.ActorID != "operator_local" || id.Role != "operator" || id.Username != "operator" {
		t.Fatalf("identity: %+v", id)
	}
	if id.Claim().ActorID != "operator_local" || id.Claim().Role != "operator" {
		t.Fatalf("claim: %+v", id.Claim())
	}
	if _, err := s.Login(nil, "operator", "wrong"); !errors.Is(err, ErrInvalidCredentials) {
		t.Fatalf("wrong password: %v", err)
	}
	if _, err := s.Login(nil, "nobody", "op-password"); !errors.Is(err, ErrInvalidCredentials) {
		t.Fatalf("unknown user: %v", err)
	}
	if _, err := s.Login(nil, "operator", ""); !errors.Is(err, ErrInvalidCredentials) {
		t.Fatalf("empty password: %v", err)
	}
	if _, err := s.Login(nil, "  OPERATOR ", "op-password"); err != nil {
		t.Fatalf("usernames are case/padding-insensitive: %v", err)
	}
}

func TestDevModeSkipsVerificationButNotIdentity(t *testing.T) {
	users := testUsers(t)
	for i := range users {
		users[i].PasswordHash = ""
	}
	s, err := NewLocalUserStore(ModeDev, users)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := s.Login(nil, "rso", "anything-at-all"); err != nil {
		t.Fatalf("dev mode must skip verification: %v", err)
	}
	if _, err := s.Login(nil, "rso", ""); !errors.Is(err, ErrInvalidCredentials) {
		t.Fatalf("dev mode must still refuse an empty password: %v", err)
	}
	id, err := s.StaticTokenIdentity("")
	if err != nil {
		t.Fatal(err)
	}
	if id.Username != "operator" {
		t.Fatalf("the static token acts as the first configured user, got %+v", id)
	}
	if id, err = s.StaticTokenIdentity("RSO"); err != nil || id.ActorID != "rso_local" {
		t.Fatalf("named static token user: %+v err=%v", id, err)
	}
}

func TestLocalStoreConstructionFailures(t *testing.T) {
	if _, err := NewLocalUserStore(ModeLocal, nil); err == nil {
		t.Fatal("an empty store must be refused")
	}
	users := testUsers(t)
	users[0].ActorID = ""
	if _, err := NewLocalUserStore(ModeLocal, users); err == nil {
		t.Fatal("a user without kernel standing must be refused")
	}
	// Local mode refuses an unparsable hash at construction (config fault, not a runtime surprise).
	users = testUsers(t)
	users[0].PasswordHash = "not-a-phc-string"
	if _, err := NewLocalUserStore(ModeLocal, users); err == nil {
		t.Fatal("an unparsable hash must be refused in local mode")
	}
	// Dev mode is the only other legal mode.
	if _, err := NewLocalUserStore("oidc", testUsers(t)); err == nil {
		t.Fatal("the local store must refuse foreign modes")
	}
}

// testClock is a mutable clock the session tests drive manually.
type testClock struct{ now time.Time }

func (c *testClock) Now() time.Time          { return c.now }
func (c *testClock) Advance(d time.Duration) { c.now = c.now.Add(d) }

func TestSessionStoreLifecycle(t *testing.T) {
	clk := &testClock{now: time.Date(2026, 9, 25, 12, 0, 0, 0, time.UTC)}
	st := NewSessionStore(time.Minute, time.Hour, 0, clk.Now)
	sess := st.Start(Identity{Username: "op", ActorID: "operator_local", Role: "operator"})
	if len(sess.ID) != 64 { // 256 bits as hex
		t.Fatalf("session id must be 256 bits of hex, got %d chars", len(sess.ID))
	}
	if len(sess.CSRFToken) != 64 {
		t.Fatalf("session csrf token must be 256 bits of hex, got %d chars", len(sess.CSRFToken))
	}
	got, ok := st.Resolve(sess.ID)
	if !ok || got.Identity.ActorID != "operator_local" {
		t.Fatalf("resolve: %+v ok=%v", got, ok)
	}
	st.Destroy(sess.ID)
	if _, ok := st.Resolve(sess.ID); ok {
		t.Fatal("a destroyed session must be absent")
	}
}

func TestSessionIdleExpirySlidesButAbsoluteDoesNot(t *testing.T) {
	clk := &testClock{now: time.Date(2026, 9, 25, 12, 0, 0, 0, time.UTC)}
	st := NewSessionStore(30*time.Minute, time.Hour, 0, clk.Now)
	sess := st.Start(Identity{Username: "op", ActorID: "a", Role: "r"})

	// 20 minutes in: still live, and the idle window slid forward.
	clk.Advance(20 * time.Minute)
	if _, ok := st.Resolve(sess.ID); !ok {
		t.Fatal("a 20-minute-old session with a 30-minute idle window must be live")
	}
	// 31 minutes later (51 since start, 31 since last touch): idle-expired.
	clk.Advance(31 * time.Minute)
	if _, ok := st.Resolve(sess.ID); ok {
		t.Fatal("a 25-minute idle gap must expire the session")
	}

	// Absolute expiry: activity every 10 minutes still dies at the 1-hour bound.
	sess = st.Start(Identity{Username: "op", ActorID: "a", Role: "r"})
	for elapsed := 10 * time.Minute; elapsed < 55*time.Minute; elapsed += 10 * time.Minute {
		clk.Advance(10 * time.Minute)
		if _, ok := st.Resolve(sess.ID); !ok {
			t.Fatalf("session died before its absolute bound (at +10m step %v)", elapsed)
		}
	}
	clk.Advance(15 * time.Minute)
	if _, ok := st.Resolve(sess.ID); ok {
		t.Fatal("the absolute expiry must hold despite continuous activity")
	}
}

func TestSessionStoreSweepsAndBounds(t *testing.T) {
	clk := &testClock{now: time.Date(2026, 9, 25, 12, 0, 0, 0, time.UTC)}
	st := NewSessionStore(time.Minute, time.Hour, 3, clk.Now)
	ids := []string{}
	for i := 0; i < 4; i++ {
		ids = append(ids, st.Start(Identity{Username: "u", ActorID: "a", Role: "r"}).ID)
		clk.Advance(time.Second)
	}
	if st.Len() != 3 {
		t.Fatalf("the store must evict the oldest beyond its bound, len=%d", st.Len())
	}
	if _, ok := st.Resolve(ids[0]); ok {
		t.Fatal("the oldest session must have been evicted")
	}
	clk.Advance(2 * time.Hour)
	swept := st.Sweep()
	if swept != 3 || st.Len() != 0 {
		t.Fatalf("sweep must clear expired sessions, swept=%d len=%d", swept, st.Len())
	}
}
