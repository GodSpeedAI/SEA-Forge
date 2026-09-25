// The live server's HTTP+SSE tests over fakes and a real relay + revision store: the served
// shapes, the cursor-keyed history with its documented 404, template endpoints, intent envelope
// pass-through, and the SSE replay/resume contract (Last-Event-ID AND ?last=) with no gaps.
package server

import (
	"bufio"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"

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

// fakeIntents records the last intent and answers a canned response.
type fakeIntents struct {
	mu   sync.Mutex
	last contract.InteractionIntent
	resp contract.IntentResponse
}

func (f *fakeIntents) Handle(ctx context.Context, in contract.InteractionIntent) contract.IntentResponse {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.last = in
	return f.resp
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

// liveHarness assembles the live server over fakes plus a real relay + store driven by a fake feed.
type liveHarness struct {
	feed   *fakeFeed
	world  *fakeWorld
	ints   *fakeIntents
	store  *projection.Store
	relay  *Relay
	ts     *httptest.Server
	cancel func()
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
	api := New(world, ints, fakeTemplates{}, store, relay, Options{Heartbeat: time.Hour})
	ts := httptest.NewServer(api.Handler())
	t.Cleanup(func() { cancel(); ts.Close() })
	return &liveHarness{feed: feed, world: world, ints: ints, store: store, relay: relay, ts: ts, cancel: cancel}
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

	resp, err := http.Get(h.ts.URL + "/api/world")
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
}

func TestWorldServesKernelHistoryAtCursorAnd404sEvicted(t *testing.T) {
	h := newLiveHarness(t)
	h.pushRevision(t, "01AAA", "case_1")
	h.pushRevision(t, "01BBB", "case_1")

	resp, err := http.Get(h.ts.URL + "/api/world?cursor=01AAA")
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

	resp2, err := http.Get(h.ts.URL + "/api/world?cursor=01ZZZ")
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
	resp, err := http.Get(h.ts.URL + "/api/world")
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

func TestWorldExplicitPerspectiveIsVerified(t *testing.T) {
	h := newLiveHarness(t)
	h.world.verify = func(a ports.ActorClaim) error { return errDenied{} }
	resp, err := http.Get(h.ts.URL + "/api/world?actor=operator_local&role=operator")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusForbidden {
		t.Fatalf("a kernel-refused perspective must be 403, got %d", resp.StatusCode)
	}
	resp2, _ := http.Get(h.ts.URL + "/api/world?actor=solo")
	if resp2 != nil {
		resp2.Body.Close()
		if resp2.StatusCode != http.StatusBadRequest {
			t.Fatalf("a half-named actor must be 400, got %d", resp2.StatusCode)
		}
	}
}

type errDenied struct{}

func (errDenied) Error() string { return "refused" }

func TestIntentsPassthroughAndStrictness(t *testing.T) {
	h := newLiveHarness(t)
	body := `{"intent_id":"i-1","kind":"CONSEQUENTIAL_CASE","action_name":"EXECUTE_ITEM","target_object_id":"item-1","case_id":"case_1","client_cursor":"01AAA","actor":{"actor_id":"operator_local","role":"operator"},"parameters":{"item_id":"item-1"}}`
	resp, err := http.Post(h.ts.URL+"/api/intents", "application/json", strings.NewReader(body))
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
	if h.ints.last.Actor.Role != "operator" || h.ints.last.ClientCursor != "01AAA" {
		t.Fatalf("the intent envelope must pass through intact: %+v", h.ints.last)
	}

	// Unknown fields are refused at the envelope (the contract is the wire).
	bad := strings.Replace(body, `"parameters":`, `"surprise":1,"parameters":`, 1)
	resp2, err := http.Post(h.ts.URL+"/api/intents", "application/json", strings.NewReader(bad))
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
	resp, err := http.Get(h.ts.URL + "/api/templates")
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

	resp2, err := http.Post(h.ts.URL+"/api/templates/preflight", "application/json",
		strings.NewReader(`{"template_ref":"e2e-sentry-chain@0.1.0","params":{"dataset_name":"orders"}}`))
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

	resp, err := http.Get(h.ts.URL + "/api/events?last=01AAA")
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

	req, err := http.NewRequest(http.MethodGet, h.ts.URL+"/api/events", nil)
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Last-Event-ID", "01AAA")
	resp, err := http.DefaultClient.Do(req)
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
	relay := NewRelay(feed, world, store, RelayOptions{DefaultActor: ports.ActorClaim{ActorID: "op", Role: "operator"}})
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go relay.Run(ctx)
	api := New(world, &fakeIntents{}, fakeTemplates{}, store, relay, Options{Heartbeat: time.Hour})
	ts := httptest.NewServer(api.Handler())
	defer ts.Close()

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

	resp, err := http.Get(ts.URL + "/api/events?last=01AAA")
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
