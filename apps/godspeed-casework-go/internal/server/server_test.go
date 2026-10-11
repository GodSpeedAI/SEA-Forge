// The live server's HTTP+SSE tests over fakes and a real relay + revision store: the served
// shapes, the cursor-keyed history with its documented 404, template endpoints, intent envelope
// pass-through, and the SSE replay/resume contract (Last-Event-ID AND ?last=) with no gaps.
package server

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/cookiejar"
	"net/http/httptest"
	"net/url"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/auth"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// fakeWorld is a canned WorldSource.
type fakeWorld struct {
	mu       sync.Mutex
	snaps    map[string]contract.CognitiveWorldSnapshot
	newestID string
	verify   func(ports.ActorClaim) error
}

func (f *fakeWorld) set(caseID string, snap contract.CognitiveWorldSnapshot) {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.snaps[caseID] = snap
}

func (f *fakeWorld) Snapshot(ctx context.Context, caseID string, actor ports.ActorClaim, cursor string) (contract.CognitiveWorldSnapshot, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	snap, ok := f.snaps[caseID]
	if !ok {
		return contract.CognitiveWorldSnapshot{}, context.DeadlineExceeded
	}
	snap.Perspective = contract.ActorPerspective{ActorID: actor.ActorID, Role: actor.Role}
	snap.Cursor = cursor
	return snap, nil
}

func (f *fakeWorld) NewestCaseID(ctx context.Context) (string, error) { return f.newestID, nil }

func (f *fakeWorld) VerifyPerspective(ctx context.Context, actor ports.ActorClaim) error {
	if f.verify != nil {
		return f.verify(actor)
	}
	return nil
}

// fakeTemplates is a canned TemplateSource.
type fakeTemplates struct{}

func (fakeTemplates) EntryOptions(ctx context.Context) ([]contract.TemplateEntryOption, error) {
	return []contract.TemplateEntryOption{{
		TemplateRef: "e2e-sentry-chain@0.1.0",
		Title:       "e2e-sentry-chain",
		Parameters:  []contract.TemplateParameter{{Name: "dataset_name", Type: "string", Required: boolPtr(true)}},
	}}, nil
}

func (fakeTemplates) Preflight(ctx context.Context, ref string, params map[string]any) (contract.TemplatePreflightResult, error) {
	digest := "sha256:385e"
	return contract.TemplatePreflightResult{
		TemplateRef: ref, Params: params, Passed: params != nil, Reasons: []string{}, Digest: &digest,
	}, nil
}

func boolPtr(b bool) *bool { return &b }

// fakeIntents records the last intent and answers a canned response; the counter is the proof
// surface for "the kernel received no call" in the CSRF/rate-limit teeth.
type fakeIntents struct {
	mu   sync.Mutex
	last contract.InteractionIntent
	resp contract.IntentResponse
	n    int
}

func (f *fakeIntents) Handle(ctx context.Context, in contract.InteractionIntent) contract.IntentResponse {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.last = in
	f.n++
	return f.resp
}

// called reports whether ANY intent reached the dispatcher.
func (f *fakeIntents) called() bool {
	f.mu.Lock()
	defer f.mu.Unlock()
	return f.n > 0
}

// dispatched reports how many intents reached the dispatcher.
func (f *fakeIntents) dispatched() int {
	f.mu.Lock()
	defer f.mu.Unlock()
	return f.n
}

// fakeFeed is a drivable EventFeed.
type fakeFeed struct {
	ch   chan KernelEvent
	done chan struct{}
}

func newFakeFeed() *fakeFeed {
	return &fakeFeed{ch: make(chan KernelEvent, 16), done: make(chan struct{})}
}

func (f *fakeFeed) Events() <-chan KernelEvent { return f.ch }
func (f *fakeFeed) Done() <-chan struct{}      { return f.done }
func (f *fakeFeed) push(ev KernelEvent)        { f.ch <- ev }

func snapshotFor(cursor, caseID string) contract.CognitiveWorldSnapshot {
	return contract.CognitiveWorldSnapshot{
		WorldID:          "world-" + caseID,
		CaseID:           caseID,
		Cursor:           cursor,
		Timestamp:        time.Date(2026, 9, 25, 12, 0, 0, 0, time.UTC).Format(time.RFC3339),
		Perspective:      contract.ActorPerspective{ActorID: "operator_local", Role: "operator"},
		VisibleObjects:   []contract.CognitiveObject{},
		AvailableActions: []contract.ActionDescriptor{},
	}
}

// liveHarness assembles the live server over fakes plus a real relay + store driven by a fake feed,
// wired with the T07 session layer (dev-mode local users: passwords unchecked, as in the dev
// posture; credential verification itself is proven in internal/auth's tests).
type liveHarness struct {
	feed   *fakeFeed
	world  *fakeWorld
	ints   *fakeIntents
	store  *projection.Store
	relay  *Relay
	ts     *httptest.Server
	cancel func()
	api    *Server
}

func newLiveHarness(t *testing.T) *liveHarness {
	t.Helper()
	feed := newFakeFeed()
	world := &fakeWorld{snaps: map[string]contract.CognitiveWorldSnapshot{}, newestID: "case_1"}
	ints := &fakeIntents{resp: contract.IntentResponse{IntentID: "i-1", Success: true}}
	store := projection.NewStore()
	relay := NewRelay(feed, world, store, RelayOptions{
		DefaultActor: ports.ActorClaim{ActorID: "operator_local", Role: "operator"},
	})
	ctx, cancel := context.WithCancel(context.Background())
	go relay.Run(ctx)
	api := newAuthedServer(t, world, ints, fakeTemplates{}, store, relay, Options{Heartbeat: time.Hour})
	ts := httptest.NewServer(api.Handler())
	t.Cleanup(func() { cancel(); ts.Close() })
	return &liveHarness{feed: feed, world: world, ints: ints, store: store, relay: relay, ts: ts, cancel: cancel, api: api}
}

// login authenticates a harness user (dev posture: the password is unchecked).
func (h *liveHarness) login(t *testing.T, username, password string) *testUser {
	t.Helper()
	return newSessionAs(t, h.ts.URL, username, password)
}

// testAuthOptions builds the dev-mode auth wiring (passwords unchecked, two users mirroring the
// live cell's operator + R-SO standing, plus the dev static bearer token).
func testAuthOptions(t *testing.T) AuthOptions {
	t.Helper()
	users := []auth.User{
		{Username: "operator", DisplayName: "Ops", ActorID: "operator_local", Role: "operator"},
		{Username: "rso", ActorID: "rso_local", Role: "R-SO"},
	}
	store, err := auth.NewLocalUserStore(auth.ModeDev, users)
	if err != nil {
		t.Fatal(err)
	}
	sessions := auth.NewSessionStore(auth.DefaultIdleTTL, auth.DefaultAbsoluteTTL, 0, nil)
	bearerID, err := store.StaticTokenIdentity("")
	if err != nil {
		t.Fatal(err)
	}
	return AuthOptions{
		Authenticator:  store,
		Sessions:       sessions,
		CookieSecure:   false, // loopback dev posture
		StaticToken:    "dev-bearer-token",
		BearerIdentity: &bearerID,
	}
}

// newAuthedServer is New() with the T07 auth layer and generous rate limits pre-wired.
func newAuthedServer(t *testing.T, world WorldSource, dispatcher IntentDispatcher, tpl TemplateSource, store RevisionHistory, relay RelayCursors, opts Options) *Server {
	t.Helper()
	opts.Auth = testAuthOptions(t)
	opts.RateLimit = RateLimitOptions{PerMinute: 600, Burst: 50}
	return New(world, dispatcher, tpl, store, relay, opts)
}

// testUser is one authenticated browser session: a cookie-jarred client plus the CSRF token the
// login issued (state-changing POSTs must present it).
type testUser struct {
	t        *testing.T
	base     string
	client   *http.Client
	csrf     string
	identity sessionBody
}

// newSessionFromServer runs the credential flow against any authed server (dev user "operator").
func newSessionFromServer(t *testing.T, base string) *testUser {
	t.Helper()
	return newSessionAs(t, base, "operator", "ignored-in-dev")
}

// newSessionAs is newSessionFromServer for an explicit credential pair.
func newSessionAs(t *testing.T, base, username, password string) *testUser {
	t.Helper()
	jar, err := cookiejar.New(nil)
	if err != nil {
		t.Fatal(err)
	}
	u := &testUser{t: t, base: base, client: &http.Client{Jar: jar}}
	// 1. Any GET mints the pre-login CSRF cookie (double submit).
	resp, err := u.client.Get(u.base + "/api/session")
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	// 2. POST the credentials with the cookie's value echoed in the header.
	header := u.cookieValue(csrfCookieName)
	body := fmt.Sprintf(`{"username":%q,"password":%q}`, username, password)
	req, err := http.NewRequest(http.MethodPost, u.base+"/api/auth/login", strings.NewReader(body))
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set(csrfHeader, header)
	resp, err = u.client.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		raw, _ := io.ReadAll(resp.Body)
		t.Fatalf("login as %s: status %d body %s", username, resp.StatusCode, raw)
	}
	if err := json.NewDecoder(resp.Body).Decode(&u.identity); err != nil {
		t.Fatal(err)
	}
	u.csrf = u.identity.CSRFToken
	if u.csrf == "" {
		t.Fatal("login must issue a CSRF token")
	}
	return u
}

func (u *testUser) cookieValue(name string) string {
	u.t.Helper()
	base, err := url.Parse(u.base)
	if err != nil {
		u.t.Fatal(err)
	}
	for _, c := range u.client.Jar.Cookies(base) {
		if c.Name == name {
			return c.Value
		}
	}
	return ""
}

// get performs an authenticated GET.
func (u *testUser) get(path string) (*http.Response, error) {
	req, err := http.NewRequest(http.MethodGet, u.base+path, nil)
	if err != nil {
		u.t.Fatal(err)
	}
	return u.client.Do(req)
}

// post performs an authenticated state-changing POST with the session's CSRF token.
func (u *testUser) post(path, contentType, body string) (*http.Response, error) {
	req, err := http.NewRequest(http.MethodPost, u.base+path, strings.NewReader(body))
	if err != nil {
		u.t.Fatal(err)
	}
	req.Header.Set("Content-Type", contentType)
	req.Header.Set(csrfHeader, u.csrf)
	return u.client.Do(req)
}

// postNoCSRF performs a state-changing POST deliberately WITHOUT the CSRF header (the teeth).
func (u *testUser) postNoCSRF(path, contentType, body string) (*http.Response, error) {
	req, err := http.NewRequest(http.MethodPost, u.base+path, strings.NewReader(body))
	if err != nil {
		u.t.Fatal(err)
	}
	req.Header.Set("Content-Type", contentType)
	return u.client.Do(req)
}

func (h *liveHarness) pushRevision(t *testing.T, cursor, caseID string) {
	t.Helper()
	h.world.snaps[caseID] = snapshotFor(cursor, caseID)
	h.feed.push(KernelEvent{Cursor: cursor, Kind: "case.trace.item_activated", CaseID: caseID, At: time.Now()})
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		if _, err := h.store.At(cursor); err == nil {
			return
		}
		time.Sleep(2 * time.Millisecond)
	}
	t.Fatalf("relay never recorded revision %s", cursor)
}

func TestHealthzProvenanceIsTheLiveLabel(t *testing.T) {
	h := newLiveHarness(t)
	resp, err := http.Get(h.ts.URL + "/api/healthz")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var got struct {
		Status       string `json:"status"`
		Provenance   string `json:"provenance"`
		KernelCursor string `json:"kernel_cursor"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatal(err)
	}
	if got.Status != "ok" || got.Provenance != "go:live:sfwp" {
		t.Fatalf("healthz: %+v", got)
	}
}

func TestWorldServesLiveSnapshotAtPerCaseCursor(t *testing.T) {
	h := newLiveHarness(t)
	h.pushRevision(t, "01AAA", "case_1")
	op := h.login(t, "operator", "ignored-in-dev")

	resp, err := op.get("/api/world")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var got worldResponse
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatal(err)
	}
	if got.Snapshot.CaseID != "case_1" || got.Snapshot.Cursor != "01AAA" {
		t.Fatalf("world: case=%q cursor=%q", got.Snapshot.CaseID, got.Snapshot.Cursor)
	}
	// The snapshot's perspective is the SESSION's kernel actor.
	if got.Snapshot.Perspective.ActorID != "operator_local" || got.Snapshot.Perspective.Role != "operator" {
		t.Fatalf("the world must render the session's perspective, got %+v", got.Snapshot.Perspective)
	}
}

func TestWorldServesKernelHistoryAtCursorAnd404sEvicted(t *testing.T) {
	h := newLiveHarness(t)
	h.pushRevision(t, "01AAA", "case_1")
	h.pushRevision(t, "01BBB", "case_1")
	op := h.login(t, "operator", "ignored-in-dev")

	resp, err := op.get("/api/world?cursor=01AAA")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var got worldResponse
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatal(err)
	}
	if got.Snapshot.Cursor != "01AAA" {
		t.Fatalf("history must serve the revision at the requested kernel cursor, got %q", got.Snapshot.Cursor)
	}

	resp2, err := op.get("/api/world?cursor=01ZZZ")
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	if resp2.StatusCode != http.StatusNotFound {
		t.Fatalf("an unknown/evicted cursor must 404, got %d", resp2.StatusCode)
	}
}

func TestWorldEmptyCellServesHonestEmptyWorld(t *testing.T) {
	h := newLiveHarness(t)
	h.world.newestID = ""
	op := h.login(t, "operator", "ignored-in-dev")
	resp, err := op.get("/api/world")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var got worldResponse
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatal(err)
	}
	if got.Snapshot.CaseID != "" || got.Snapshot.Summary.Headline != "No cases yet" {
		t.Fatalf("empty world: %+v", got.Snapshot)
	}
}

// T07 tooth (c)+part of (d): unauthenticated reads are 401 (no snapshot leak), and the pre-T07
// ?actor=&role= override is refused for authenticated requests - identity is session-bound.
func TestWorldIdentityIsSessionBound(t *testing.T) {
	h := newLiveHarness(t)
	h.pushRevision(t, "01AAA", "case_1")
	op := h.login(t, "operator", "ignored-in-dev")

	// Unauthenticated: 401, and the body is the typed error, not a snapshot.
	resp, err := http.Get(h.ts.URL + "/api/world")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("an unauthenticated world read must be 401, got %d", resp.StatusCode)
	}
	raw, _ := io.ReadAll(resp.Body)
	if strings.Contains(string(raw), "visible_objects") || strings.Contains(string(raw), "world_id") {
		t.Fatalf("the 401 must not leak snapshot content: %s", raw)
	}

	// The ?actor= override is refused even with a valid session.
	resp2, err := op.get("/api/world?actor=operator_local&role=operator")
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	if resp2.StatusCode != http.StatusBadRequest {
		t.Fatalf("the ?actor= override must be refused, got %d", resp2.StatusCode)
	}

	// A half-named override is the same refusal.
	resp3, err := op.get("/api/world?actor=solo")
	if err != nil {
		t.Fatal(err)
	}
	defer resp3.Body.Close()
	if resp3.StatusCode != http.StatusBadRequest {
		t.Fatalf("a half-named override must be refused, got %d", resp3.StatusCode)
	}
}

func TestIntentsPassthroughAndStrictness(t *testing.T) {
	h := newLiveHarness(t)
	op := h.login(t, "operator", "ignored-in-dev")
	// The payload carries a FORGED actor: the session's standing must overwrite it (T07).
	body := `{"intent_id":"i-1","kind":"CONSEQUENTIAL_CASE","action_name":"EXECUTE_ITEM","target_object_id":"item-1","case_id":"case_1","client_cursor":"01AAA","actor":{"actor_id":"someone_else","role":"admin"},"parameters":{"item_id":"item-1"}}`
	resp, err := op.post("/api/intents", "application/json", body)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var got contract.IntentResponse
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatal(err)
	}
	if !got.Success || got.IntentID != "i-1" {
		t.Fatalf("intent response: %+v", got)
	}
	// The dispatcher saw the SESSION's actor, never the forged one.
	if h.ints.last.Actor.ActorID != "operator_local" || h.ints.last.Actor.Role != "operator" {
		t.Fatalf("the session's kernel standing must overwrite the payload actor: %+v", h.ints.last.Actor)
	}
	if h.ints.last.ClientCursor != "01AAA" {
		t.Fatalf("the rest of the intent envelope must pass through intact: %+v", h.ints.last)
	}

	// Unknown fields are refused at the envelope (the contract is the wire).
	bad := strings.Replace(body, `"parameters":`, `"surprise":1,"parameters":`, 1)
	resp2, err := op.post("/api/intents", "application/json", bad)
	if err != nil {
		t.Fatal(err)
	}
	resp2.Body.Close()
	if resp2.StatusCode != http.StatusBadRequest {
		t.Fatalf("an unknown envelope field must be 400, got %d", resp2.StatusCode)
	}
}

func TestTemplatesAndPreflight(t *testing.T) {
	h := newLiveHarness(t)
	op := h.login(t, "operator", "ignored-in-dev")
	resp, err := op.get("/api/templates")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var got templatesResponse
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatal(err)
	}
	if len(got.Templates) != 1 || got.Templates[0].TemplateRef != "e2e-sentry-chain@0.1.0" {
		t.Fatalf("templates: %+v", got)
	}

	resp2, err := op.post("/api/templates/preflight", "application/json",
		`{"template_ref":"e2e-sentry-chain@0.1.0","params":{"dataset_name":"orders"}}`)
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	var pf contract.TemplatePreflightResult
	if err := json.NewDecoder(resp2.Body).Decode(&pf); err != nil {
		t.Fatal(err)
	}
	if !pf.Passed || pf.Digest == nil || *pf.Digest != "sha256:385e" {
		t.Fatalf("preflight: %+v", pf)
	}
}

// --- SSE ---

type liveSSEEvent struct {
	id    string
	event string
	data  string
}

func streamLiveSSE(t *testing.T, body io.Reader) <-chan liveSSEEvent {
	t.Helper()
	ch := make(chan liveSSEEvent, 256)
	go func() {
		defer close(ch)
		reader := bufio.NewReader(body)
		var ev liveSSEEvent
		for {
			line, err := reader.ReadString('\n')
			switch {
			case strings.HasPrefix(line, "id: "):
				ev.id = strings.TrimSuffix(strings.TrimPrefix(line, "id: "), "\n")
			case strings.HasPrefix(line, "event: "):
				ev.event = strings.TrimSuffix(strings.TrimPrefix(line, "event: "), "\n")
			case strings.HasPrefix(line, "data: "):
				ev.data = strings.TrimSuffix(strings.TrimPrefix(line, "data: "), "\n")
			case strings.HasPrefix(line, ":"):
				ch <- liveSSEEvent{event: "comment", data: strings.TrimSuffix(strings.TrimPrefix(line, ":"), "\n")}
			case line == "\n" || line == "":
				if ev.event != "" || ev.data != "" {
					ch <- ev
					ev = liveSSEEvent{}
				}
			}
			if err != nil {
				return
			}
		}
	}()
	return ch
}

func nextLiveEvent(t *testing.T, ch <-chan liveSSEEvent, what string) liveSSEEvent {
	t.Helper()
	select {
	case ev, ok := <-ch:
		if !ok {
			t.Fatalf("stream closed while waiting for %s", what)
		}
		return ev
	case <-time.After(3 * time.Second):
		t.Fatalf("timed out waiting for %s", what)
	}
	return liveSSEEvent{}
}

func TestSSEReplayFromLastAndLiveWithKernelCursorIDs(t *testing.T) {
	h := newLiveHarness(t)
	h.pushRevision(t, "01AAA", "case_1")
	h.pushRevision(t, "01BBB", "case_1")
	op := h.login(t, "operator", "ignored-in-dev")

	resp, err := op.get("/api/events?last=01AAA")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if ct := resp.Header.Get("Content-Type"); !strings.HasPrefix(ct, "text/event-stream") {
		t.Fatalf("events content type: %q", ct)
	}
	events := streamLiveSSE(t, resp.Body)

	hello := nextLiveEvent(t, events, "hello")
	if hello.event != "hello" || !strings.Contains(hello.data, `"go:live:sfwp"`) {
		t.Fatalf("hello: %+v", hello)
	}

	// Replay is exactly the revisions after the requested cursor, id = kernel cursor.
	ev := nextLiveEvent(t, events, "replay 01BBB")
	if ev.event != "snapshot" || ev.id != "01BBB" {
		t.Fatalf("replay event: %+v", ev)
	}
	var frame contract.StreamEvent
	if err := json.Unmarshal([]byte(ev.data), &frame); err != nil {
		t.Fatal(err)
	}
	if frame.EventType != "snapshot" || frame.Cursor != "01BBB" {
		t.Fatalf("frame envelope: %+v", frame)
	}
	snap, ok := frame.Payload.(map[string]any)
	if !ok || snap["case_id"] != "case_1" {
		t.Fatalf("frame payload must be the snapshot: %v", frame.Payload)
	}

	// A live mutation arrives on the same stream, again keyed by the kernel cursor.
	h.pushRevision(t, "01CCC", "case_1")
	ev = nextLiveEvent(t, events, "live 01CCC")
	if ev.id != "01CCC" || ev.event != "snapshot" {
		t.Fatalf("live event: %+v", ev)
	}
}

func TestSSEHonoursLastEventIDHeader(t *testing.T) {
	h := newLiveHarness(t)
	h.pushRevision(t, "01AAA", "case_1")
	h.pushRevision(t, "01BBB", "case_1")
	op := h.login(t, "operator", "ignored-in-dev")

	req, err := http.NewRequest(http.MethodGet, h.ts.URL+"/api/events", nil)
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Last-Event-ID", "01AAA")
	resp, err := op.client.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	events := streamLiveSSE(t, resp.Body)
	nextLiveEvent(t, events, "hello")
	if ev := nextLiveEvent(t, events, "replay after header resume"); ev.id != "01BBB" {
		t.Fatalf("Last-Event-ID resume must replay from the header's cursor, got id %q", ev.id)
	}
}

func TestSSEResyncRequiredWhenLastPredatesRetention(t *testing.T) {
	// A small store: push three revisions, evicting the first; a client holding the evicted cursor
	// gets resync_required, not a silently gapped replay.
	feed := newFakeFeed()
	world := &fakeWorld{snaps: map[string]contract.CognitiveWorldSnapshot{}, newestID: "case_1"}
	store := projection.NewStoreWithRetention(2)
	relay := NewRelay(feed, world, store, RelayOptions{DefaultActor: ports.ActorClaim{ActorID: "operator_local", Role: "operator"}})
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go relay.Run(ctx)
	api := newAuthedServer(t, world, &fakeIntents{}, fakeTemplates{}, store, relay, Options{Heartbeat: time.Hour})
	ts := httptest.NewServer(api.Handler())
	defer ts.Close()
	op := newSessionFromServer(t, ts.URL)

	for _, c := range []string{"01AAA", "01BBB", "01CCC"} {
		world.set("case_1", snapshotFor(c, "case_1"))
		feed.push(KernelEvent{Cursor: c, Kind: "case.trace.item_activated", CaseID: "case_1", At: time.Now()})
	}
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		if store.Len() == 2 {
			break
		}
		time.Sleep(2 * time.Millisecond)
	}

	resp, err := op.get("/api/events?last=01AAA")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	events := streamLiveSSE(t, resp.Body)
	nextLiveEvent(t, events, "resync event")
	if ev := nextLiveEvent(t, events, "resync event"); ev.event != "resync_required" {
		t.Fatalf("a pre-retention cursor must trigger resync_required, got %+v", ev)
	}
	// The replay then covers everything retained, gap-free within the window.
	if ev := nextLiveEvent(t, events, "replay 01BBB"); ev.id != "01BBB" {
		t.Fatalf("replay after resync: %+v", ev)
	}
	if ev := nextLiveEvent(t, events, "replay 01CCC"); ev.id != "01CCC" {
		t.Fatalf("replay after resync: %+v", ev)
	}
}

func TestRelayCursorBookkeepingFeedsStaleness(t *testing.T) {
	h := newLiveHarness(t)
	h.pushRevision(t, "01AAA", "case_1")
	cursor, ok := h.relay.CursorForCase("case_1")
	if !ok || cursor != "01AAA" {
		t.Fatalf("CursorForCase: %q %v", cursor, ok)
	}
	if got := h.relay.WaitForCaseAdvance(context.Background(), "case_1", ""); got != "01AAA" {
		t.Fatalf("WaitForCaseAdvance with no advance must return the current cursor, got %q", got)
	}
	// An advance unblocks a waiting waiter.
	go func() {
		time.Sleep(10 * time.Millisecond)
		h.pushRevision(t, "01BBB", "case_1")
	}()
	if got := h.relay.WaitForCaseAdvance(context.Background(), "case_1", "01AAA"); got != "01BBB" {
		t.Fatalf("WaitForCaseAdvance must observe the advance, got %q", got)
	}
}

func TestRelayDeduplicatesAndKeepsMonotonicCursors(t *testing.T) {
	h := newLiveHarness(t)
	h.pushRevision(t, "01AAA", "case_1")
	// A duplicate frame (at-least-once delivery) must not store a second revision.
	h.feed.push(KernelEvent{Cursor: "01AAA", Kind: "case.trace.item_activated", CaseID: "case_1", At: time.Now()})
	time.Sleep(20 * time.Millisecond)
	if h.store.Len() != 1 {
		t.Fatalf("duplicate frames must not duplicate revisions, len = %d", h.store.Len())
	}
}
