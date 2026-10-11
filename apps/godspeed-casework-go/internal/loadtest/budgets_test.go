//go:build load

package loadtest

import "fmt"

// Budgets are enforced at any scale (a larger scale is a harsher test, never a looser one) and are
// calibrated at the default scale (50 SSE clients, 100 intents over 20 cases). The measured
// baselines, host, run count, variance and the headroom rationale for every number live in
// .agents/reports/casework-live-wiring/load-budgets.md; change both together.
//
// Two kinds of budget:
//   - structural budgets (errors, missing deliveries, reconnects, queue depth, memory,
//     goroutines, fds, gateway CPU): tight, because these are properties of the gateway;
//   - latency budgets (intent latency, kernel-append -> receipt lag): loose tripwires. At 20
//     concurrent intents the kernel is the bottleneck (it burns ~3 cores for the whole burst), so
//     these numbers move with host contention far more than with gateway changes.
const (
	budgetIntentP95Ms = 20_000.0
	budgetIntentP99Ms = 25_000.0
	budgetLagP95Ms    = 40_000.0
	budgetLagP99Ms    = 45_000.0

	budgetErrorRate = 0.0  // any failed intent that is not a retried stale refusal
	budgetStaleRate = 0.20 // stale refusals retried per intent (the client protocol absorbs them)

	// Gateway RSS peak with 50 streams and a 100-intent burst measured 47.0-47.4 MB (idle with the
	// streams open is the same: the Go runtime holds its heap); the budget is 2x.
	budgetPeakRSSKB = 96 * 1024
	// Per-client allowances: goroutines (measured ~2.3 per client + ~15 base) and descriptors
	// (measured ~1.4 per client + ~15 base).
	budgetGoroutinesPerClient = 4
	budgetGoroutinesBase      = 60
	budgetFDsPerClient        = 3
	budgetFDsBase             = 60
	// Gateway CPU spent serving the burst (rendering 50 streams x ~100 revisions): measured
	// 0.70-0.74 s, budget 5 s. This is the guard for the "per-client projection rebuild" redesign
	// trigger: if rendering cost per delivery regresses by a large factor it trips here first.
	budgetGatewayCPUMs = 5_000
	// Gateway-visible backlog: revisions queued per client / behind the delivered cursor.
	budgetQueueDepthMax  = 8
	budgetCursorLagMax   = 8
	budgetKernelPeakKB   = 256 * 1024 // measured 60-67 MB
	budgetRecoveryMs     = 20_000     // restart begun -> first accepted intent (measured 2-4 s)
	budgetToothRSSFactor = 1.5        // RSS after a restart vs before it
	budgetToothRSSSlack  = 16 * 1024  // kB
)

func checkBudgets(r loadReport) []string {
	var p []string
	over := func(name string, got, budget float64) {
		if got > budget {
			p = append(p, fmt.Sprintf("%s = %.1f exceeds budget %.1f", name, got, budget))
		}
	}
	over("intent latency p95 (ms)", r.IntentLatency.P95, budgetIntentP95Ms)
	over("intent latency p99 (ms)", r.IntentLatency.P99, budgetIntentP99Ms)
	over("event lag p95 (ms)", r.EventLag.P95, budgetLagP95Ms)
	over("event lag p99 (ms)", r.EventLag.P99, budgetLagP99Ms)
	over("error rate", r.ErrorRate, budgetErrorRate)
	over("stale refusal rate", r.StaleRate, budgetStaleRate)
	over("gateway peak RSS (kB)", float64(r.PeakRSSKB), budgetPeakRSSKB)
	over("gateway goroutines", maxf(r.PeakGoroutines, r.IdleGoroutines), float64(budgetGoroutinesPerClient*r.Clients+budgetGoroutinesBase))
	over("gateway file descriptors", float64(r.PeakFDs), float64(budgetFDsPerClient*r.Clients+budgetFDsBase))
	over("gateway CPU during the burst (ms)", float64(r.GatewayCPUMs), budgetGatewayCPUMs)
	over("peak per-client queue depth", r.QueueDepthPeak, budgetQueueDepthMax)
	over("peak cursor lag", r.CursorLagPeak, budgetCursorLagMax)
	over("kernel peak RSS (kB)", float64(r.KernelPeakKB), budgetKernelPeakKB)
	if r.MissingDeliveries > 0 {
		p = append(p, fmt.Sprintf("%d snapshot deliveries were missing across clients", r.MissingDeliveries))
	}
	if r.Reconnects > 0 {
		p = append(p, fmt.Sprintf("%d SSE reconnects during a steady-state burst (a slow-consumer eviction)", r.Reconnects))
	}
	if r.ResyncFrames > 0 {
		p = append(p, fmt.Sprintf("%d resync_required frames during a steady-state burst", r.ResyncFrames))
	}
	return p
}

func maxf(a, b float64) float64 {
	if a > b {
		return a
	}
	return b
}

// checkToothBounds are the restart teeth's resource bounds: memory, goroutines and descriptors
// must not grow because a peer restarted under 50 streams, and recovery must be prompt.
func checkToothBounds(kind restartKind, r toothReport, after scrape) []string {
	var p []string
	if r.RSSPeakKB > budgetPeakRSSKB {
		p = append(p, fmt.Sprintf("gateway RSS peak %d kB exceeds %d kB across a %s restart", r.RSSPeakKB, int64(budgetPeakRSSKB), kind))
	}
	if limit := float64(r.RSSBeforeKB)*budgetToothRSSFactor + budgetToothRSSSlack; float64(r.RSSAfterKB) > limit {
		p = append(p, fmt.Sprintf("gateway RSS after the %s restart %d kB exceeds %.0f kB (1.5x before + 16 MB; before %d kB)", kind, r.RSSAfterKB, limit, r.RSSBeforeKB))
	}
	if limit := r.GoroutinesBefore + 50; r.GoroutinesAfter > limit {
		p = append(p, fmt.Sprintf("gateway goroutines grew %.0f -> %.0f across a %s restart (a leak)", r.GoroutinesBefore, r.GoroutinesAfter, kind))
	}
	if limit := r.FDsBefore + 50; r.FDsAfter > limit {
		p = append(p, fmt.Sprintf("gateway file descriptors grew %d -> %d across a %s restart (a leak)", r.FDsBefore, r.FDsAfter, kind))
	}
	if r.RecoveryMs > budgetRecoveryMs {
		p = append(p, fmt.Sprintf("first accepted intent %d ms after the restart began exceeds %d ms", r.RecoveryMs, int64(budgetRecoveryMs)))
	}
	if r.ClientsResynced != r.Clients {
		p = append(p, fmt.Sprintf("only %d/%d clients resynced", r.ClientsResynced, r.Clients))
	}
	return p
}
