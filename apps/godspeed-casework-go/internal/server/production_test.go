// T07 permanent teeth: the security posture a later change must never weaken. Each test names
// the attack it pins. The evidence transcripts under
// .agents/evidence/casework-live-wiring/T07/teeth/ are captures of these tests (plus the live
// variants in server_live_teeth_test.go) running against the real server.
package server

import (
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/cookiejar"
	"net/http/httptest"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/auth"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// --- shared helpers ---

func newCookieJarClient(t *testing.T) *http.Client {
	t.Helper()
	jar, err := cookiejar.New(nil)
	if err != nil {
		t.Fatal(err)
	}
	return &http.Client{Jar: jar}
}

func jarCookieValue(t *testing.T, client *http.Client, base, name string) string {
	t.Helper()
	u, err := url.Parse(base)
	if err != nil {
		t.Fatal(err)
	}
	for _, c := range client.Jar.Cookies(u) {
		if c.Name == name {
			return c.Value
		}
	}
	return ""
}

// postWithCSRF performs a state-changing POST with an EXPLICIT CSRF header value (so tests can
// present a forged token too).
func (u *testUser) postWithCSRF(path, contentType, body, csrf string) (*http.Response, error) {
	req, err := http.NewRequest(http.MethodPost, u.base+path, strings.NewReader(body))
	if err != nil {
		u.t.Fatal(err)
	}
	req.Header.Set("Content-Type", contentType)
	req.Header.Set(csrfHeader, csrf)
	return u.client.Do(req)
}

// fakeProbe is the readiness fake: readyz reflects the kernel's own readiness.get.
type fakeProbe struct {
	err   error
	total string
}

func (f fakeProbe) Readiness(ctx context.Context) (ports.ReadinessReport, error) {
	return ports.ReadinessReport{Overall: f.total}, f.err
}

// --- TEETH ---

// T07 TEETH (a): a cross-origin POST /api/intents with a VALID session cookie but no CSRF token
// must be 403 with ZERO kernel calls. The cookie proves nothing; the synchronizer token does.
func TestToothAValidCookieWithoutCSRFIs403AndKernelUntouched(t *testing.T) {
	h := newLiveHarness(t)
	op := h.login(t, "operator", "ignored-in-dev")

	body := `{"intent_id":"tooth-a","kind":"CONSEQUENTIAL_CASE","action_name":"EXECUTE_ITEM","case_id":"case_1","client_cursor":"01AAA","parameters":{"item_id":"i"}}`
	resp, err := op.postNoCSRF("/api/intents", "application/json", body)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusForbidden {
		t.Fatalf("a cookie without the CSRF token must be 403, got %d", resp.StatusCode)
	}
	raw, _ := io.ReadAll(resp.Body)
	var wire struct {
		Error struct {
			Kind string `json:"kind"`
		} `json:"error"`
	}
	if err := json.Unmarshal(raw, &wire); err != nil || wire.Error.Kind != "csrf_refused" {
		t.Fatalf("the refusal must be typed csrf_refused, got %s", raw)
	}
	// ZERO kernel calls: the dispatcher (the gateway's only path to the kernel) was never invoked.
	if h.ints.called() {
		t.Fatal("a CSRF refusal must never reach the intent dispatcher")
	}
	// A WRONG token is equally refused.
	resp2, err := op.postWithCSRF("/api/intents", "application/json", body, "forged-token")
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	if resp2.StatusCode != http.StatusForbidden {
		t.Fatalf("a forged CSRF token must be 403, got %d", resp2.StatusCode)
	}
	if h.ints.called() {
		t.Fatal("a forged CSRF token must never reach the intent dispatcher")
	}
}

// T07 TEETH (d): user A's session can never act as user B's kernel actor - not via a forged
// payload actor, not via the removed ?actor= override. Two DIFFERENT sessions are exercised so
// the sessions provably carry different standings (the shape T10's L5 two-user ladder needs).
func TestToothDSessionCannotActAsAnotherActor(t *testing.T) {
	h := newLiveHarness(t)
	op := h.login(t, "operator", "ignored-in-dev")
	rso := h.login(t, "rso", "ignored-in-dev")

	if op.identity.ActorID != "operator_local" || rso.identity.ActorID != "rso_local" {
		t.Fatalf("the two sessions must carry distinct kernel actors: %q vs %q", op.identity.ActorID, rso.identity.ActorID)
	}

	intent := func(id string) string {
		return `{"intent_id":"` + id + `","kind":"CONSEQUENTIAL_CASE","action_name":"EXECUTE_ITEM","case_id":"case_1","client_cursor":"01AAA","actor":{"actor_id":"operator_local","role":"operator"},"parameters":{"item_id":"i"}}`
	}
	// The R-SO session posts an intent whose payload claims to be the OPERATOR actor.
	resp, err := rso.post("/api/intents", "application/json", intent("tooth-d-1"))
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("the envelope itself is well-formed (200 with a typed outcome), got %d", resp.StatusCode)
	}
	if h.ints.last.Actor.ActorID != "rso_local" || h.ints.last.Actor.Role != "R-SO" {
		t.Fatalf("the forged payload actor must be overwritten with the session's standing, got %+v", h.ints.last.Actor)
	}

	// The ?actor= override is refused outright for authenticated requests.
	resp2, err := rso.get("/api/world?actor=operator_local&role=operator")
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	if resp2.StatusCode != http.StatusBadRequest {
		t.Fatalf("the ?actor= override must be refused, got %d", resp2.StatusCode)
	}

	// Each session's /api/world renders ITS OWN standing.
	h.pushRevision(t, "01AAA", "case_1")
	for _, tc := range []struct {
		u    *testUser
		want string
	}{{op, "operator_local"}, {rso, "rso_local"}} {
		resp3, err := tc.u.get("/api/world")
		if err != nil {
			t.Fatal(err)
		}
		var wr worldResponse
		if err := json.NewDecoder(resp3.Body).Decode(&wr); err != nil {
			t.Fatal(err)
		}
		resp3.Body.Close()
		if wr.Snapshot.Perspective.ActorID != tc.want {
			t.Fatalf("the world must speak as %s, got %+v", tc.want, wr.Snapshot.Perspective)
		}
	}
}

// T07 TEETH (e): a burst beyond the token bucket is a typed 429 and the kernel stays untouched.
func TestToothERateLimitRefusesBursts(t *testing.T) {
	feed := newFakeFeed()
	world := &fakeWorld{snaps: map[string]contract.CognitiveWorldSnapshot{}, newestID: "case_1"}
	ints := &fakeIntents{resp: contract.IntentResponse{IntentID: "i", Success: true}}
	store := projection.NewStore()
	relay := NewRelay(feed, world, store, RelayOptions{DefaultActor: ports.ActorClaim{ActorID: "op", Role: "operator"}})
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go relay.Run(ctx)
	api := newAuthedServer(t, world, ints, fakeTemplates{}, store, relay, Options{Heartbeat: time.Hour})
	// Tighten the bucket: burst 2, refilling at 6/minute.
	api.intentsLimiter = NewRateLimiter(6, 2, 0, nil)
	ts := httptest.NewServer(api.Handler())
	defer ts.Close()
	op := newSessionFromServer(t, ts.URL)

	body := `{"intent_id":"rate-%d","kind":"CONSEQUENTIAL_CASE","action_name":"EXECUTE_ITEM","case_id":"case_1","client_cursor":"01AAA","parameters":{"item_id":"i"}}`
	codes := []int{}
	for i := 0; i < 4; i++ {
		resp, err := op.post("/api/intents", "application/json", strings.Replace(body, "%d", string(rune('0'+i)), 1))
		if err != nil {
			t.Fatal(err)
		}
		codes = append(codes, resp.StatusCode)
		resp.Body.Close()
	}
	if codes[0] != http.StatusOK || codes[1] != http.StatusOK {
		t.Fatalf("the first two intents are inside the burst, got %v", codes)
	}
	if codes[2] != http.StatusTooManyRequests || codes[3] != http.StatusTooManyRequests {
		t.Fatalf("the burst beyond the bucket must be 429, got %v", codes)
	}
	// The refusal is typed, not a bare status.
	resp2, err := op.post("/api/intents", "application/json", strings.Replace(body, "%d", "x", 1))
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	raw, _ := io.ReadAll(resp2.Body)
	if !strings.Contains(string(raw), "rate_limited") {
		t.Fatalf("the 429 must be typed rate_limited, got %s", raw)
	}
	// Kernel untouched by the refused requests: only the two admitted intents arrived.
	if got := ints.dispatched(); got != 2 {
		t.Fatalf("exactly 2 intents may reach the dispatcher, got %d", got)
	}
}

// The per-IP bucket backs the per-session one: a second identity on the same address shares the
// address budget (spoofable X-Forwarded-For headers are deliberately ignored).
func TestRateLimitPerIPBucketIsShared(t *testing.T) {
	feed := newFakeFeed()
	world := &fakeWorld{snaps: map[string]contract.CognitiveWorldSnapshot{}, newestID: "case_1"}
	ints := &fakeIntents{resp: contract.IntentResponse{IntentID: "i", Success: true}}
	store := projection.NewStore()
	relay := NewRelay(feed, world, store, RelayOptions{DefaultActor: ports.ActorClaim{ActorID: "op", Role: "operator"}})
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go relay.Run(ctx)
	api := newAuthedServer(t, world, ints, fakeTemplates{}, store, relay, Options{Heartbeat: time.Hour})
	api.intentsLimiter = NewRateLimiter(6, 2, 0, nil)
	ts := httptest.NewServer(api.Handler())
	defer ts.Close()

	op := newSessionAs(t, ts.URL, "operator", "ignored-in-dev")
	bearer := &http.Client{} // no cookies: bearer identity instead
	body := `{"intent_id":"ip-%d","kind":"CONSEQUENTIAL_CASE","action_name":"EXECUTE_ITEM","case_id":"case_1","client_cursor":"01AAA","parameters":{"item_id":"i"}}`
	// The session exhausts its burst.
	for i := 0; i < 2; i++ {
		resp, err := op.post("/api/intents", "application/json", strings.Replace(body, "%d", string(rune('0'+i)), 1))
		if err != nil {
			t.Fatal(err)
		}
		resp.Body.Close()
	}
	// A bearer request is a DIFFERENT session bucket but the SAME IP bucket: refused.
	req, _ := http.NewRequest(http.MethodPost, ts.URL+"/api/intents", strings.NewReader(strings.Replace(body, "%d", "b", 1)))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer dev-bearer-token")
	resp, err := bearer.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusTooManyRequests {
		t.Fatalf("the per-IP bucket must refuse a second identity at the same address, got %d", resp.StatusCode)
	}
}

// T07 TEETH (c) as a named permanent test: an unauthenticated GET /api/world gets a typed 401
// and no snapshot content.
func TestToothCUnauthenticatedWorldIs401NoSnapshotLeak(t *testing.T) {
	h := newLiveHarness(t)
	h.pushRevision(t, "01AAA", "case_1")
	resp, err := http.Get(h.ts.URL + "/api/world")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("got %d, want 401", resp.StatusCode)
	}
	raw, _ := io.ReadAll(resp.Body)
	for _, leak := range []string{"visible_objects", "world_id", "available_actions", "case_id"} {
		if strings.Contains(string(raw), leak) {
			t.Fatalf("the 401 must not leak %q: %s", leak, raw)
		}
	}
	// SSE and templates are equally closed.
	resp2, err := http.Get(h.ts.URL + "/api/events")
	if err != nil {
		t.Fatal(err)
	}
	resp2.Body.Close()
	if resp2.StatusCode != http.StatusUnauthorized {
		t.Fatalf("unauthenticated SSE must be 401, got %d", resp2.StatusCode)
	}
	resp3, err := http.Get(h.ts.URL + "/api/templates")
	if err != nil {
		t.Fatal(err)
	}
	resp3.Body.Close()
	if resp3.StatusCode != http.StatusUnauthorized {
		t.Fatalf("unauthenticated templates must be 401, got %d", resp3.StatusCode)
	}
}

// --- session lifecycle ---

// Session lifecycle: login issues the standing, logout destroys server-side state (the replayed
// cookie is dead), and the session cookie is HttpOnly.
func TestSessionLifecycleLoginSessionLogout(t *testing.T) {
	h := newLiveHarness(t)
	op := h.login(t, "operator", "ignored-in-dev")

	resp, err := op.get("/api/session")
	if err != nil {
		t.Fatal(err)
	}
	var sb sessionBody
	if err := json.NewDecoder(resp.Body).Decode(&sb); err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	if !sb.Authenticated || sb.ActorID != "operator_local" || sb.Role != "operator" || sb.CSRFToken != op.csrf {
		t.Fatalf("session: %+v", sb)
	}

	resp2, err := op.post("/api/auth/logout", "application/json", `{}`)
	if err != nil {
		t.Fatal(err)
	}
	resp2.Body.Close()
	if resp2.StatusCode != http.StatusOK {
		t.Fatalf("logout: %d", resp2.StatusCode)
	}
	// The destroyed session's cookie no longer resolves: a follow-up read is 401.
	resp3, err := op.get("/api/world")
	if err != nil {
		t.Fatal(err)
	}
	defer resp3.Body.Close()
	if resp3.StatusCode != http.StatusUnauthorized {
		t.Fatalf("a destroyed session must 401, got %d", resp3.StatusCode)
	}
}

// The session cookie must be HttpOnly (JS never reads it; the CSRF cookie must NOT be).
func TestSessionCookieIsHttpOnlyAndCSRFCookieIsNot(t *testing.T) {
	h := newLiveHarness(t)
	h.login(t, "operator", "ignored-in-dev")
	// Drive the login manually to inspect raw Set-Cookie headers.
	client := newCookieJarClient(t)
	resp, err := client.Get(h.ts.URL + "/api/session")
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	req, _ := http.NewRequest(http.MethodPost, h.ts.URL+"/api/auth/login",
		strings.NewReader(`{"username":"operator","password":"x"}`))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set(csrfHeader, jarCookieValue(t, client, h.ts.URL, csrfCookieName))
	resp2, err := client.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	var sawSession, sawCSRF bool
	for _, c := range resp2.Cookies() {
		switch c.Name {
		case sessionCookieName:
			sawSession = true
			if !c.HttpOnly {
				t.Fatal("the session cookie must be HttpOnly")
			}
		case csrfCookieName:
			sawCSRF = true
			if c.HttpOnly {
				t.Fatal("the CSRF cookie must be readable by the UI (it echoes the token)")
			}
		}
		if c.SameSite != http.SameSiteStrictMode {
			t.Fatalf("%s must be SameSite=Strict", c.Name)
		}
	}
	if !sawSession || !sawCSRF {
		t.Fatalf("login must set both cookies, saw session=%t csrf=%t", sawSession, sawCSRF)
	}
}

// The unauthenticated /api/session answers honestly (authenticated:false) and issues the
// pre-login CSRF cookie the login double-submit needs.
func TestSessionEndpointBootstrapsLoginCSRF(t *testing.T) {
	h := newLiveHarness(t)
	resp, err := http.Get(h.ts.URL + "/api/session")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("/api/session is the login bootstrap, got %d", resp.StatusCode)
	}
	var sb sessionBody
	if err := json.NewDecoder(resp.Body).Decode(&sb); err != nil {
		t.Fatal(err)
	}
	if sb.Authenticated {
		t.Fatalf("an unauthenticated session must say so: %+v", sb)
	}
	var csrfCookie *http.Cookie
	for _, c := range resp.Cookies() {
		if c.Name == csrfCookieName {
			csrfCookie = c
		}
	}
	if csrfCookie == nil || len(csrfCookie.Value) != 64 {
		t.Fatalf("the pre-login CSRF cookie must be issued (256-bit), got %+v", csrfCookie)
	}

	// The login POST without the double-submit pair is refused (login-CSRF defense).
	resp2, err := http.Post(h.ts.URL+"/api/auth/login", "application/json",
		strings.NewReader(`{"username":"operator","password":"x"}`))
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	if resp2.StatusCode != http.StatusForbidden {
		t.Fatalf("a login without the CSRF pair must be 403, got %d", resp2.StatusCode)
	}
}

// A wrong password is a typed 401 and issues no cookies (local mode: real argon2id verification).
func TestLoginRefusesBadCredentials(t *testing.T) {
	hash, err := auth.HashPassword("real-password")
	if err != nil {
		t.Fatal(err)
	}
	feed := newFakeFeed()
	world := &fakeWorld{snaps: map[string]contract.CognitiveWorldSnapshot{}, newestID: "case_1"}
	store := projection.NewStore()
	relay := NewRelay(feed, world, store, RelayOptions{DefaultActor: ports.ActorClaim{ActorID: "op", Role: "operator"}})
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go relay.Run(ctx)
	localStore, err := auth.NewLocalUserStore(auth.ModeLocal, []auth.User{
		{Username: "operator", PasswordHash: hash, ActorID: "operator_local", Role: "operator"},
	})
	if err != nil {
		t.Fatal(err)
	}
	api := New(world, &fakeIntents{}, fakeTemplates{}, store, relay, Options{
		Heartbeat: time.Hour,
		Auth: AuthOptions{
			Authenticator: localStore,
			Sessions:      auth.NewSessionStore(0, 0, 0, nil),
			CookieSecure:  false,
		},
	})
	ts := httptest.NewServer(api.Handler())
	defer ts.Close()

	client := newCookieJarClient(t)
	client.Get(ts.URL + "/api/session")
	req, _ := http.NewRequest(http.MethodPost, ts.URL+"/api/auth/login",
		strings.NewReader(`{"username":"operator","password":"wrong"}`))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set(csrfHeader, jarCookieValue(t, client, ts.URL, csrfCookieName))
	resp, err := client.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("a wrong password must be 401, got %d", resp.StatusCode)
	}
	if len(resp.Cookies()) != 0 {
		t.Fatalf("a failed login must issue no session cookie, got %+v", resp.Cookies())
	}
}

// The oidc surface wiring: GET /api/auth/login redirects to the provider, the callback issues
// the session, and POST /api/auth/login is a typed refusal. The REAL protocol behaviour
// (discovery, exchange, signature verification) is proven in internal/auth's tests against a
// conformant httptest provider; this test proves the SERVER wiring around the flow.
func TestOIDCServerWiring(t *testing.T) {
	h := newLiveHarness(t)
	h.api.opts.Auth.Authenticator = &fakeOIDCFlow{identity: auth.Identity{
		Username: "alice", ActorID: "operator_local", Role: "operator",
	}}

	// POST login in oidc mode is a typed refusal.
	resp, err := http.Post(h.ts.URL+"/api/auth/login", "application/json",
		strings.NewReader(`{"username":"alice","password":"x"}`))
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("oidc mode must refuse credential login, got %d", resp.StatusCode)
	}

	// GET /api/auth/login redirects to the provider with the flow's state. The client must NOT
	// follow the redirect (there is no real provider behind it).
	client := newCookieJarClient(t)
	client.CheckRedirect = func(req *http.Request, via []*http.Request) error {
		return http.ErrUseLastResponse
	}
	resp2, err := client.Get(h.ts.URL + "/api/auth/login")
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	if resp2.StatusCode != http.StatusFound {
		t.Fatalf("GET /api/auth/login must redirect, got %d", resp2.StatusCode)
	}
	loc := resp2.Header.Get("Location")
	if !strings.HasPrefix(loc, "https://idp.test/authorize?") || !strings.Contains(loc, "state=") {
		t.Fatalf("the redirect must target the provider with a state: %q", loc)
	}

	// The provider would redirect back to /api/auth/callback?code=..&state=..: extract the state
	// from the authorization redirect and complete the flow at the callback route.
	locURL, err := url.Parse(loc)
	if err != nil {
		t.Fatal(err)
	}
	state := locURL.Query().Get("state")
	resp3, err := client.Get(h.ts.URL + "/api/auth/callback?code=the-code&state=" + url.QueryEscape(state))
	if err != nil {
		t.Fatal(err)
	}
	defer resp3.Body.Close()
	if resp3.StatusCode != http.StatusSeeOther {
		t.Fatalf("the callback must redirect to /, got %d", resp3.StatusCode)
	}
	// The session works (redirect-following back on for normal requests).
	client.CheckRedirect = nil
	resp4, err := client.Get(h.ts.URL + "/api/session")
	if err != nil {
		t.Fatal(err)
	}
	defer resp4.Body.Close()
	var sb sessionBody
	if err := json.NewDecoder(resp4.Body).Decode(&sb); err != nil {
		t.Fatal(err)
	}
	if !sb.Authenticated || sb.ActorID != "operator_local" || sb.Username != "alice" {
		t.Fatalf("the oidc session must carry the mapped identity: %+v", sb)
	}
}

// fakeOIDCFlow scripts the oidc authenticator for server-wiring tests.
type fakeOIDCFlow struct {
	identity auth.Identity
}

func (f *fakeOIDCFlow) Mode() string { return "oidc" }

func (f *fakeOIDCFlow) LoginURL(_ context.Context) (string, string, error) {
	return "state-1", "https://idp.test/authorize?state=state-1", nil
}

func (f *fakeOIDCFlow) Callback(_ context.Context, state, code string) (auth.Identity, error) {
	if state != "state-1" || code == "" {
		return auth.Identity{}, auth.ErrInvalidCredentials
	}
	return f.identity, nil
}

// Readyz reflects the kernel's own readiness.get through the wired probe.
func TestReadyzReflectsKernelReadiness(t *testing.T) {
	h := newLiveHarness(t)
	h.api.opts.Ready = fakeProbe{total: "ready"}
	resp, err := http.Get(h.ts.URL + "/api/readyz")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("a ready kernel is 200, got %d", resp.StatusCode)
	}
	var body struct {
		Status string `json:"status"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil || body.Status != "ready" {
		t.Fatalf("readyz: %+v err=%v", body, err)
	}

	h.api.opts.Ready = fakeProbe{total: "degraded"}
	resp2, err := http.Get(h.ts.URL + "/api/readyz")
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	if resp2.StatusCode != http.StatusServiceUnavailable {
		t.Fatalf("a degraded kernel is 503, got %d", resp2.StatusCode)
	}
	h.api.opts.Ready = nil
	resp3, err := http.Get(h.ts.URL + "/api/readyz")
	if err != nil {
		t.Fatal(err)
	}
	defer resp3.Body.Close()
	if resp3.StatusCode != http.StatusServiceUnavailable {
		t.Fatalf("no probe wired is 503, got %d", resp3.StatusCode)
	}
}

// Unknown /api paths are typed JSON 404s, never the SPA fallback (an HTML shell must not answer
// API routes).
func TestUnknownAPIRoutesAreTyped404s(t *testing.T) {
	h := newLiveHarness(t)
	resp, err := http.Get(h.ts.URL + "/api/nonexistent")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("got %d, want 404", resp.StatusCode)
	}
	if ct := resp.Header.Get("Content-Type"); !strings.Contains(ct, "application/json") {
		t.Fatalf("API 404s are JSON, got %q", ct)
	}
}

// --- static serving ---

func writeFakeDist(t *testing.T) string {
	t.Helper()
	root := t.TempDir()
	assets := filepath.Join(root, "assets")
	if err := os.MkdirAll(assets, 0o755); err != nil {
		t.Fatal(err)
	}
	files := map[string]string{
		"index.html":            "<!doctype html><html><head><script src=\"/assets/app-abc123.js\"></script></head><body></body></html>",
		"assets/app-abc123.js":  "console.log('hashed bundle')",
		"assets/app-abc123.css": "body{}",
		"favicon.ico":           "icon",
	}
	for name, content := range files {
		if err := os.WriteFile(filepath.Join(root, name), []byte(content), 0o644); err != nil {
			t.Fatal(err)
		}
	}
	return root
}

func TestStaticServesIndexAssetsAndSPAFallback(t *testing.T) {
	h := newLiveHarness(t)
	h.api.opts.StaticRoot = writeFakeDist(t)
	// Rebuild the server wrapper with the static root configured.
	h.ts.Close()
	h.ts = httptest.NewServer(h.api.Handler())
	t.Cleanup(h.ts.Close)

	// index.html: no-cache + CSP with a strict script-src.
	resp, err := http.Get(h.ts.URL + "/")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("index: %d", resp.StatusCode)
	}
	if cc := resp.Header.Get("Cache-Control"); cc != "no-cache" {
		t.Fatalf("index cache: %q", cc)
	}
	csp := resp.Header.Get("Content-Security-Policy")
	if !strings.Contains(csp, "script-src 'self'") {
		t.Fatalf("script-src must be 'self', got %q", csp)
	}
	scriptPart := strings.Split(csp, "style-src")[0]
	if strings.Contains(scriptPart, "unsafe-inline") || strings.Contains(scriptPart, "unsafe-eval") {
		t.Fatalf("script-src must not carry unsafe-inline/eval, got %q", csp)
	}
	raw, _ := io.ReadAll(resp.Body)
	if !strings.Contains(string(raw), "app-abc123.js") {
		t.Fatalf("index content: %s", raw)
	}

	// A hashed asset: immutable + the right content type.
	resp2, err := http.Get(h.ts.URL + "/assets/app-abc123.js")
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	if cc := resp2.Header.Get("Cache-Control"); !strings.Contains(cc, "immutable") || !strings.Contains(cc, "max-age=31536000") {
		t.Fatalf("hashed assets must be immutable, got %q", cc)
	}
	if ct := resp2.Header.Get("Content-Type"); !strings.Contains(ct, "javascript") {
		t.Fatalf("js content type: %q", ct)
	}

	// SPA fallback: a deep link serves the shell with no-cache.
	resp3, err := http.Get(h.ts.URL + "/some/client/route")
	if err != nil {
		t.Fatal(err)
	}
	defer resp3.Body.Close()
	if cc := resp3.Header.Get("Cache-Control"); cc != "no-cache" {
		t.Fatalf("SPA fallback cache: %q", cc)
	}
	raw3, _ := io.ReadAll(resp3.Body)
	if !strings.Contains(string(raw3), "app-abc123.js") {
		t.Fatalf("SPA fallback must serve the shell, got %s", raw3)
	}

	// But /api routes never fall through to the shell.
	resp4, err := http.Get(h.ts.URL + "/api/definitely-not-a-route")
	if err != nil {
		t.Fatal(err)
	}
	defer resp4.Body.Close()
	if ct := resp4.Header.Get("Content-Type"); !strings.Contains(ct, "application/json") {
		t.Fatalf("/api 404s must stay JSON, got %q", ct)
	}
}
