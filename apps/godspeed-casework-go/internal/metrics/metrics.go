// Package metrics is the gateway's tiny, dependency-free Prometheus text exposition (plan T11).
//
// It exposes exactly the operational signals the plan names and nothing that can identify a user
// or a case: request latency by route PATTERN and status class, SFWP errors by class and verb,
// the number of open SSE clients, and subscription lag (how many retained kernel revisions each
// SSE client has not been sent yet, plus how long delivered revisions waited). Every label value
// comes from a closed set (route patterns, error kinds, wire verbs), so cardinality is bounded.
//
// All Registry methods are safe on a nil receiver, so call sites need no feature checks.
package metrics

import (
	"fmt"
	"io"
	"net"
	"net/http"
	"sort"
	"strings"
	"sync"
	"time"
)

var latencyBuckets = []float64{0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10}

type histogram struct {
	counts []uint64 // per bucket (non-cumulative); the last slot is +Inf
	sum    float64
	count  uint64
}

func newHistogram() *histogram { return &histogram{counts: make([]uint64, len(latencyBuckets)+1)} }

func (h *histogram) observe(v float64) {
	i := sort.SearchFloat64s(latencyBuckets, v)
	h.counts[i]++
	h.sum += v
	h.count++
}

func (h *histogram) write(w io.Writer, name, labels string) {
	var cum uint64
	for i, le := range latencyBuckets {
		cum += h.counts[i]
		fmt.Fprintf(w, "%s_bucket{%sle=\"%g\"} %d\n", name, labels, le, cum)
	}
	cum += h.counts[len(latencyBuckets)]
	fmt.Fprintf(w, "%s_bucket{%sle=\"+Inf\"} %d\n", name, labels, cum)
	trimmed := strings.TrimSuffix(labels, ",")
	if trimmed != "" {
		trimmed = "{" + trimmed + "}"
	}
	fmt.Fprintf(w, "%s_sum%s %g\n", name, trimmed, h.sum)
	fmt.Fprintf(w, "%s_count%s %d\n", name, trimmed, h.count)
}

// SSEClient is one open event stream's accounting handle.
type SSEClient struct {
	reg     *Registry
	id      int
	pending func() int
	mu      sync.Mutex
	last    string
}

// Registry holds every metric.
type Registry struct {
	mu          sync.Mutex
	requests    map[string]*histogram // key "route\x00class"
	sfwpErrors  map[string]uint64     // key "class\x00verb"
	deliveryLag *histogram
	sseTotal    uint64
	sse         map[int]*SSEClient
	sseSeq      int
	behind      func(cursor string) int
}

// New builds an empty registry.
func New() *Registry {
	return &Registry{
		requests:    map[string]*histogram{},
		sfwpErrors:  map[string]uint64{},
		deliveryLag: newHistogram(),
		sse:         map[int]*SSEClient{},
	}
}

// SetBehindSource installs the function that counts retained kernel revisions newer than a cursor
// (the store's CountAfter). Without it the cursor-lag gauges are omitted rather than faked.
func (r *Registry) SetBehindSource(f func(cursor string) int) {
	if r == nil {
		return
	}
	r.mu.Lock()
	r.behind = f
	r.mu.Unlock()
}

// ObserveRequest records one finished request. route is the mux pattern ("" = unmatched).
func (r *Registry) ObserveRequest(route string, status int, d time.Duration) {
	if r == nil {
		return
	}
	if route == "" {
		route = "unmatched"
	}
	key := route + "\x00" + fmt.Sprintf("%dxx", status/100)
	r.mu.Lock()
	h := r.requests[key]
	if h == nil {
		h = newHistogram()
		r.requests[key] = h
	}
	h.observe(d.Seconds())
	r.mu.Unlock()
}

// ObserveSFWPError counts one failed SFWP call by error class (apperr kind, or "canceled") and verb.
func (r *Registry) ObserveSFWPError(verb, class string) {
	if r == nil {
		return
	}
	r.mu.Lock()
	r.sfwpErrors[class+"\x00"+verb]++
	r.mu.Unlock()
}

// SSEOpen registers an open stream. startCursor is the position the client resumed from;
// pending reports the revisions already queued for it but not yet written.
func (r *Registry) SSEOpen(startCursor string, pending func() int) *SSEClient {
	if r == nil {
		return nil
	}
	c := &SSEClient{reg: r, pending: pending, last: startCursor}
	r.mu.Lock()
	r.sseSeq++
	c.id = r.sseSeq
	r.sse[c.id] = c
	r.sseTotal++
	r.mu.Unlock()
	return c
}

// Delivered records that the revision with this cursor was written to the client; recordedAt is
// when the gateway recorded it, so now-recordedAt is the delivery lag.
func (c *SSEClient) Delivered(cursor string, recordedAt time.Time) {
	if c == nil {
		return
	}
	c.mu.Lock()
	c.last = cursor
	c.mu.Unlock()
	lag := time.Since(recordedAt).Seconds()
	if lag < 0 {
		lag = 0
	}
	c.reg.mu.Lock()
	c.reg.deliveryLag.observe(lag)
	c.reg.mu.Unlock()
}

// Close unregisters the stream.
func (c *SSEClient) Close() {
	if c == nil {
		return
	}
	c.reg.mu.Lock()
	delete(c.reg.sse, c.id)
	c.reg.mu.Unlock()
}

// Write renders the Prometheus text exposition format (version 0.0.4).
func (r *Registry) Write(w io.Writer) {
	r.mu.Lock()
	clients := make([]*SSEClient, 0, len(r.sse))
	for _, c := range r.sse {
		clients = append(clients, c)
	}
	behind := r.behind
	reqKeys := sortedKeys(r.requests)
	errKeys := sortedKeys(r.sfwpErrors)

	fmt.Fprintln(w, "# HELP casework_http_request_duration_seconds Request latency by route pattern and status class (SSE streams excluded).")
	fmt.Fprintln(w, "# TYPE casework_http_request_duration_seconds histogram")
	for _, k := range reqKeys {
		route, class, _ := strings.Cut(k, "\x00")
		r.requests[k].write(w, "casework_http_request_duration_seconds", fmt.Sprintf("route=%q,status_class=%q,", route, class))
	}
	fmt.Fprintln(w, "# HELP casework_sfwp_errors_total Failed SFWP calls by error class (apperr kind) and wire verb.")
	fmt.Fprintln(w, "# TYPE casework_sfwp_errors_total counter")
	for _, k := range errKeys {
		class, verb, _ := strings.Cut(k, "\x00")
		fmt.Fprintf(w, "casework_sfwp_errors_total{class=%q,verb=%q} %d\n", class, verb, r.sfwpErrors[k])
	}
	fmt.Fprintln(w, "# HELP casework_sse_delivery_lag_seconds Time between the gateway recording a revision and writing it to an SSE client.")
	fmt.Fprintln(w, "# TYPE casework_sse_delivery_lag_seconds histogram")
	r.deliveryLag.write(w, "casework_sse_delivery_lag_seconds", "")
	sseTotal := r.sseTotal
	r.mu.Unlock()

	fmt.Fprintln(w, "# HELP casework_sse_clients Open SSE event streams.")
	fmt.Fprintln(w, "# TYPE casework_sse_clients gauge")
	fmt.Fprintf(w, "casework_sse_clients %d\n", len(clients))
	fmt.Fprintln(w, "# HELP casework_sse_clients_opened_total SSE streams opened since start.")
	fmt.Fprintln(w, "# TYPE casework_sse_clients_opened_total counter")
	fmt.Fprintf(w, "casework_sse_clients_opened_total %d\n", sseTotal)

	maxQueued, maxBehind := 0, 0
	for _, c := range clients {
		if c.pending != nil {
			if q := c.pending(); q > maxQueued {
				maxQueued = q
			}
		}
		if behind != nil {
			c.mu.Lock()
			last := c.last
			c.mu.Unlock()
			if b := behind(last); b > maxBehind {
				maxBehind = b
			}
		}
	}
	fmt.Fprintln(w, "# HELP casework_sse_queue_depth_max Largest per-client count of revisions queued but not yet written.")
	fmt.Fprintln(w, "# TYPE casework_sse_queue_depth_max gauge")
	fmt.Fprintf(w, "casework_sse_queue_depth_max %d\n", maxQueued)
	if behind != nil {
		fmt.Fprintln(w, "# HELP casework_sse_cursor_lag_max Largest count of retained kernel revisions newer than the last cursor delivered to any open client.")
		fmt.Fprintln(w, "# TYPE casework_sse_cursor_lag_max gauge")
		fmt.Fprintf(w, "casework_sse_cursor_lag_max %d\n", maxBehind)
	}
}

func sortedKeys[V any](m map[string]V) []string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return keys
}

// Handler serves the exposition. It carries no authentication of its own: it is meant for a
// separate loopback-only listener (see ValidateListenAddr), never the browser-facing mux.
func (r *Registry) Handler() http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		if req.Method != http.MethodGet && req.Method != http.MethodHead {
			w.Header().Set("Allow", "GET, HEAD")
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		w.Header().Set("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
		w.Header().Set("Cache-Control", "no-store")
		if req.Method == http.MethodGet {
			r.Write(w)
		}
	})
}

// ServeMux wraps Handler so only /metrics answers; everything else is a 404.
func (r *Registry) ServeMux() http.Handler {
	mux := http.NewServeMux()
	mux.Handle("/metrics", r.Handler())
	return mux
}

// ValidateListenAddr refuses a metrics listener that is not loopback: the exposition is
// unauthenticated, so reaching it from another host must go through an operator-controlled proxy.
func ValidateListenAddr(addr string) error {
	host, _, err := net.SplitHostPort(addr)
	if err != nil {
		return fmt.Errorf("metrics address %q: %w", addr, err)
	}
	if host == "localhost" {
		return nil
	}
	ip := net.ParseIP(host)
	if ip == nil || !ip.IsLoopback() {
		return fmt.Errorf("metrics address %q must be a loopback host (127.0.0.1, ::1 or localhost): the endpoint is unauthenticated", addr)
	}
	return nil
}
