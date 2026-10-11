//go:build load

package loadtest

import (
	"fmt"
	"sync"
	"testing"
	"time"
)

// settle waits until every client has stopped receiving (no new snapshot for quiet) or budget ends.
func (f *fleet) settle(quiet, budget time.Duration) {
	deadline := time.Now().Add(budget)
	lastTotal, lastChange := -1, time.Now()
	for time.Now().Before(deadline) {
		total := 0
		for _, c := range f.clients {
			total += c.snapshotCount()
		}
		if total != lastTotal {
			lastTotal, lastChange = total, time.Now()
		} else if time.Since(lastChange) >= quiet {
			return
		}
		time.Sleep(50 * time.Millisecond)
	}
}

func (f *fleet) sessions() []*session {
	out := make([]*session, 0, len(f.clients))
	for _, c := range f.clients {
		c.mu.Lock()
		if c.sess != nil {
			out = append(out, c.sess)
		}
		c.mu.Unlock()
	}
	return out
}

func (r *rig) makeCases(t *testing.T, sess *session, n int) []caseRef {
	t.Helper()
	cases := make([]caseRef, 0, n)
	for i := 0; i < n; i++ {
		c, err := r.proposeCase(sess, i)
		if err != nil {
			t.Fatalf("case %d: %v", i, err)
		}
		cases = append(cases, c)
	}
	return cases
}

// lagsMs returns kernel-append -> receipt lag for every snapshot received in the phase.
func (f *fleet) lagsMs(t *testing.T, phase string) []float64 {
	var out []float64
	for _, c := range f.clients {
		c.mu.Lock()
		for _, e := range c.events {
			if e.Phase != phase {
				continue
			}
			ts, ok := ulidTime(e.ID)
			if !ok {
				c.mu.Unlock()
				t.Fatalf("event id %q is not a ULID; the kernel-append timestamp cannot be derived", e.ID)
			}
			out = append(out, float64(e.Recv.Sub(ts).Microseconds())/1000)
		}
		c.mu.Unlock()
	}
	return out
}

func countPhase(f *fleet, phase string) int {
	n := 0
	for _, c := range f.clients {
		c.mu.Lock()
		for _, e := range c.events {
			if e.Phase == phase {
				n++
			}
		}
		c.mu.Unlock()
	}
	return n
}

type loadReport struct {
	Label    string `json:"label"`
	Clients  int    `json:"sse_clients"`
	Intents  int    `json:"intents"`
	Cases    int    `json:"cases"`
	HostCPUs int    `json:"host_cpus,omitempty"`

	IntentLatency dist    `json:"intent_latency"`
	IntentErrors  int     `json:"intent_errors"`
	StaleRefusals int     `json:"stale_projection_refusals_retried"`
	StaleRate     float64 `json:"stale_refusal_rate"`
	ErrorRate     float64 `json:"error_rate"`
	BurstWallMs   int64   `json:"burst_wall_ms"`
	IntentsPerSec float64 `json:"intents_per_sec"`

	EventLag           dist `json:"event_lag"`
	RevisionsPerClient int  `json:"burst_revisions_per_client"`
	MissingDeliveries  int  `json:"missing_deliveries"`
	Reconnects         int  `json:"client_reconnects_in_run"`
	ResyncFrames       int  `json:"resync_required_frames"`

	IdleRSSKB      int64   `json:"gateway_rss_idle_with_clients_kb"`
	PeakRSSKB      int64   `json:"gateway_rss_peak_kb"`
	AfterRSSKB     int64   `json:"gateway_rss_after_kb"`
	PeakGoroutines float64 `json:"gateway_goroutines_peak"`
	IdleGoroutines float64 `json:"gateway_goroutines_idle"`
	PeakFDs        int     `json:"gateway_fds_peak"`
	IdleFDs        int     `json:"gateway_fds_idle"`
	PeakThreads    int64   `json:"gateway_threads_peak"`
	KernelPeakKB   int64   `json:"kernel_rss_peak_kb"`
	QueueDepthPeak float64 `json:"sse_queue_depth_peak"`
	CursorLagPeak  float64 `json:"sse_cursor_lag_peak"`
	GatewayCPUMs   int64   `json:"gateway_cpu_ms_during_burst"`
	KernelCPUMs    int64   `json:"kernel_cpu_ms_during_burst"`

	Metrics  map[string]float64 `json:"metrics_after"`
	Samples  []sample           `json:"samples"`
	Problems []string           `json:"problems"`
}

func TestLoadBudgets(t *testing.T) {
	nClients := envInt("LOAD_CLIENTS", 50)
	nIntents := envInt("LOAD_INTENTS", 100)
	nCases := envInt("LOAD_CASES", 20)

	r := newRig(t, nClients)
	smp := r.startSampler(r.cell.PID)
	fl := r.openFleet(nClients)
	if !fl.waitAllLive(30 * time.Second) {
		t.Fatalf("not all %d SSE clients came up", nClients)
	}
	sessions := fl.sessions()
	if len(sessions) != nClients {
		t.Fatalf("expected %d logged-in sessions, have %d", nClients, len(sessions))
	}

	cases := r.makeCases(t, sessions[0], nCases)
	fl.settle(500*time.Millisecond, 20*time.Second)
	time.Sleep(1 * time.Second)
	idle := smp.latest()
	idleMetrics := r.scrapeMetrics()
	if idleMetrics["casework_sse_clients"] != float64(nClients) {
		t.Fatalf("gateway reports %v open SSE clients, want %d", idleMetrics["casework_sse_clients"], nClients)
	}

	// --- the burst
	smp.setPhase("burst")
	fl.setPhase("burst")
	cpu0, kcpu0 := procCPUms(r.gwPID()), procCPUms(r.cell.PID())
	out := r.burst(sessions, cases, nIntents, "b")
	gwCPU, kernelCPU := procCPUms(r.gwPID())-cpu0, procCPUms(r.cell.PID())-kcpu0
	fl.settle(750*time.Millisecond, 30*time.Second)
	smp.setPhase("after")
	fl.setPhase("after")
	time.Sleep(2 * time.Second)
	afterMetrics := r.scrapeMetrics()
	after := smp.latest()
	series := smp.finish()

	rep := loadReport{Label: r.label, Clients: nClients, Intents: nIntents, Cases: nCases, Metrics: afterMetrics, Samples: series,
		QueueDepthPeak: smp.peakQueue, CursorLagPeak: smp.peakCursorLag}
	var lat []float64
	for _, res := range out.Results {
		lat = append(lat, float64(res.Latency.Microseconds())/1000)
		if !res.OK {
			rep.IntentErrors++
		}
		rep.StaleRefusals += res.Stale
	}
	rep.StaleRate = float64(rep.StaleRefusals) / float64(len(out.Results))
	rep.IntentLatency = summarize(lat)
	rep.ErrorRate = float64(rep.IntentErrors) / float64(len(out.Results))
	rep.BurstWallMs = out.Wall.Milliseconds()
	rep.IntentsPerSec = float64(len(out.Results)) / out.Wall.Seconds()
	rep.EventLag = summarize(fl.lagsMs(t, "burst"))
	rep.RevisionsPerClient = countPhase(fl, "burst") / nClients
	for _, c := range fl.clients {
		c.mu.Lock()
		if c.conns > 1 {
			rep.Reconnects += c.conns - 1
		}
		rep.ResyncFrames += c.resync
		c.mu.Unlock()
	}
	rep.IdleRSSKB, rep.AfterRSSKB, rep.PeakRSSKB = idle.RSSKB, after.RSSKB, maxRSS(series, "")
	rep.IdleFDs = idle.FDs
	rep.GatewayCPUMs, rep.KernelCPUMs = gwCPU, kernelCPU
	rep.IdleGoroutines = idleMetrics["casework_go_goroutines"]
	rep.PeakGoroutines = afterMetrics["casework_go_goroutines"]
	for _, s := range series {
		if s.FDs > rep.PeakFDs {
			rep.PeakFDs = s.FDs
		}
		if s.Threads > rep.PeakThreads {
			rep.PeakThreads = s.Threads
		}
		if s.KernelRSS > rep.KernelPeakKB {
			rep.KernelPeakKB = s.KernelRSS
		}
	}
	// Missing deliveries: every client must have the revision count of the most complete client.
	most := 0
	for _, c := range fl.clients {
		if n := c.snapshotCount(); n > most {
			most = n
		}
	}
	for _, c := range fl.clients {
		rep.MissingDeliveries += most - c.snapshotCount()
	}

	rep.Problems = append(rep.Problems, auditStreams(fl)...)
	if len(out.Failures) > 0 {
		rep.Problems = append(rep.Problems, fmt.Sprintf("%d intent failures, first: %s", len(out.Failures), out.Failures[0]))
	}
	rep.Problems = append(rep.Problems, checkBudgets(rep)...)
	path := r.writeEvidence("load", rep)
	r.preserveLogs("load")

	t.Logf("evidence: %s", path)
	t.Logf("intent latency ms p50/p95/p99/max = %.1f/%.1f/%.1f/%.1f over %d intents; errors %d (%.2f%%); stale refusals retried %d; %.1f intents/s",
		rep.IntentLatency.P50, rep.IntentLatency.P95, rep.IntentLatency.P99, rep.IntentLatency.Max, len(out.Results),
		rep.IntentErrors, rep.ErrorRate*100, rep.StaleRefusals, rep.IntentsPerSec)
	t.Logf("event lag ms p50/p95/p99/max = %.1f/%.1f/%.1f/%.1f over %d deliveries (%d revisions per client)",
		rep.EventLag.P50, rep.EventLag.P95, rep.EventLag.P99, rep.EventLag.Max, rep.EventLag.N, rep.RevisionsPerClient)
	t.Logf("gateway RSS MB idle/peak/after = %.1f/%.1f/%.1f; goroutines idle/after = %.0f/%.0f; fds idle/peak = %d/%d; threads peak %d; kernel RSS peak %.1f MB",
		float64(rep.IdleRSSKB)/1024, float64(rep.PeakRSSKB)/1024, float64(rep.AfterRSSKB)/1024,
		rep.IdleGoroutines, rep.PeakGoroutines, rep.IdleFDs, rep.PeakFDs, rep.PeakThreads, float64(rep.KernelPeakKB)/1024)
	t.Logf("gateway backlog peaks: queue depth %v, cursor lag %v", rep.QueueDepthPeak, rep.CursorLagPeak)
	t.Logf("CPU during burst: gateway %d ms, kernel %d ms (burst wall %d ms)", rep.GatewayCPUMs, rep.KernelCPUMs, rep.BurstWallMs)
	t.Logf("metrics: sse_clients=%v queue_depth_max=%v cursor_lag_max=%v delivery_lag_count=%v delivery_lag_sum_s=%v",
		afterMetrics["casework_sse_clients"], afterMetrics["casework_sse_queue_depth_max"], afterMetrics["casework_sse_cursor_lag_max"],
		afterMetrics["casework_sse_delivery_lag_seconds_count"], afterMetrics["casework_sse_delivery_lag_seconds_sum"])
	for _, p := range rep.Problems {
		t.Errorf("load budget/invariant violation: %s", p)
	}
}

// ---------------------------------------------------------------------------------------------
// the T11 tooth: restart under load

type restartKind string

const (
	restartKernel  restartKind = "kernel"
	restartGateway restartKind = "gateway"
)

type toothReport struct {
	Label               string   `json:"label"`
	Kind                string   `json:"restart"`
	Clients             int      `json:"sse_clients"`
	PreRevisions        int      `json:"revisions_per_client_before"`
	PostRevisions       int      `json:"revisions_per_client_after"`
	ClientsResynced     int      `json:"clients_resynced"`
	ClientsReconn       int      `json:"clients_that_reconnected"`
	ClientsRelogin      int      `json:"clients_that_relogged_in"`
	ResyncFrames        int      `json:"resync_required_frames"`
	OutageProbeMs       int64    `json:"outage_probe_ms"`
	OutageProbe         string   `json:"outage_probe_result"`
	RecoveryMs          int64    `json:"recovery_to_first_accepted_intent_ms"`
	PostBurstErrors     int      `json:"post_recovery_burst_errors"`
	InFlightIntents     int      `json:"in_flight_intents_at_crash"`
	InFlightFailures    int      `json:"in_flight_intents_failed"`
	AcknowledgedIntents int      `json:"acknowledged_intents"`
	DurableMutations    int      `json:"durable_plan_mutations"`
	PostBurstIntents    int      `json:"post_recovery_burst_intents"`
	RSSBeforeKB         int64    `json:"gateway_rss_before_kb"`
	RSSAfterKB          int64    `json:"gateway_rss_after_kb"`
	RSSPeakKB           int64    `json:"gateway_rss_peak_kb"`
	GoroutinesBefore    float64  `json:"gateway_goroutines_before"`
	GoroutinesAfter     float64  `json:"gateway_goroutines_after"`
	FDsBefore           int      `json:"gateway_fds_before"`
	FDsAfter            int      `json:"gateway_fds_after"`
	Problems            []string `json:"problems"`
	Samples             []sample `json:"samples"`
}

func TestToothKernelRestartUnder50SSEClients(t *testing.T)  { runRestartTooth(t, restartKernel) }
func TestToothGatewayRestartUnder50SSEClients(t *testing.T) { runRestartTooth(t, restartGateway) }

func runRestartTooth(t *testing.T, kind restartKind) {
	nClients := envInt("LOAD_TOOTH_CLIENTS", 50)
	r := newRig(t, nClients)
	smp := r.startSampler(r.cell.PID)
	fl := r.openFleet(nClients)
	if !fl.waitAllLive(30 * time.Second) {
		t.Fatalf("not all %d SSE clients came up", nClients)
	}
	rep := toothReport{Label: r.label, Kind: string(kind), Clients: nClients}
	sessions := fl.sessions()
	cases := r.makeCases(t, sessions[0], 8)

	// Warm load before the crash so the stores, queues and sessions are populated.
	pre := r.burst(sessions, cases, 24, "pre")
	if len(pre.Failures) > 0 {
		t.Fatalf("pre-crash burst failed: %s", pre.Failures[0])
	}
	fl.settle(750*time.Millisecond, 20*time.Second)
	time.Sleep(time.Second)
	before := smp.latest()
	beforeM := r.scrapeMetrics()
	rep.RSSBeforeKB, rep.FDsBefore, rep.GoroutinesBefore = before.RSSKB, before.FDs, beforeM["casework_go_goroutines"]
	rep.PreRevisions = fl.clients[0].snapshotCount()
	preLast := make([]string, nClients)
	preConns := make([]int, nClients)
	preLogins := make([]int, nClients)
	for i, c := range fl.clients {
		preLast[i] = c.lastID()
		c.mu.Lock()
		preConns[i], preLogins[i] = c.conns, c.logins
		c.mu.Unlock()
	}

	// --- the crash, mid-burst: intents are in flight when the process dies
	smp.setPhase("outage")
	fl.setPhase("outage")
	var flight burstOutcome
	flightDone := make(chan struct{})
	go func() {
		defer close(flightDone)
		flight = r.burst(sessions, cases, 80, "flight")
	}()
	time.Sleep(2 * time.Second)
	switch kind {
	case restartKernel:
		r.cell.Kill9()
	case restartGateway:
		r.killGateway()
	}
	select {
	case <-flightDone:
	case <-time.After(90 * time.Second):
		rep.Problems = append(rep.Problems, "in-flight intents did not finish within 90s of the crash")
		<-flightDone
	}
	rep.InFlightIntents, rep.InFlightFailures = len(flight.Results), len(flight.Failures)
	time.Sleep(time.Second)
	if kind == restartKernel {
		// The gateway must stay up and keep every stream open while the kernel is dead.
		live := 0
		for _, c := range fl.clients {
			if c.isLive() {
				live++
			}
		}
		if live != nClients {
			rep.Problems = append(rep.Problems, fmt.Sprintf("only %d/%d SSE streams stayed open across a kernel crash", live, nClients))
		}
		// An intent during the outage must fail fast and honestly, not hang.
		t0 := time.Now()
		res, _ := r.addWork(sessions[0], cases[0], "outage")
		rep.OutageProbeMs = time.Since(t0).Milliseconds()
		rep.OutageProbe = fmt.Sprintf("ok=%v %s", res.OK, truncate(res.Detail, 160))
		if res.OK {
			rep.Problems = append(rep.Problems, "an intent was ACCEPTED while the kernel was dead")
		}
	}
	// The position each client holds when the stack comes back: every client must move past it.
	for i, c := range fl.clients {
		preLast[i] = c.lastID()
	}

	// --- the restart
	restarted := time.Now()
	switch kind {
	case restartKernel:
		r.cell.Restart()
	case restartGateway:
		r.startGateway()
	}
	smp.setPhase("recovery")
	fl.setPhase("recovery")
	if !r.waitReady(60 * time.Second) {
		rep.Problems = append(rep.Problems, "gateway never reported ready after the restart")
	}
	if kind == restartGateway && !fl.waitAllLive(60*time.Second) {
		rep.Problems = append(rep.Problems, "not every client re-established its stream after the gateway restart")
	}

	// First accepted intent after the restart (retrying: the relay/subscription may still be
	// recovering). Sessions: after a gateway restart the clients logged in again.
	sessions = fl.sessions()
	if len(sessions) != nClients {
		rep.Problems = append(rep.Problems, fmt.Sprintf("only %d/%d clients hold a session after the restart", len(sessions), nClients))
		if len(sessions) == 0 {
			t.Fatalf("no sessions after restart: %v", rep.Problems)
		}
	}
	// Client-side resync: refetch each case's world to adopt the fresh cursor (an in-flight
	// intent may or may not have been committed before the crash).
	for i := range cases {
		var err error
		for attempt := 0; attempt < 120; attempt++ {
			if cases[i], err = r.refreshCursor(sessions[0], cases[i]); err == nil {
				break
			}
			time.Sleep(500 * time.Millisecond)
		}
		if err != nil {
			rep.Problems = append(rep.Problems, "world refetch after the restart: "+err.Error())
		}
	}
	var recovered bool
	deadline := time.Now().Add(90 * time.Second)
	for attempt := 0; time.Now().Before(deadline); attempt++ {
		res, next := r.addWork(sessions[0], cases[0], fmt.Sprintf("probe-%d", attempt))
		cases[0].cursor = next
		if res.OK {
			recovered = true
			rep.RecoveryMs = time.Since(restarted).Milliseconds()
			break
		}
		time.Sleep(500 * time.Millisecond)
	}
	if !recovered {
		rep.Problems = append(rep.Problems, "no intent was accepted within 90s of the restart")
	}

	// A burst after recovery: the stack must be fully usable, not merely alive.
	smp.setPhase("post")
	fl.setPhase("post")
	post := r.burst(sessions, cases, 40, "post")
	rep.PostBurstIntents, rep.PostBurstErrors = len(post.Results), len(post.Failures)
	if len(post.Failures) > 0 {
		rep.Problems = append(rep.Problems, fmt.Sprintf("%d/%d intents failed after recovery, first: %s", len(post.Failures), len(post.Results), post.Failures[0]))
	}
	fl.settle(1*time.Second, 30*time.Second)
	time.Sleep(2 * time.Second)

	// --- the assertions
	for i, c := range fl.clients {
		if last := c.lastID(); last > preLast[i] {
			rep.ClientsResynced++
		} else {
			rep.Problems = append(rep.Problems, fmt.Sprintf("client %d never received a revision newer than its pre-crash cursor %s", i, preLast[i]))
		}
		c.mu.Lock()
		if c.conns > preConns[i] {
			rep.ClientsReconn++
		}
		if c.logins > preLogins[i] {
			rep.ClientsRelogin++
		}
		rep.ResyncFrames += c.resync
		c.mu.Unlock()
	}
	if kind == restartGateway {
		if rep.ClientsReconn != nClients {
			rep.Problems = append(rep.Problems, fmt.Sprintf("%d/%d clients reconnected after a gateway restart", rep.ClientsReconn, nClients))
		}
	}
	rep.PostRevisions = fl.clients[0].snapshotCount()
	// Durability: every intent the gateway ACCEPTED (before the crash, after it, in flight or
	// not) must be in the kernel's durable ledger. kill -9 must not lose an acknowledged write.
	accepted := map[string]int{}
	for _, b := range []burstOutcome{pre, flight, post} {
		for _, res := range b.Results {
			if res.OK {
				accepted[res.CaseID]++
			}
		}
	}
	for _, c := range cases {
		durable := r.cell.CountTraceKinds(c.id, "plan_mutated")
		if durable < accepted[c.id] {
			rep.Problems = append(rep.Problems, fmt.Sprintf("case %s: %d intents were acknowledged but only %d plan_mutated events are durable", c.id, accepted[c.id], durable))
		}
		rep.AcknowledgedIntents += accepted[c.id]
		rep.DurableMutations += durable
	}
	rep.Problems = append(rep.Problems, auditStreams(fl)...)

	afterM := r.scrapeMetrics()
	after := smp.latest()
	series := smp.finish()
	rep.Samples = series
	rep.RSSAfterKB, rep.FDsAfter, rep.GoroutinesAfter = after.RSSKB, after.FDs, afterM["casework_go_goroutines"]
	rep.RSSPeakKB = maxRSS(series, "outage")
	for _, ph := range []string{"recovery", "post"} {
		if m := maxRSS(series, ph); m > rep.RSSPeakKB {
			rep.RSSPeakKB = m
		}
	}
	rep.Problems = append(rep.Problems, checkToothBounds(kind, rep, afterM)...)
	if afterM["casework_sse_clients"] != float64(nClients) {
		rep.Problems = append(rep.Problems, fmt.Sprintf("gateway reports %v open streams after recovery, want %d (leaked or lost streams)", afterM["casework_sse_clients"], nClients))
	}

	path := r.writeEvidence("tooth-"+string(kind), rep)
	r.preserveLogs("tooth-" + string(kind))
	t.Logf("evidence: %s", path)
	t.Logf("%s restart: resynced %d/%d clients (reconnected %d, re-logged-in %d, resync_required frames %d); first accepted intent %d ms after the restart began; post burst %d intents, %d errors",
		kind, rep.ClientsResynced, nClients, rep.ClientsReconn, rep.ClientsRelogin, rep.ResyncFrames, rep.RecoveryMs, rep.PostBurstIntents, rep.PostBurstErrors)
	t.Logf("in flight at the crash: %d intents, %d failed; acknowledged %d, durable plan_mutated events %d",
		rep.InFlightIntents, rep.InFlightFailures, rep.AcknowledgedIntents, rep.DurableMutations)
	t.Logf("gateway RSS MB before/peak/after = %.1f/%.1f/%.1f; goroutines %.0f -> %.0f; fds %d -> %d; revisions per client %d -> %d; outage probe (%d ms): %s",
		float64(rep.RSSBeforeKB)/1024, float64(rep.RSSPeakKB)/1024, float64(rep.RSSAfterKB)/1024,
		rep.GoroutinesBefore, rep.GoroutinesAfter, rep.FDsBefore, rep.FDsAfter, rep.PreRevisions, rep.PostRevisions, rep.OutageProbeMs, rep.OutageProbe)
	for _, p := range rep.Problems {
		t.Errorf("restart tooth violation: %s", p)
	}
}

var _ sync.Mutex
