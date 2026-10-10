package server

import (
	"bytes"
	"context"
	"log"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/metrics"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

type lockedBuf struct {
	mu sync.Mutex
	b  bytes.Buffer
}

func (l *lockedBuf) Write(p []byte) (int, error) {
	l.mu.Lock()
	defer l.mu.Unlock()
	return l.b.Write(p)
}
func (l *lockedBuf) String() string { l.mu.Lock(); defer l.mu.Unlock(); return l.b.String() }

func newMetricsHarness(t *testing.T) (*liveHarness, *metrics.Registry, *lockedBuf) {
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
	reg := metrics.New()
	logs := &lockedBuf{}
	api := newAuthedServer(t, world, ints, fakeTemplates{}, store, relay, Options{
		Heartbeat: time.Hour, Metrics: reg, Logger: log.New(logs, "", 0),
	})
	ts := httptest.NewServer(api.Handler())
	t.Cleanup(func() { cancel(); ts.Close() })
	return &liveHarness{feed: feed, world: world, ints: ints, store: store, relay: relay, ts: ts, cancel: cancel, api: api}, reg, logs
}

func scrape(reg *metrics.Registry) string {
	rec := httptest.NewRecorder()
	reg.Handler().ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/metrics", nil))
	return rec.Body.String()
}

func TestMetricsRequestLatencyBySessionRoutePatternNotRawPath(t *testing.T) {
	h, reg, _ := newMetricsHarness(t)
	op := h.login(t, "operator", "ignored-in-dev")
	for _, p := range []string{"/api/templates", "/api/no-such-route/abc-123"} {
		resp, err := op.get(p)
		if err != nil {
			t.Fatal(err)
		}
		resp.Body.Close()
	}
	out := scrape(reg)
	if !strings.Contains(out, `route="GET /api/templates",status_class="2xx"`) {
		t.Fatalf("missing templates histogram:\n%s", out)
	}
	if !strings.Contains(out, `route="GET /api/",status_class="4xx"`) {
		t.Fatalf("unknown /api path must be counted under its pattern:\n%s", out)
	}
	if strings.Contains(out, "abc-123") {
		t.Fatalf("raw paths must never become label values:\n%s", out)
	}
}

func TestMetricsSSEClientsGaugeAndCursorLag(t *testing.T) {
	h, reg, _ := newMetricsHarness(t)
	h.pushRevision(t, "01AAA", "case_1")
	h.pushRevision(t, "01BBB", "case_1")
	op := h.login(t, "operator", "ignored-in-dev")

	resp, err := op.get("/api/events?last=01AAA")
	if err != nil {
		t.Fatal(err)
	}
	events := streamLiveSSE(t, resp.Body)
	nextLiveEvent(t, events, "hello")
	nextLiveEvent(t, events, "replay 01BBB")

	// The delivered revision is accounted after the write; poll briefly for the gauge.
	deadline := time.Now().Add(2 * time.Second)
	var out string
	for time.Now().Before(deadline) {
		out = scrape(reg)
		if strings.Contains(out, "casework_sse_clients 1\n") && strings.Contains(out, "casework_sse_cursor_lag_max 0\n") {
			break
		}
		time.Sleep(10 * time.Millisecond)
	}
	if !strings.Contains(out, "casework_sse_clients 1\n") || !strings.Contains(out, "casework_sse_cursor_lag_max 0\n") {
		t.Fatalf("one caught-up client expected:\n%s", out)
	}
	if !strings.Contains(out, "casework_sse_delivery_lag_seconds_count 1\n") {
		t.Fatalf("one delivery expected:\n%s", out)
	}
	// A revision appended while the client reads is delivered, so lag returns to zero.
	h.pushRevision(t, "01CCC", "case_1")
	nextLiveEvent(t, events, "live 01CCC")

	resp.Body.Close()
	deadline = time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) && !strings.Contains(scrape(reg), "casework_sse_clients 0\n") {
		time.Sleep(10 * time.Millisecond)
	}
	if out = scrape(reg); !strings.Contains(out, "casework_sse_clients 0\n") {
		t.Fatalf("the gauge must drop when the client leaves:\n%s", out)
	}
}

func TestMetricsExposureIsNotOnTheBrowserMux(t *testing.T) {
	h, _, _ := newMetricsHarness(t)
	for _, p := range []string{"/metrics", "/api/metrics"} {
		resp, err := http.Get(h.ts.URL + p)
		if err != nil {
			t.Fatal(err)
		}
		resp.Body.Close()
		if resp.StatusCode == http.StatusOK {
			t.Fatalf("%s must not be served by the browser-facing gateway mux", p)
		}
	}
}

func TestCorrelationIDEchoedAndEqualsIntentIDInLog(t *testing.T) {
	h, _, logs := newMetricsHarness(t)
	op := h.login(t, "operator", "ignored-in-dev")
	body := `{"intent_id":"corr-intent-42","kind":"CONSEQUENTIAL_CASE","action_name":"EXECUTE_ITEM","target_object_id":"item-1","case_id":"case_1","client_cursor":"01AAA","parameters":{"item_id":"item-1"}}`
	resp, err := op.post("/api/intents", "application/json", body)
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	if got := resp.Header.Get("X-Correlation-Id"); got != "corr-intent-42" {
		t.Fatalf("X-Correlation-Id = %q, want the intent id (the SFWP request_id)", got)
	}
	if !strings.Contains(logs.String(), `correlation_id="corr-intent-42" method="POST" path="/api/intents"`) {
		t.Fatalf("gateway log lacks the intent correlation line:\n%s", logs.String())
	}
	// A non-intent request gets a minted id, also echoed.
	r2, err := op.get("/api/templates")
	if err != nil {
		t.Fatal(err)
	}
	r2.Body.Close()
	if r2.Header.Get("X-Correlation-Id") == "" {
		t.Fatal("every response must carry X-Correlation-Id")
	}
}
