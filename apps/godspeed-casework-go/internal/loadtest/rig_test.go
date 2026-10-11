//go:build load

// Package loadtest is the T11 load harness: a FRESH live stack (the real sea-forge-server kernel
// and the real godspeed-casework gateway binary, each its own OS process, over a temp cell), N
// dev-auth users holding N concurrent SSE streams, and a concurrent intent burst. It measures what
// the plan's budgets are written against: intent latency, kernel-append -> client-receipt event
// lag, error rate, and gateway RSS / goroutines / file descriptors over time.
//
// Run through `just casework-load`. The scale is configurable (LOAD_CLIENTS, LOAD_INTENTS,
// LOAD_CASES); the budgets in budgets_test.go are calibrated for the default scale and are
// enforced at any scale (a larger scale is a harsher test, never a looser one).
package loadtest

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/http/cookiejar"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livetest"
)

func TestMain(m *testing.M) {
	livetest.EnsureServerBinary()
	os.Exit(m.Run())
}

func envInt(name string, def int) int {
	if v := os.Getenv(name); v != "" {
		if n, err := strconv.Atoi(v); err == nil && n > 0 {
			return n
		}
	}
	return def
}

// ---------------------------------------------------------------------------------------------
// statistics

func percentile(sorted []float64, p float64) float64 {
	if len(sorted) == 0 {
		return 0
	}
	// nearest rank
	rank := int(p/100*float64(len(sorted)) + 0.999999)
	if rank < 1 {
		rank = 1
	}
	if rank > len(sorted) {
		rank = len(sorted)
	}
	return sorted[rank-1]
}

type dist struct {
	N   int     `json:"n"`
	P50 float64 `json:"p50_ms"`
	P95 float64 `json:"p95_ms"`
	P99 float64 `json:"p99_ms"`
	Max float64 `json:"max_ms"`
}

func summarize(ms []float64) dist {
	s := append([]float64(nil), ms...)
	sort.Float64s(s)
	d := dist{N: len(s), P50: percentile(s, 50), P95: percentile(s, 95), P99: percentile(s, 99)}
	if len(s) > 0 {
		d.Max = s[len(s)-1]
	}
	return d
}

// ulidTime decodes the 48-bit millisecond timestamp of a ULID: the kernel's event cursor is the
// ledger entry ULID, minted when the kernel appended the entry, so now-ulidTime is the
// kernel-append -> receipt lag measured on one host clock.
func ulidTime(id string) (time.Time, bool) {
	const alphabet = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
	if len(id) != 26 {
		return time.Time{}, false
	}
	var ms int64
	for _, ch := range strings.ToUpper(id[:10]) {
		i := strings.IndexRune(alphabet, ch)
		if i < 0 {
			return time.Time{}, false
		}
		ms = ms<<5 | int64(i)
	}
	return time.UnixMilli(ms), true
}

// ---------------------------------------------------------------------------------------------
// process management

type rig struct {
	t        *testing.T
	cell     *livetest.Cell
	gwBin    string
	cfgFile  string
	addr     string
	metrics  string
	base     string
	users    []string
	evidence string
	label    string

	gw    *exec.Cmd
	gwLog string
	gwMu  sync.Mutex
}

func freeAddr(t *testing.T) string {
	t.Helper()
	l, err := net.Listen("tcp4", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	defer l.Close()
	return l.Addr().String()
}

func newRig(t *testing.T, nUsers int) *rig {
	t.Helper()
	r := &rig{t: t, cell: livetest.NewCell(t), label: os.Getenv("LOAD_RUN_LABEL")}
	if r.label == "" {
		r.label = time.Now().UTC().Format("20060102T150405Z")
	}
	r.evidence = os.Getenv("LOAD_EVIDENCE_DIR")
	if r.evidence == "" {
		r.evidence = filepath.Join(t.TempDir(), "evidence")
	}
	if err := os.MkdirAll(r.evidence, 0o755); err != nil {
		t.Fatal(err)
	}
	r.addr, r.metrics = freeAddr(t), freeAddr(t)
	r.base = "http://" + r.addr

	root := livetest.RepoRoot(t)
	r.gwBin = filepath.Join(t.TempDir(), "godspeed-casework")
	build := exec.Command("go", "build", "-o", r.gwBin, "./cmd/godspeed-casework")
	build.Dir = filepath.Join(root, "apps", "godspeed-casework-go")
	if out, err := build.CombinedOutput(); err != nil {
		t.Fatalf("gateway build failed: %v\n%s", err, out)
	}

	type user struct {
		Username    string `json:"username"`
		DisplayName string `json:"display_name"`
		ActorID     string `json:"actor_id"`
		Role        string `json:"role"`
	}
	var users []user
	for i := 0; i < nUsers; i++ {
		name := fmt.Sprintf("load-u%03d", i)
		r.users = append(r.users, name)
		users = append(users, user{name, "Load user " + name, "operator_local", "operator"})
	}
	cfg := map[string]any{
		"version":       "1",
		"evidence_root": filepath.Join(r.cell.Root(), "evidence"),
		"serve": map[string]any{
			"gateway_actor_id":      "gateway",
			"gateway_role":          "service",
			"policy_ref":            "authority/active-policy.json",
			"perspective_actor_id":  "operator_local",
			"perspective_role":      "operator",
			"production":            false,
			"rate_limit":            map[string]any{"intents_per_minute": 60000, "burst": 1000},
			"execution_timeout_sec": 60,
		},
		"auth": map[string]any{"mode": "dev", "users": users},
		"capabilities": []any{map[string]any{
			"name": "authority", "kind": "authority", "required": false, "adapter": "sfwp",
			"endpoint": "unix://" + r.cell.SocketPath(),
		}},
	}
	raw, _ := json.Marshal(cfg)
	r.cfgFile = filepath.Join(r.cell.Root(), "gateway.json")
	if err := os.WriteFile(r.cfgFile, raw, 0o600); err != nil {
		t.Fatal(err)
	}
	r.startGateway()
	t.Cleanup(r.killGateway)
	return r
}

func (r *rig) startGateway() {
	t := r.t
	t.Helper()
	logPath := filepath.Join(r.cell.Root(), "gateway.log")
	f, err := os.OpenFile(logPath, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o644)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	cmd := exec.Command(r.gwBin, "-serve", "-addr", r.addr, "-metrics-addr", r.metrics, "-config", r.cfgFile)
	cmd.Env = append(os.Environ(), "GODSPEED_CELL_ROOT="+r.cell.Root())
	cmd.Stdout, cmd.Stderr = f, f
	if err := cmd.Start(); err != nil {
		t.Fatalf("start gateway: %v", err)
	}
	r.gwMu.Lock()
	r.gw, r.gwLog = cmd, logPath
	r.gwMu.Unlock()
	deadline := time.Now().Add(30 * time.Second)
	for time.Now().Before(deadline) {
		if resp, err := http.Get(r.base + "/api/healthz"); err == nil {
			resp.Body.Close()
			if resp.StatusCode == 200 {
				return
			}
		}
		time.Sleep(100 * time.Millisecond)
	}
	t.Fatalf("gateway never answered healthz; log %s", logPath)
}

func (r *rig) gwPID() int {
	r.gwMu.Lock()
	defer r.gwMu.Unlock()
	if r.gw == nil || r.gw.Process == nil {
		return 0
	}
	return r.gw.Process.Pid
}

// killGateway is kill -9 (the crash); idempotent.
func (r *rig) killGateway() {
	r.gwMu.Lock()
	cmd := r.gw
	r.gw = nil
	r.gwMu.Unlock()
	if cmd == nil || cmd.Process == nil {
		return
	}
	_ = cmd.Process.Kill()
	_, _ = cmd.Process.Wait()
}

func (r *rig) waitReady(budget time.Duration) bool {
	deadline := time.Now().Add(budget)
	for time.Now().Before(deadline) {
		if resp, err := http.Get(r.base + "/api/readyz"); err == nil {
			ok := resp.StatusCode == 200
			resp.Body.Close()
			if ok {
				return true
			}
		}
		time.Sleep(200 * time.Millisecond)
	}
	return false
}

// ---------------------------------------------------------------------------------------------
// process samples (/proc)

type sample struct {
	AtMs      int64  `json:"t_ms"`
	Phase     string `json:"phase"`
	RSSKB     int64  `json:"gateway_rss_kb"`
	HWMKB     int64  `json:"gateway_hwm_kb"`
	Threads   int64  `json:"gateway_threads"`
	FDs       int    `json:"gateway_fds"`
	KernelRSS int64  `json:"kernel_rss_kb"`
}

func procStatus(pid int) (rss, hwm, threads int64) {
	raw, err := os.ReadFile(fmt.Sprintf("/proc/%d/status", pid))
	if err != nil {
		return
	}
	for _, line := range strings.Split(string(raw), "\n") {
		f := strings.Fields(line)
		if len(f) < 2 {
			continue
		}
		v, _ := strconv.ParseInt(f[1], 10, 64)
		switch f[0] {
		case "VmRSS:":
			rss = v
		case "VmHWM:":
			hwm = v
		case "Threads:":
			threads = v
		}
	}
	return
}

// procCPUms is the process's user+system CPU time in milliseconds (USER_HZ is 100 on Linux).
func procCPUms(pid int) int64 {
	raw, err := os.ReadFile(fmt.Sprintf("/proc/%d/stat", pid))
	if err != nil {
		return 0
	}
	// The command name may contain spaces; fields after the closing paren start at "state".
	i := strings.LastIndex(string(raw), ")")
	if i < 0 {
		return 0
	}
	f := strings.Fields(string(raw)[i+2:])
	if len(f) < 13 {
		return 0
	}
	ut, _ := strconv.ParseInt(f[11], 10, 64)
	st, _ := strconv.ParseInt(f[12], 10, 64)
	return (ut + st) * 10
}

func fdCount(pid int) int {
	ents, err := os.ReadDir(fmt.Sprintf("/proc/%d/fd", pid))
	if err != nil {
		return 0
	}
	return len(ents)
}

type sampler struct {
	r      *rig
	start  time.Time
	phase  atomic.Value
	mu     sync.Mutex
	series []sample
	// Peaks of the gateway's own backlog gauges, scraped on every tick.
	peakQueue, peakCursorLag float64
	stop                     chan struct{}
	done                     chan struct{}
}

func (r *rig) startSampler(kernelPID func() int) *sampler {
	s := &sampler{r: r, start: time.Now(), stop: make(chan struct{}), done: make(chan struct{})}
	s.phase.Store("setup")
	go func() {
		defer close(s.done)
		tick := time.NewTicker(250 * time.Millisecond)
		defer tick.Stop()
		for {
			select {
			case <-s.stop:
				return
			case <-tick.C:
				s.take(kernelPID)
			}
		}
	}()
	return s
}

func (s *sampler) take(kernelPID func() int) {
	pid := s.r.gwPID()
	if pid == 0 {
		return
	}
	rss, hwm, th := procStatus(pid)
	if rss == 0 {
		return // process gone (between kill and restart)
	}
	var krss int64
	if kernelPID != nil {
		if kp := kernelPID(); kp > 0 {
			krss, _, _ = procStatus(kp)
		}
	}
	m := s.r.scrapeMetrics()
	s.mu.Lock()
	if v := m["casework_sse_queue_depth_max"]; v > s.peakQueue {
		s.peakQueue = v
	}
	if v := m["casework_sse_cursor_lag_max"]; v > s.peakCursorLag {
		s.peakCursorLag = v
	}
	s.series = append(s.series, sample{
		AtMs: time.Since(s.start).Milliseconds(), Phase: s.phase.Load().(string),
		RSSKB: rss, HWMKB: hwm, Threads: th, FDs: fdCount(pid), KernelRSS: krss,
	})
	s.mu.Unlock()
}

func (s *sampler) setPhase(p string) { s.phase.Store(p) }

func (s *sampler) finish() []sample {
	close(s.stop)
	<-s.done
	s.mu.Lock()
	defer s.mu.Unlock()
	return append([]sample(nil), s.series...)
}

// latest returns the most recent sample (a fresh read), for point-in-time bounds.
func (s *sampler) latest() sample {
	pid := s.r.gwPID()
	rss, hwm, th := procStatus(pid)
	return sample{AtMs: time.Since(s.start).Milliseconds(), RSSKB: rss, HWMKB: hwm, Threads: th, FDs: fdCount(pid)}
}

func maxRSS(series []sample, phase string) int64 {
	var m int64
	for _, s := range series {
		if (phase == "" || s.Phase == phase) && s.RSSKB > m {
			m = s.RSSKB
		}
	}
	return m
}

// ---------------------------------------------------------------------------------------------
// metrics scrape

type scrape map[string]float64

func (r *rig) scrapeMetrics() scrape {
	out := scrape{}
	resp, err := http.Get("http://" + r.metrics + "/metrics")
	if err != nil {
		return out
	}
	defer resp.Body.Close()
	sc := bufio.NewScanner(resp.Body)
	for sc.Scan() {
		line := sc.Text()
		if strings.HasPrefix(line, "#") || strings.Contains(line, "{") {
			continue
		}
		f := strings.Fields(line)
		if len(f) == 2 {
			v, _ := strconv.ParseFloat(f[1], 64)
			out[f[0]] = v
		}
	}
	return out
}

// ---------------------------------------------------------------------------------------------
// browser-like session + SSE client

type session struct {
	user string
	base string
	hc   *http.Client
	mu   sync.Mutex
	csrf string
}

func newTransport() *http.Transport {
	return &http.Transport{MaxIdleConns: 512, MaxIdleConnsPerHost: 512, IdleConnTimeout: 30 * time.Second}
}

func login(base, user string, tr http.RoundTripper) (*session, error) {
	jar, _ := cookiejar.New(nil)
	s := &session{user: user, base: base, hc: &http.Client{Jar: jar, Transport: tr, Timeout: 60 * time.Second}}
	resp, err := s.hc.Get(base + "/api/session")
	if err != nil {
		return nil, err
	}
	resp.Body.Close()
	var csrfCookie string
	for _, c := range jar.Cookies(mustURL(base)) {
		if c.Name == "casework_csrf" {
			csrfCookie = c.Value
		}
	}
	req, _ := http.NewRequest(http.MethodPost, base+"/api/auth/login",
		strings.NewReader(fmt.Sprintf(`{"username":%q,"password":"load"}`, user)))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-CSRF-Token", csrfCookie)
	resp, err = s.hc.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != 200 {
		raw, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("login %s: %d %s", user, resp.StatusCode, raw)
	}
	var wire struct {
		CSRFToken string `json:"csrf_token"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&wire); err != nil || wire.CSRFToken == "" {
		return nil, fmt.Errorf("login %s: no csrf token (%v)", user, err)
	}
	s.csrf = wire.CSRFToken
	return s, nil
}

func (s *session) setCSRF(v string) { s.mu.Lock(); s.csrf = v; s.mu.Unlock() }
func (s *session) token() string    { s.mu.Lock(); defer s.mu.Unlock(); return s.csrf }

func (s *session) postJSON(path string, body any) (*http.Response, error) {
	raw, _ := json.Marshal(body)
	req, _ := http.NewRequest(http.MethodPost, s.base+path, bytes.NewReader(raw))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-CSRF-Token", s.token())
	return s.hc.Do(req)
}

func mustURL(s string) *url.URL { u, _ := url.Parse(s); return u }

type evRec struct {
	ID    string
	Recv  time.Time
	Phase string
	Conn  int
}

// sseClient is an EventSource-equivalent: it reconnects with Last-Event-ID after any break,
// re-authenticates after a 401 (gateway sessions are in memory), and records every snapshot.
type sseClient struct {
	idx    int
	rg     *rig
	tr     http.RoundTripper
	phase  *atomic.Value
	cancel context.CancelFunc
	done   chan struct{}

	mu       sync.Mutex
	sess     *session
	last     string
	events   []evRec
	conns    int
	resync   int // resync_required frames received
	logins   int
	connFail int
	live     bool // a stream is currently open (hello received)
}

func (c *sseClient) snapshotCount() int {
	c.mu.Lock()
	defer c.mu.Unlock()
	return len(c.events)
}

func (c *sseClient) lastID() string { c.mu.Lock(); defer c.mu.Unlock(); return c.last }

func (c *sseClient) isLive() bool { c.mu.Lock(); defer c.mu.Unlock(); return c.live }

func (c *sseClient) run(ctx context.Context) {
	defer close(c.done)
	for ctx.Err() == nil {
		c.mu.Lock()
		sess := c.sess
		last := c.last
		c.mu.Unlock()
		if sess == nil {
			s, err := login(c.rg.base, c.rg.users[c.idx], c.tr)
			if err != nil {
				c.mu.Lock()
				c.connFail++
				c.mu.Unlock()
				sleepCtx(ctx, 200*time.Millisecond)
				continue
			}
			c.mu.Lock()
			c.sess = s
			c.logins++
			c.mu.Unlock()
			continue
		}
		req, _ := http.NewRequestWithContext(ctx, http.MethodGet, c.rg.base+"/api/events", nil)
		if last != "" {
			req.Header.Set("Last-Event-ID", last)
		}
		resp, err := sess.hc.Do(req)
		if err != nil {
			c.mu.Lock()
			c.connFail++
			c.mu.Unlock()
			sleepCtx(ctx, 100*time.Millisecond)
			continue
		}
		if resp.StatusCode == http.StatusUnauthorized || resp.StatusCode == http.StatusForbidden {
			resp.Body.Close()
			c.mu.Lock()
			c.sess = nil
			c.mu.Unlock()
			continue
		}
		if resp.StatusCode != 200 {
			resp.Body.Close()
			c.mu.Lock()
			c.connFail++
			c.mu.Unlock()
			sleepCtx(ctx, 200*time.Millisecond)
			continue
		}
		c.mu.Lock()
		c.conns++
		conn := c.conns
		c.mu.Unlock()
		c.consume(resp.Body, conn)
		resp.Body.Close()
		c.mu.Lock()
		c.live = false
		c.mu.Unlock()
	}
}

func (c *sseClient) consume(body io.Reader, conn int) {
	rd := bufio.NewReaderSize(body, 1<<16)
	var id, event string
	for {
		line, err := rd.ReadString('\n')
		now := time.Now()
		switch {
		case strings.HasPrefix(line, "id: "):
			id = strings.TrimSpace(line[4:])
		case strings.HasPrefix(line, "event: "):
			event = strings.TrimSpace(line[7:])
		case line == "\n" || line == "\r\n":
			c.mu.Lock()
			switch event {
			case "hello":
				c.live = true
			case "resync_required":
				c.resync++
			case "snapshot":
				if id != "" {
					c.events = append(c.events, evRec{ID: id, Recv: now, Phase: c.phase.Load().(string), Conn: conn})
					c.last = id
				}
			}
			c.mu.Unlock()
			id, event = "", ""
		}
		if err != nil {
			return
		}
	}
}

func sleepCtx(ctx context.Context, d time.Duration) {
	select {
	case <-ctx.Done():
	case <-time.After(d):
	}
}

type fleet struct {
	clients []*sseClient
	phase   *atomic.Value
	tr      *http.Transport
	cancel  context.CancelFunc
}

func (r *rig) openFleet(n int) *fleet {
	f := &fleet{phase: &atomic.Value{}, tr: newTransport()}
	f.phase.Store("setup")
	ctx, cancel := context.WithCancel(context.Background())
	f.cancel = cancel
	for i := 0; i < n; i++ {
		c := &sseClient{idx: i, rg: r, tr: f.tr, phase: f.phase, done: make(chan struct{})}
		f.clients = append(f.clients, c)
		go c.run(ctx)
	}
	r.t.Cleanup(f.close)
	return f
}

func (f *fleet) close() {
	f.cancel()
	f.tr.CloseIdleConnections()
	for _, c := range f.clients {
		select {
		case <-c.done:
		case <-time.After(5 * time.Second):
		}
	}
}

func (f *fleet) setPhase(p string) { f.phase.Store(p) }

func (f *fleet) waitAllLive(budget time.Duration) bool {
	deadline := time.Now().Add(budget)
	for time.Now().Before(deadline) {
		all := true
		for _, c := range f.clients {
			if !c.isLive() {
				all = false
				break
			}
		}
		if all {
			return true
		}
		time.Sleep(50 * time.Millisecond)
	}
	return false
}

// ---------------------------------------------------------------------------------------------
// intents

type intentResult struct {
	Stale   int // STALE_PROJECTION refusals absorbed by the client's refetch-and-retry
	CaseID  string
	Latency time.Duration
	OK      bool
	Detail  string
}

type caseRef struct {
	id     string
	cursor string
}

func (r *rig) proposeCase(sess *session, i int) (caseRef, error) {
	params := map[string]string{"dataset_name": fmt.Sprintf("load-%d", i), "dataset_label": "load", "max_rows": "25", "out_dir": "work"}
	pre, err := sess.postJSON("/api/templates/preflight", map[string]any{"template_ref": "e2e-sentry-chain@0.1.0", "params": params})
	if err != nil {
		return caseRef{}, err
	}
	var pf struct {
		Passed bool    `json:"passed"`
		Digest *string `json:"digest"`
	}
	raw, _ := io.ReadAll(pre.Body)
	pre.Body.Close()
	if err := json.Unmarshal(raw, &pf); err != nil || !pf.Passed || pf.Digest == nil {
		return caseRef{}, fmt.Errorf("preflight: %s", raw)
	}
	in := contract.InteractionIntent{
		IntentID: fmt.Sprintf("load-%s-propose-%d", r.label, i), Kind: "CONSEQUENTIAL_CASE", ActionName: "PROPOSE_CASE",
		CaseID: "case-new", Actor: contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters: map[string]any{"template_ref": "e2e-sentry-chain@0.1.0", "preflight_digest": *pf.Digest, "params": params},
	}
	resp, err := sess.postJSON("/api/intents", in)
	if err != nil {
		return caseRef{}, err
	}
	raw, _ = io.ReadAll(resp.Body)
	resp.Body.Close()
	var out contract.IntentResponse
	if err := json.Unmarshal(raw, &out); err != nil || !out.Success || out.NewCursor == nil {
		return caseRef{}, fmt.Errorf("propose refused: %s", raw)
	}
	caseID := ""
	if out.ResultingObject != nil {
		caseID = out.ResultingObject.ID
	}
	if caseID == "" {
		return caseRef{}, fmt.Errorf("propose returned no case id: %s", raw)
	}
	return caseRef{id: caseID, cursor: *out.NewCursor}, nil
}

// addWork posts one ADD_DISCRETIONARY_WORK intent (a cheap governed plan mutation) and returns
// the new case cursor.
func (r *rig) addWork(sess *session, c caseRef, tag string) (intentResult, string) {
	in := contract.InteractionIntent{
		IntentID: "load-" + r.label + "-" + tag, Kind: "CONSEQUENTIAL_CASE", ActionName: "ADD_DISCRETIONARY_WORK",
		CaseID: c.id, ClientCursor: c.cursor, Actor: contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters: map[string]any{
			"case_id": c.id, "stage_id": "task_prepare", "kind": "sandboxed_task",
			"title": "Load item " + tag, "summary": "Load-test discretionary work item.",
			"justification": "T11 load harness: cheap valid governed plan mutation.",
		},
	}
	t0 := time.Now()
	resp, err := sess.postJSON("/api/intents", in)
	if err != nil {
		return intentResult{CaseID: c.id, Latency: time.Since(t0), Detail: "transport: " + err.Error()}, c.cursor
	}
	raw, _ := io.ReadAll(resp.Body)
	resp.Body.Close()
	lat := time.Since(t0)
	var out contract.IntentResponse
	if resp.StatusCode != 200 || json.Unmarshal(raw, &out) != nil || !out.Success || out.NewCursor == nil {
		d := fmt.Sprintf("status %d: %s", resp.StatusCode, truncate(string(raw), 300))
		// A stale refusal carries the fresh cursor; adopt it so the NEXT intent is honest about
		// freshness, but this one still counts as an error.
		if out.Refusal != nil && out.Refusal.CurrentCursor != nil && *out.Refusal.CurrentCursor != "" {
			return intentResult{CaseID: c.id, Latency: lat, Detail: d}, *out.Refusal.CurrentCursor
		}
		return intentResult{CaseID: c.id, Latency: lat, Detail: d}, c.cursor
	}
	return intentResult{CaseID: c.id, Latency: lat, OK: true}, *out.NewCursor
}

// addWorkRetry follows the documented client protocol: a STALE_PROJECTION refusal carries the
// fresh cursor, so the client adopts it and retries (a stale refusal produces no kernel write).
// The reported latency spans every attempt; refusals are counted, never hidden. Any other failure
// is final.
func (r *rig) addWorkRetry(sess *session, c caseRef, tag string) (intentResult, string) {
	t0 := time.Now()
	stale := 0
	for attempt := 0; ; attempt++ {
		id := tag
		if attempt > 0 {
			id = fmt.Sprintf("%s-r%d", tag, attempt)
		}
		res, cur := r.addWork(sess, c, id)
		if !res.OK && attempt < 3 && cur != c.cursor && strings.Contains(res.Detail, "STALE_PROJECTION") {
			stale++
			c.cursor = cur
			continue
		}
		res.Latency, res.Stale = time.Since(t0), stale
		return res, cur
	}
}

func truncate(s string, n int) string {
	if len(s) > n {
		return s[:n] + "..."
	}
	return s
}

type burstOutcome struct {
	Results  []intentResult
	Wall     time.Duration
	Failures []string
}

// burst posts total intents across the given cases: one worker per case posts that case's share
// sequentially (an intent carries the case cursor the previous one returned, so a case is a
// serial stream); the cases run concurrently, so concurrency == len(cases).
func (r *rig) burst(sessions []*session, cases []caseRef, total int, tagPrefix string) burstOutcome {
	var mu sync.Mutex
	var out burstOutcome
	var wg sync.WaitGroup
	t0 := time.Now()
	for ci := range cases {
		wg.Add(1)
		go func(ci int) {
			defer wg.Done()
			c := cases[ci]
			for j := ci; j < total; j += len(cases) {
				sess := sessions[(ci+j)%len(sessions)]
				res, cur := r.addWorkRetry(sess, c, fmt.Sprintf("%s-%d", tagPrefix, j))
				c.cursor = cur
				mu.Lock()
				out.Results = append(out.Results, res)
				if !res.OK {
					out.Failures = append(out.Failures, res.Detail)
				}
				mu.Unlock()
			}
			cases[ci] = c // carry the newest case cursor into the next burst
		}(ci)
	}
	wg.Wait()
	out.Wall = time.Since(t0)
	return out
}

// ---------------------------------------------------------------------------------------------
// stream invariants

// auditStreams checks, per client: no duplicate revision ids over the whole run, ids strictly
// increasing in receipt order (the ULID cursors are lexicographically ordered), and that every
// client saw the identical revision sequence (same actor, same store). It returns the problems.
func auditStreams(f *fleet) []string {
	var problems []string
	var ref []string
	for _, c := range f.clients {
		c.mu.Lock()
		ids := make([]string, len(c.events))
		for i, e := range c.events {
			ids[i] = e.ID
		}
		c.mu.Unlock()
		seen := map[string]bool{}
		for i, id := range ids {
			if seen[id] {
				problems = append(problems, fmt.Sprintf("client %d: duplicate revision %s", c.idx, id))
			}
			seen[id] = true
			if i > 0 && id <= ids[i-1] {
				problems = append(problems, fmt.Sprintf("client %d: id %s not after %s (not monotonic)", c.idx, id, ids[i-1]))
			}
		}
		if ref == nil {
			ref = ids
			continue
		}
		if !sameStrings(ids, ref) {
			problems = append(problems, fmt.Sprintf("client %d: saw %d revisions, client 0 saw %d (sequences differ: %s)",
				c.idx, len(ids), len(ref), firstDiff(ids, ref)))
		}
	}
	return problems
}

func sameStrings(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

func firstDiff(a, b []string) string {
	for i := 0; i < len(a) && i < len(b); i++ {
		if a[i] != b[i] {
			return fmt.Sprintf("first difference at %d: %s vs %s", i, a[i], b[i])
		}
	}
	return "length differs"
}

func (r *rig) writeEvidence(name string, v any) string {
	raw, _ := json.MarshalIndent(v, "", "  ")
	p := filepath.Join(r.evidence, fmt.Sprintf("%s-%s.json", name, r.label))
	if err := os.WriteFile(p, raw, 0o644); err != nil {
		r.t.Fatal(err)
	}
	return p
}

// snapshotLogs copies the gateway and kernel logs next to the evidence (the temp cell is removed).
func (r *rig) preserveLogs(name string) {
	for src, dst := range map[string]string{
		filepath.Join(r.cell.Root(), "gateway.log"): fmt.Sprintf("%s-%s-gateway.log", name, r.label),
		filepath.Join(r.cell.Root(), "server.log"):  fmt.Sprintf("%s-%s-kernel.log", name, r.label),
	} {
		if raw, err := os.ReadFile(src); err == nil {
			if len(raw) > 200_000 {
				raw = raw[len(raw)-200_000:]
			}
			_ = os.WriteFile(filepath.Join(r.evidence, dst), raw, 0o644)
		}
	}
}

// refreshCursor is the client-side resync: refetch the case's world and adopt its cursor.
func (r *rig) refreshCursor(sess *session, c caseRef) (caseRef, error) {
	resp, err := sess.hc.Get(r.base + "/api/world?case_id=" + c.id)
	if err != nil {
		return c, err
	}
	defer resp.Body.Close()
	var wire struct {
		Snapshot contract.CognitiveWorldSnapshot `json:"snapshot"`
	}
	raw, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != 200 || json.Unmarshal(raw, &wire) != nil || wire.Snapshot.Cursor == "" {
		return c, fmt.Errorf("world refetch for %s: %d %s", c.id, resp.StatusCode, truncate(string(raw), 200))
	}
	c.cursor = wire.Snapshot.Cursor
	return c, nil
}
