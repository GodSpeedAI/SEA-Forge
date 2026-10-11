// T11 security review regression tests: each names the attack it pins and fails on the
// pre-review code.
package server

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/http/cookiejar"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/auth"
)

func loginAttempt(t *testing.T, base, username, password string) int {
	t.Helper()
	jar, _ := cookiejar.New(nil)
	u := &testUser{t: t, base: base, client: &http.Client{Jar: jar}}
	resp, err := u.client.Get(base + "/api/session")
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	req, _ := http.NewRequest(http.MethodPost, base+"/api/auth/login",
		strings.NewReader(fmt.Sprintf(`{"username":%q,"password":%q}`, username, password)))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set(csrfHeader, u.cookieValue(csrfCookieName))
	resp, err = u.client.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	io.Copy(io.Discard, resp.Body)
	return resp.StatusCode
}

// ATTACK: online password guessing. Failed logins for one username are throttled; after the
// budget is spent even the CORRECT password is refused with 429 until the bucket refills.
func TestT11LoginBruteForceIsThrottled(t *testing.T) {
	h := newLiveHarness(t)
	hash, err := auth.HashPassword("correct horse")
	if err != nil {
		t.Fatal(err)
	}
	store, err := auth.NewLocalUserStore(auth.ModeLocal, []auth.User{
		{Username: "operator", PasswordHash: hash, ActorID: "operator_local", Role: "operator"},
	})
	if err != nil {
		t.Fatal(err)
	}
	h.api.opts.Auth = AuthOptions{Authenticator: store, Sessions: auth.NewSessionStore(0, 0, 0, nil)}

	if got := loginAttempt(t, h.ts.URL, "operator", "correct horse"); got != http.StatusOK {
		t.Fatalf("a correct login before any failure must succeed, got %d", got)
	}
	for i := 0; i < 10; i++ {
		if got := loginAttempt(t, h.ts.URL, "operator", fmt.Sprintf("wrong-%d", i)); got != http.StatusUnauthorized {
			t.Fatalf("attempt %d: want 401, got %d", i, got)
		}
	}
	if got := loginAttempt(t, h.ts.URL, "operator", "correct horse"); got != http.StatusTooManyRequests {
		t.Fatalf("after the failure budget is spent the login must be 429, got %d", got)
	}
}

// ATTACK: oversized credentials (argon2id over a megabyte password, memory pressure).
func TestT11LoginRefusesOversizedCredentials(t *testing.T) {
	h := newLiveHarness(t)
	if got := loginAttempt(t, h.ts.URL, "operator", strings.Repeat("x", maxLoginFieldBytes+1)); got != http.StatusBadRequest {
		t.Fatalf("oversized password must be 400, got %d", got)
	}
}

// ATTACK: SSE exhaustion. One session cannot hold unbounded /api/events streams.
func TestT11SSEPerSessionCap(t *testing.T) {
	h := newLiveHarness(t)
	op := h.login(t, "operator", "x")
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	open := func() (*http.Response, error) {
		req, _ := http.NewRequestWithContext(ctx, http.MethodGet, op.base+"/api/events", nil)
		return op.client.Do(req)
	}
	var resps []*http.Response
	for i := 0; i < maxSSEPerSession; i++ {
		resp, err := open()
		if err != nil || resp.StatusCode != http.StatusOK {
			t.Fatalf("stream %d should open: %v %v", i, err, resp)
		}
		resps = append(resps, resp)
	}
	resp, err := open()
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusTooManyRequests {
		t.Fatalf("stream %d must be refused with 429, got %d", maxSSEPerSession+1, resp.StatusCode)
	}
	// Closing one frees a slot.
	resps[0].Body.Close()
	deadline := time.Now().Add(3 * time.Second)
	for {
		r2, err := open()
		if err == nil && r2.StatusCode == http.StatusOK {
			r2.Body.Close()
			break
		}
		if err == nil {
			r2.Body.Close()
		}
		if time.Now().After(deadline) {
			t.Fatal("a closed stream must free its slot")
		}
		time.Sleep(20 * time.Millisecond)
	}
	// Another session is unaffected.
	rso := h.login(t, "rso", "x")
	req, _ := http.NewRequestWithContext(ctx, http.MethodGet, rso.base+"/api/events", nil)
	r3, err := rso.client.Do(req)
	if err != nil || r3.StatusCode != http.StatusOK {
		t.Fatalf("a different session must still stream: %v %v", err, r3)
	}
}

// ATTACK: information disclosure. /api/readyz is unauthenticated; it must not echo the kernel
// error text (socket paths, internals) to an anonymous caller.
func TestT11ReadyzDoesNotLeakKernelErrorText(t *testing.T) {
	h := newLiveHarness(t)
	h.api.opts.Ready = fakeProbe{err: fmt.Errorf("dial unix /var/lib/sea-forge/secret-cell/server.sock: connection refused")}
	resp, err := http.Get(h.ts.URL + "/api/readyz")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	body, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusServiceUnavailable {
		t.Fatalf("want 503, got %d", resp.StatusCode)
	}
	if strings.Contains(string(body), "secret-cell") || strings.Contains(string(body), "server.sock") {
		t.Fatalf("readyz leaked the kernel error: %s", body)
	}
}
