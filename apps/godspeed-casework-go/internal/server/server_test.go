package server

import (
	"bufio"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/artifactstore"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/coordinator"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// harness wires a real server over the fixture with instant staged timings.
type harness struct {
	proj  *projection.Store
	arts  *artifactstore.Store
	coord *coordinator.Coordinator
	ts    *httptest.Server
}

func newHarness(t *testing.T) *harness {
	t.Helper()
	return newHarnessWithOptions(t, Options{})
}

func newHarnessWithOptions(t *testing.T, opts Options) *harness {
	t.Helper()
	ds, err := projection.Fixture()
	if err != nil {
		t.Fatalf("decode embedded fixture: %v", err)
	}
	proj, err := projection.NewStore(ds)
	if err != nil {
		t.Fatalf("build store: %v", err)
	}
	arts := artifactstore.New(ds.Artifacts)
	coord := coordinator.New(proj, arts, coordinator.Options{
		Sleep:  func(time.Duration) {},
		Stage1: time.Millisecond,
		Stage2: time.Millisecond,
	})
	ts := httptest.NewServer(New(proj, coord, arts, opts).Handler())
	t.Cleanup(ts.Close)
	return &harness{proj: proj, arts: arts, coord: coord, ts: ts}
}

func (h *harness) post(t *testing.T, path, body string) (int, string) {
	t.Helper()
	resp, err := http.Post(h.ts.URL+path, "application/json", strings.NewReader(body))
	if err != nil {
		t.Fatalf("POST %s: %v", path, err)
	}
	defer resp.Body.Close()
	raw, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(raw)
}

func (h *harness) postRaw(t *testing.T, path, contentType, body string) (int, string) {
	t.Helper()
	resp, err := http.Post(h.ts.URL+path, contentType, strings.NewReader(body))
	if err != nil {
		t.Fatalf("POST %s: %v", path, err)
	}
	defer resp.Body.Close()
	raw, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(raw)
}

func get(t *testing.T, url string) (int, string, string) {
	t.Helper()
	resp, err := http.Get(url)
	if err != nil {
		t.Fatalf("GET %s: %v", url, err)
	}
	defer resp.Body.Close()
	raw, _ := io.ReadAll(resp.Body)
	return resp.StatusCode, string(raw), resp.Header.Get("Content-Type")
}

func decode(t *testing.T, body string, v any) {
	t.Helper()
	if err := json.Unmarshal([]byte(body), v); err != nil {
		t.Fatalf("decode %q into %T: %v", body, v, err)
	}
}

func TestHealthz(t *testing.T) {
	h := newHarness(t)
	status, body, ct := get(t, h.ts.URL+"/api/healthz")
	if status != 200 || !strings.Contains(ct, "application/json") {
		t.Fatalf("healthz: %d %s", status, ct)
	}
	var got struct {
		Status     string `json:"status"`
		Provenance string `json:"provenance"`
		LiveCursor int64  `json:"liveCursor"`
	}
	decode(t, body, &got)
	if got.Status != "ok" || got.Provenance != "go:fixture:northstar" || got.LiveCursor != 1150 {
		t.Fatalf("healthz body: %s", body)
	}
}

func TestWorldLiveAtUnknownAndMalformed(t *testing.T) {
	h := newHarness(t)

	status, body, _ := get(t, h.ts.URL+"/api/world")
	if status != 200 {
		t.Fatalf("live world: %d", status)
	}
	var live struct {
		Snapshot projection.Snapshot `json:"snapshot"`
	}
	decode(t, body, &live)
	if live.Snapshot.Cursor != 1150 || live.Snapshot.Provenance != "go:fixture:northstar" {
		t.Fatalf("live world body: cursor=%d provenance=%q", live.Snapshot.Cursor, live.Snapshot.Provenance)
	}
	if len(live.Snapshot.Objects) == 0 || len(live.Snapshot.Surfaces) == 0 {
		t.Fatal("live snapshot must carry objects and surfaces")
	}

	status, body, _ = get(t, h.ts.URL+"/api/world?cursor=900")
	if status != 200 {
		t.Fatalf("world at 900: %d", status)
	}
	var at900 struct {
		Snapshot projection.Snapshot `json:"snapshot"`
	}
	decode(t, body, &at900)
	if at900.Snapshot.Cursor != 900 {
		t.Fatalf("world at 900 returned cursor %d", at900.Snapshot.Cursor)
	}

	status, body, _ = get(t, h.ts.URL+"/api/world?cursor=901")
	if status != 404 {
		t.Fatalf("unknown cursor must 404, got %d", status)
	}
	var e errorBody
	decode(t, body, &e)
	if e.Error.Kind != "invalid" || e.Error.Note != "unknown cursor" {
		t.Fatalf("unknown cursor error body: %s", body)
	}

	if status, _, _ = get(t, h.ts.URL+"/api/world?cursor=abc"); status != 404 {
		t.Fatalf("malformed cursor must 404, got %d", status)
	}
}

func TestTimeEndpointListsAscendingPositionsAndGrows(t *testing.T) {
	h := newHarness(t)
	status, body, _ := get(t, h.ts.URL+"/api/time")
	if status != 200 {
		t.Fatalf("time: %d", status)
	}
	var got timeBody
	decode(t, body, &got)
	if len(got.Positions) != 7 || got.Positions[0].Cursor != 900 || got.Positions[6].Cursor != 1150 {
		t.Fatalf("time positions: %s", body)
	}
	if got.Positions[0].At != "2026-09-15T09:00:00Z" || got.Positions[0].Summary == "" {
		t.Fatalf("time position shape: %+v", got.Positions[0])
	}
	if got.Truncated {
		t.Fatal("the full history is served; truncated must be false")
	}

	// A consequential append grows the window: the time feed tracks the live world.
	code, respBody := h.post(t, "/api/intents", `{"id":"time-1","kind":"propose-consequence","target":"ns-migration","parameters":{"action":"implement","cursor":1150}}`)
	if code != 200 || !strings.Contains(respBody, `"accepted"`) {
		t.Fatalf("intent: %d %s", code, respBody)
	}
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		_, body, _ = get(t, h.ts.URL+"/api/time")
		decode(t, body, &got)
		if len(got.Positions) == 10 {
			return
		}
		time.Sleep(5 * time.Millisecond)
	}
	t.Fatalf("positions did not grow to 10 after the consequential flow: %d", len(got.Positions))
}

func TestArtifactCatalogMatchesFixtureAndGrowsOnPersist(t *testing.T) {
	h := newHarness(t)
	status, body, _ := get(t, h.ts.URL+"/api/artifacts")
	if status != 200 {
		t.Fatalf("artifact list: %d", status)
	}
	var got struct {
		Descriptors []artifactDescriptor `json:"descriptors"`
	}
	decode(t, body, &got)
	if len(got.Descriptors) != 9 {
		t.Fatalf("artifact catalog: got %d descriptors, want the fixture's 9 (%s)", len(got.Descriptors), body)
	}
	found := map[string]artifactDescriptor{}
	for _, d := range got.Descriptors {
		found[d.Ref] = d
	}
	if q := found["art-interview-quote"]; q.Kind != "quotation" || q.BoundObject != "ns-interview" || q.Title != "Interview - clinical ops lead" {
		t.Fatalf("descriptor shape drifted: %+v", q)
	}

	h.post(t, "/api/artifacts", `{"ref":"art-live-note","boundObject":"ns-workflow","title":"Live note"}`)
	_, body, _ = get(t, h.ts.URL+"/api/artifacts")
	decode(t, body, &got)
	if len(got.Descriptors) != 10 {
		t.Fatalf("catalog must grow after a persist: %d", len(got.Descriptors))
	}
}

func TestArtifactBytesAreLevelAware(t *testing.T) {
	h := newHarness(t)

	status, body, ct := get(t, h.ts.URL+"/api/artifacts/art-interview-quote?level=minimal")
	if status != 200 || ct != "text/plain" || body != "Secondary coverage is not optional." {
		t.Fatalf("minimal level: %d %q %q", status, ct, body)
	}
	status, body, ct = get(t, h.ts.URL+"/api/artifacts/art-workflow-fragment?level=summary")
	if status != 200 || ct != "text/html" || !strings.Contains(body, "external record lookup") {
		t.Fatalf("summary level: %d %q", status, ct)
	}
	// Default level is minimal.
	status, body, ct = get(t, h.ts.URL+"/api/artifacts/art-interview-quote")
	if status != 200 || body != "Secondary coverage is not optional." {
		t.Fatalf("default level: %d %q", status, body)
	}
	// Binary artifact levels decode to their bytes and media type.
	status, raw, ct := get(t, h.ts.URL+"/api/artifacts/art-pilot-report?level=source")
	if status != 200 || ct != "application/pdf" || !strings.HasPrefix(raw, "%PDF") {
		t.Fatalf("pdf level: %d %q %q", status, ct, raw[:min(8, len(raw))])
	}
}

func TestArtifactUnknownRefAndBadLevel(t *testing.T) {
	h := newHarness(t)
	status, body, _ := get(t, h.ts.URL+"/api/artifacts/art-nope")
	if status != 404 {
		t.Fatalf("unknown ref must 404, got %d", status)
	}
	var e errorBody
	decode(t, body, &e)
	if e.Error.Kind != "invalid" || e.Error.Note != "unknown artifact ref" {
		t.Fatalf("unknown ref body: %s", body)
	}
	if status, _, _ = get(t, h.ts.URL+"/api/artifacts/art-interview-quote?level=verbose"); status != 400 {
		t.Fatalf("unknown level must 400, got %d", status)
	}
}

// Durability, pinned at the boundary: an artifact persisted over HTTP is served afterwards, with
// honest placeholder levels, for the life of the process (the test server).
func TestArtifactPersistDurability(t *testing.T) {
	h := newHarness(t)
	code, body := h.post(t, "/api/artifacts", `{"ref":"art-durable","boundObject":"ns-validation","title":"Durable note"}`)
	if code != 200 || !strings.Contains(body, `"status":"accepted"`) {
		t.Fatalf("persist: %d %s", code, body)
	}
	// Re-persist is accepted and idempotent.
	code, _ = h.post(t, "/api/artifacts", `{"ref":"art-durable","boundObject":"ns-validation","title":"Durable note"}`)
	if code != 200 {
		t.Fatalf("re-persist: %d", code)
	}
	// Intervening traffic must not evict it.
	h.post(t, "/api/intents", `{"id":"dur-1","kind":"focus-object","target":"ns-goal"}`)
	status, body, ct := get(t, h.ts.URL+"/api/artifacts/art-durable?level=source")
	if status != 200 || ct != "text/plain" || !strings.Contains(body, "not governed persistence") {
		t.Fatalf("durable artifact: %d %q %q", status, ct, body)
	}

	// A refusal for a body without a ref is a normal outcome, not a transport error.
	code, body = h.post(t, "/api/artifacts", `{"boundObject":"ns-validation"}`)
	if code != 200 || !strings.Contains(body, `"reason":"invalid"`) {
		t.Fatalf("refless persist: %d %s", code, body)
	}
}

func TestIntentEndpointValidationAndIdempotency(t *testing.T) {
	h := newHarness(t)

	// Happy path over HTTP with the response shape the wire specifies.
	code, body := h.post(t, "/api/intents", `{"id":"http-1","kind":"propose-consequence","target":"ns-migration","parameters":{"action":"implement","approach":"compat-layer","cursor":1150}}`)
	if code != 200 {
		t.Fatalf("accepted intent http: %d %s", code, body)
	}
	var accepted coordinator.Outcome
	decode(t, body, &accepted)
	if accepted.Status != "accepted" || accepted.Lease == nil || accepted.Lease.State != "active" {
		t.Fatalf("accepted shape: %s", body)
	}

	// Idempotent replay over HTTP returns the identical body.
	code, replay := h.post(t, "/api/intents", `{ "parameters" : {"cursor":1150,"approach":"compat-layer","action":"implement"}, "target":"ns-migration", "kind":"propose-consequence", "id":"http-1" }`)
	if code != 200 || replay != body {
		t.Fatalf("replay must be byte-stable across key order:\n%s\n%s", body, replay)
	}

	// Same id, different body.
	code, body = h.post(t, "/api/intents", `{"id":"http-1","kind":"propose-consequence","target":"ns-secondary","parameters":{"action":"implement"}}`)
	if code != 200 || !strings.Contains(body, `"reason":"invalid"`) {
		t.Fatalf("conflicting id: %d %s", code, body)
	}

	// Stale cursor.
	code, body = h.post(t, "/api/intents", `{"id":"http-2","kind":"propose-consequence","target":"ns-migration","parameters":{"action":"implement","cursor":900}}`)
	if code != 200 || !strings.Contains(body, `"reason":"stale_projection"`) {
		t.Fatalf("stale: %d %s", code, body)
	}

	// Authority denial over HTTP leaves the world untouched.
	code, body = h.post(t, "/api/intents", `{"id":"http-3","kind":"decide-approval","target":"ns-pr-491"}`)
	if code != 200 || !strings.Contains(body, `"reason":"authority_denied"`) {
		t.Fatalf("decide-approval: %d %s", code, body)
	}

	// Unknown kind is a normal refusal.
	code, body = h.post(t, "/api/intents", `{"id":"http-4","kind":"teleport","target":"ns-goal"}`)
	if code != 200 || !strings.Contains(body, `"reason":"invalid"`) {
		t.Fatalf("unknown kind: %d %s", code, body)
	}
}

func TestIntentEndpointTransportStrictness(t *testing.T) {
	h := newHarness(t)

	// Wrong content type.
	code, body := h.postRaw(t, "/api/intents", "text/plain", `{"id":"t1","kind":"focus-object","target":"ns-goal"}`)
	if code != 415 {
		t.Fatalf("content type: %d %s", code, body)
	}
	code, _ = h.postRaw(t, "/api/artifacts", "", `{"ref":"x"}`)
	if code != 415 {
		t.Fatalf("artifact content type: %d", code)
	}

	// Unknown top-level field.
	code, body = h.post(t, "/api/intents", `{"id":"t2","kind":"focus-object","target":"ns-goal","surprise":1}`)
	if code != 400 || !strings.Contains(body, `"kind":"invalid"`) {
		t.Fatalf("unknown field: %d %s", code, body)
	}

	// Trailing second JSON value.
	code, _ = h.post(t, "/api/intents", `{"id":"t3","kind":"focus-object","target":"ns-goal"} {"id":"t4"}`)
	if code != 400 {
		t.Fatalf("trailing value: %d", code)
	}

	// Malformed JSON.
	code, _ = h.post(t, "/api/intents", `{nope`)
	if code != 400 {
		t.Fatalf("malformed json: %d", code)
	}

	// Oversized body (> 1 MiB).
	big := `{"id":"t5","kind":"focus-object","target":"` + strings.Repeat("x", 1<<20) + `"}`
	code, _ = h.post(t, "/api/intents", big)
	if code != 413 {
		t.Fatalf("oversized body: %d", code)
	}
}

// --- SSE ---

type sseEvent struct {
	id    string
	event string
	data  string
}

// streamSSE reads the event stream into a channel, preserving arrival order.
func streamSSE(t *testing.T, body io.Reader) <-chan sseEvent {
	t.Helper()
	ch := make(chan sseEvent, 256)
	go func() {
		defer close(ch)
		reader := bufio.NewReader(body)
		var ev sseEvent
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
				ch <- sseEvent{event: "comment", data: strings.TrimSuffix(strings.TrimPrefix(line, ":"), "\n")}
			case line == "\n" || line == "":
				if ev.event != "" || ev.data != "" {
					ch <- ev
					ev = sseEvent{}
				}
				if line == "" && err != nil {
					return
				}
			}
			if err != nil {
				return
			}
		}
	}()
	return ch
}

func nextEvent(t *testing.T, ch <-chan sseEvent, what string) sseEvent {
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
	return sseEvent{}
}

func TestSSEHelloReplayLiveAndLeases(t *testing.T) {
	h := newHarness(t)

	resp, err := http.Get(h.ts.URL + "/api/events?last=1070")
	if err != nil {
		t.Fatalf("open events: %v", err)
	}
	defer resp.Body.Close()
	if ct := resp.Header.Get("Content-Type"); !strings.HasPrefix(ct, "text/event-stream") {
		t.Fatalf("events content type: %q", ct)
	}
	events := streamSSE(t, resp.Body)

	hello := nextEvent(t, events, "hello")
	if hello.event != "hello" || !strings.Contains(hello.data, `"provenance":"go:fixture:northstar"`) || !strings.Contains(hello.data, `"liveCursor":1150`) {
		t.Fatalf("hello event: %+v", hello)
	}

	// Replay of everything newer than last=1070.
	for _, want := range []string{"1110", "1150"} {
		ev := nextEvent(t, events, "replay "+want)
		if ev.event != "revision" || ev.id != want || !strings.Contains(ev.data, `"cursor":`+want) {
			t.Fatalf("replay event for %s: %+v", want, ev)
		}
	}

	// A consequential intent drives the rest of the stream live.
	code, body := h.post(t, "/api/intents", `{"id":"sse-1","kind":"propose-consequence","target":"ns-migration","parameters":{"action":"implement","cursor":1150}}`)
	if code != 200 || !strings.Contains(body, `"accepted"`) {
		t.Fatalf("intent: %d %s", code, body)
	}

	// Collect the live flow. The two feeds (revisions, leases) are ordered per feed but not
	// against each other, so the collection waits for BOTH the three staged revisions and the
	// released transition rather than breaking on whichever arrives first.
	var revisions, leases []sseEvent
	sawReleased := false
	deadline := time.After(5 * time.Second)
	for len(revisions) < 3 || !sawReleased {
		select {
		case ev := <-events:
			switch ev.event {
			case "revision":
				revisions = append(revisions, ev)
			case "lease":
				leases = append(leases, ev)
				if strings.Contains(ev.data, `"released"`) {
					sawReleased = true
				}
			}
		case <-deadline:
			t.Fatalf("timed out collecting the live flow; revisions=%d leases=%d released=%t", len(revisions), len(leases), sawReleased)
		}
	}

	if len(revisions) != 3 {
		t.Fatalf("live revisions: got %d, want 3 (1151, 1152, 1153)", len(revisions))
	}
	for i, want := range []string{"1151", "1152", "1153"} {
		if revisions[i].id != want {
			t.Fatalf("live revision order: position %d has id %q, want %q", i, revisions[i].id, want)
		}
		if !strings.Contains(revisions[i].data, `"snapshot"`) {
			t.Fatalf("revision data must wrap a snapshot: %s", revisions[i].data)
		}
	}
	if !strings.Contains(revisions[0].data, "ns-fix-layer") || !strings.Contains(revisions[0].data, `"attention":"notable"`) {
		t.Fatalf("revision 1151 must introduce the claimed compatibility layer: %s", truncateFor(revisions[0].data, 300))
	}
	if !strings.Contains(revisions[2].data, `"Pilot ready for review"`) {
		t.Fatalf("revision 1153 must quiet the world: %s", truncateFor(revisions[2].data, 300))
	}

	if len(leases) != 3 {
		t.Fatalf("lease events: got %d, want 3 (claimed, active, released): %v", len(leases), leases)
	}
	for i, want := range []string{"claimed", "active", "released"} {
		if !strings.Contains(leases[i].data, `"state":"`+want+`"`) || !strings.Contains(leases[i].data, `"summary"`) {
			t.Fatalf("lease event %d: %s", i, leases[i].data)
		}
	}
	if leases[0].id != "" || !strings.HasPrefix(extractJSONField(leases[0].data, "id"), "lease-") {
		t.Fatalf("lease events carry an id field inside data and none outside: %+v", leases[0])
	}
}

func TestSSEWithoutLastReplaysEverything(t *testing.T) {
	h := newHarness(t)
	resp, err := http.Get(h.ts.URL + "/api/events")
	if err != nil {
		t.Fatalf("open events: %v", err)
	}
	defer resp.Body.Close()
	events := streamSSE(t, resp.Body)
	nextEvent(t, events, "hello")
	for _, want := range []string{"900", "940", "980", "1030", "1070", "1110", "1150"} {
		if ev := nextEvent(t, events, "replay "+want); ev.id != want {
			t.Fatalf("full replay: got id %q, want %q", ev.id, want)
		}
	}
}

func TestSSEHeartbeatComment(t *testing.T) {
	h := newHarnessWithOptions(t, Options{Heartbeat: 20 * time.Millisecond})
	resp, err := http.Get(h.ts.URL + "/api/events")
	if err != nil {
		t.Fatalf("open events: %v", err)
	}
	defer resp.Body.Close()
	events := streamSSE(t, resp.Body)
	nextEvent(t, events, "hello")
	// Skip the replay quickly.
	for i := 0; i < 7; i++ {
		nextEvent(t, events, "replay")
	}
	ev := nextEvent(t, events, "heartbeat comment")
	if ev.event != "comment" || strings.TrimSpace(ev.data) != "heartbeat" {
		t.Fatalf("heartbeat comment: %+v", ev)
	}
}

// A client disconnect must stop the server-side work for that stream: subscriptions are
// cancelled, no goroutine keeps writing.
func TestSSEClientDisconnectCancelsSubscriptions(t *testing.T) {
	h := newHarness(t)
	ctx, cancel := context.WithCancel(context.Background())
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, h.ts.URL+"/api/events", nil)
	if err != nil {
		t.Fatalf("request: %v", err)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("open events: %v", err)
	}
	defer resp.Body.Close()
	events := streamSSE(t, resp.Body)
	nextEvent(t, events, "hello")

	if got := h.proj.SubscriberCount(); got != 1 {
		t.Fatalf("subscriber count while streaming: %d", got)
	}
	cancel()
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		if h.proj.SubscriberCount() == 0 {
			return
		}
		time.Sleep(2 * time.Millisecond)
	}
	t.Fatal("subscriptions survived the client disconnect")
}

func truncateFor(s string, n int) string {
	if len(s) <= n {
		return s
	}
	return s[:n] + "..."
}

func extractJSONField(jsonLine, field string) string {
	var m map[string]any
	if err := json.Unmarshal([]byte(jsonLine), &m); err != nil {
		return ""
	}
	v, _ := m[field].(string)
	return v
}

func TestLocalCORSAllowsColocatedUIAndRefusesOthers(t *testing.T) {
	h := newHarnessWithOptions(t, Options{Heartbeat: time.Hour})

	req, err := http.NewRequest(http.MethodGet, h.ts.URL+"/api/healthz", nil)
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Origin", "http://127.0.0.1:4178")
	rec := httptest.NewRecorder()
	New(h.proj, h.coord, h.arts, Options{}).Handler().ServeHTTP(rec, req)
	if got := rec.Header().Get("Access-Control-Allow-Origin"); got != "http://127.0.0.1:4178" {
		t.Fatalf("colocated origin not allowed: %q", got)
	}

	req2, err := http.NewRequest(http.MethodGet, h.ts.URL+"/api/healthz", nil)
	if err != nil {
		t.Fatal(err)
	}
	req2.Header.Set("Origin", "http://evil.example")
	rec2 := httptest.NewRecorder()
	New(h.proj, h.coord, h.arts, Options{}).Handler().ServeHTTP(rec2, req2)
	if got := rec2.Header().Get("Access-Control-Allow-Origin"); got != "" {
		t.Fatalf("non-local origin must not receive CORS grant, got %q", got)
	}

	req3, err := http.NewRequest(http.MethodOptions, h.ts.URL+"/api/intents", nil)
	if err != nil {
		t.Fatal(err)
	}
	req3.Header.Set("Origin", "http://localhost:4178")
	req3.Header.Set("Access-Control-Request-Method", "POST")
	rec3 := httptest.NewRecorder()
	New(h.proj, h.coord, h.arts, Options{}).Handler().ServeHTTP(rec3, req3)
	if rec3.Code != http.StatusNoContent {
		t.Fatalf("preflight expected 204, got %d", rec3.Code)
	}
}
