# Casework live wiring: load budgets and restart teeth (plan T11, part B)

Run it: `just casework-load` (exit 1 on any budget or invariant violation). Harness:
`apps/godspeed-casework-go/internal/loadtest/` (build tag `load`; `rig_test.go` boots the stack and the
clients, `load_test.go` holds the three tests, `budgets_test.go` holds every number below as a
constant). Evidence: `.agents/evidence/casework-live-wiring/T11/`.

## What is measured

Each test boots a FRESH temp cell with the real `sea-forge-server` kernel and the real
`godspeed-casework -serve` gateway binary, as separate OS processes (the gateway also serves the
metrics listener). N dev-auth users log in (N distinct usernames, all mapped to `operator_local`)
and each holds one `/api/events` stream with an EventSource-equivalent client (reconnect with
`Last-Event-ID`, re-login after a 401).

- Intent latency: `POST /api/intents` round trip, `ADD_DISCRETIONARY_WORK` (a cheap governed plan
  mutation). 100 intents over 20 cases; one worker per case (an intent carries the cursor the
  previous one returned), 20 intents in flight at once. A `STALE_PROJECTION` refusal is retried
  with the refusal's fresh cursor (the documented client protocol); latency spans the retries and
  the refusals are counted and budgeted, not hidden.
- Event lag: client receipt time minus the kernel append time of that revision. The SSE id is the
  kernel's entry ULID, whose 48-bit prefix is the kernel's append timestamp in milliseconds (one
  host, one clock). Every delivery to every client is a sample (50 x ~100 = ~5000 per run).
- Error rate: intents that fail for any reason other than a retried stale refusal.
- Gateway RSS, threads and descriptors from `/proc/<pid>` every 250 ms; goroutines, heap, queue
  depth and cursor lag from the new metrics endpoint (`casework_go_goroutines` and
  `casework_go_heap_inuse_bytes` were added for this); gateway and kernel CPU from `/proc/<pid>/stat`.
- Stream invariants, every run: no duplicate revision id on any client, ids strictly increasing,
  every client saw the identical revision sequence, no reconnect and no `resync_required` during a
  steady-state burst.

## Host and conditions

Intel Core Ultra 5 226V (6 logical CPUs), 7.9 GB RAM, WSL2 Linux 6.18, Go 1.27.1, debug-profile
kernel (`target/debug/sea-forge-server`). The host was NOT quiet: other tools (bun, node,
postgres) were using roughly 1-3 cores at times (load average 4-5 on 6 CPUs), and `/tmp` is a
94%-full tmpfs. The latency numbers carry that noise, which is why the latency budgets are loose
tripwires. The kernel is CPU-bound during the burst (60-89 s of kernel CPU in a 20-37 s burst, about
3 cores) while the gateway spends under 1.1 s of CPU: at 20 concurrent intents the kernel, not the
gateway, sets throughput (about 3-4 intents/s) and therefore latency (about 20 / throughput).

## Measured baselines and budgets (default scale: 50 SSE clients, 100 intents, 20 cases)

10 runs at the default scale (7 during calibration and development, 3 final `just casework-load`
runs; calibration runs predate only the read-only lock-open tweak in the ledger fix, which does not
change the measured paths). Min / median / max:

| Metric | Measured (min / median / max) | Budget | Headroom rationale |
|---|---|---|---|
| Intent latency p50 | 3.9 / 5.4 / 6.9 s | none | reported only |
| Intent latency p95 | 6.3 / 8.6 / 11.1 s | 20 s | 1.8x worst run |
| Intent latency p99 | 8.0 / 10.1 / 11.9 s | 25 s | 2.1x worst run (n=100, so p99 is near the max) |
| Event lag p50 | 2.4 / 3.7 / 8.6 s | none | reported only |
| Event lag p95 | 4.2 / 11.5 / 23.8 s | 40 s | 1.7x worst run, which coincided with the heaviest host contention |
| Event lag p99 | 5.0 / 12.4 / 25.6 s | 45 s | 1.8x worst run |
| Error rate | 0 in all 10 runs | 0 | any failure that is not a retried stale refusal fails the run |
| Stale refusals retried per intent | 0.00 / 0.06 / 0.13 | 0.20 | 1.5x worst; shows the stale guard works under a lagging relay |
| Gateway RSS peak | 47.0 / 47.5 / 48.1 MB | 96 MB | 2x; Go holds its heap, so idle with streams open is the same |
| Goroutines (peak or idle) | 131 (137 once) | 4 per client + 60 = 260 | ~2x; measured ~1.8 per client + ~40 |
| File descriptors peak | 85 (91 once) | 3 per client + 60 = 210 | ~2.3x |
| Peak per-client queue depth (gateway gauge) | 0 | 8 | a client more than 8 behind is a fan-out regression |
| Peak cursor lag (gateway gauge) | 1 | 8 | same |
| Gateway CPU during the burst | 0.73 / 0.96 / 1.04 s | 5 s | ~5x; the per-delivery render guard (see redesign trigger) |
| Kernel RSS peak | 62 / 64 / 68 MB | 256 MB | 3.7x |
| Missing deliveries, reconnects, `resync_required` frames | 0 | 0 | structural |

Variance. The structural numbers (RSS, goroutines, descriptors, gateway CPU, queue depth) vary by
under 5% between runs. The latency numbers vary by about 2x for the same code (event lag p95 4.2 to
23.8 s), driven by host contention and by how many stale refusals a run happens to absorb. They are
tripwires for order-of-magnitude regressions, not service-level targets. They are not a statement
about production hardware: this is a debug-profile kernel on a shared 6-CPU laptop-class host.

## The redesign trigger (per-client projection rebuild) is NOT met

The trigger reads: subscription lag exceeds budget because of the per-client projection rebuild
(then share projections per cursor). The evidence says the lag is not caused by client fan-out:

- Paired runs of the same burst with 1, 10 and 50 clients
  (`T11/fanout-comparison/`, two runs each, medians): event lag p95 9.0 / 11.8 / 5.8 s and
  intent p95 8.1 / 9.1 / 7.3 s. Lag does not rise with client count (it is not monotonic: it is noise
  around the kernel-bound throughput). Gateway CPU does rise with clients, 0.33 / 0.49 / 0.87 s,
  which is the rendering cost: about 0.17 ms per delivery, under 1 s for 5000 deliveries.
- The gateway-visible backlog never builds: per-client queue depth 0, cursor lag 1.
- The queue that grows is the relay's serial per-frame rebuild behind a CPU-saturated kernel
  (`Relay.accept` reads the case facts from the kernel once per frame, then every client renders
  the shared facts at write time). That is per frame, not per client, and it is bounded by kernel
  throughput.

So projections are not shared per cursor yet, and the budget on gateway CPU (5 s) is the guard that
will say when to do it: a change that makes per-client rendering 5x costlier trips it before users
see lag.

## The restart teeth (T11: "Restart the server under 50 SSE clients")

Both teeth are run by `just casework-load`, three times each in the final evidence. Procedure: 50
clients connected, 8 cases, 24 warm intents; then 80 intents in flight, and 2 s into the burst the
process is killed with `kill -9`. Intents in flight fail (honestly: 66-71 of 80 in the kernel-kill
runs, 70-76 in the gateway-kill runs). The process is restarted on the same cell, each case's world
is refetched for a fresh cursor, one probe intent is retried until accepted, and then 40 more
intents run.

Assertions (all enforced, none relaxed): every client receives a revision newer than its position at
restart; no duplicate revision id on any client over the whole run; ids strictly increasing; every
client saw the identical sequence; every intent the gateway acknowledged is durable in the kernel
ledger (`plan_mutated` count >= acknowledged); the post-recovery burst has zero errors; the gateway
reports exactly 50 open streams afterwards; gateway RSS peak <= 96 MB and RSS after <= 1.5x before
+ 16 MB; goroutines and descriptors grew by under 50; first accepted intent within 20 s of the
restart beginning.

| Tooth | Result (3 final runs) |
|---|---|
| Kernel `kill -9` mid-burst | 50/50 clients resynced in every run; 0 reconnects (the gateway keeps the streams open and its subscription recovers); first accepted intent 0.9 / 1.1 / 1.2 s after restart; RSS 25-29 MB before, 28.7-29.7 peak, 28 after; goroutines 119-126 -> 119; fds 73-80 -> 73; acknowledged 73-78 intents, durable 74-79 (one in-flight intent committed without being acknowledged, never the reverse); an intent during the outage returned 502 in 1 ms |
| Gateway `kill -9` mid-burst | 50/50 clients reconnected with `Last-Event-ID` after re-login (sessions are in memory); 0 `resync_required`; first accepted intent 1.1-1.6 s after restart; RSS 25-27 MB before, 47.5-48.4 peak (the restarted gateway re-learns history from the kernel), 26-29 after; goroutines and fds back to 119 / 73-76; acknowledged 68-72, durable 76-82 |

Both pass. Negative controls, kept as evidence: removing the resume watermark fixed below makes the
gateway tooth fail with 552 duplicate / non-monotonic violations
(`T11/negative-control-no-resume-watermark/`); a 1 s p95 budget makes `just casework-load` exit 1
(`T11/negative-control-p95-budget-1000ms/`).

## Product defects the load test found and fixed

1. Gateway: duplicate and out-of-order SSE revisions after a gateway restart. A client reconnecting
   with `Last-Event-ID` against a restarted gateway subscribed to an empty store; the relay then
   re-learned history from the kernel and `Store.Append` pushed every historic revision (cursor at or
   before the client's position) to the client as if live. Fix: `projection.Store.Subscribe` records a
   per-subscriber watermark and `Append` never delivers at or before it. Test:
   `TestStoreSubscribeNeverRedeliversAtOrBeforeTheResumePoint`.
2. Kernel ledger: readers raced appenders. `LedgerStream::read_entries`, `verify` and the readers in
   `create_pre_action_assurance` read `entries.jsonl` without the stream lock, so a concurrent append
   showed up as a torn last line (`parse entry: EOF while parsing a string`) and the unrelated
   request was denied as `AUTHORITY_DENIED serialization_error`. Fix: public `read_entries`, `verify`
   and the checkpoint readers take a shared flock (read-only open; absent or unreadable lock file
   falls back to the old behaviour); callers already inside the exclusive lock use `*_unlocked`.
   Test: `verify_never_sees_a_half_written_append` (probabilistic: it failed 1 run in 3 before the
   fix and in none of the more than ten runs since).
3. Kernel ledger: `materialize_aggregate_view` wrote every view through `<path>.tmp`, so two
   concurrent writers of one view path made the loser fail with `replace view: No such file or
   directory`. Fix: a unique temporary name per writer. Test:
   `concurrent_view_materializations_of_one_path_all_succeed` (fails deterministically before the fix).

## Findings that are not fixed here

- The kernel's per-intent cost grows with the number of ledger streams (`create_pre_action_assurance`
  verifies every stream), about 0.6-0.9 CPU-seconds per intent at 20-30 cases in a debug build. That
  is what sets the 3-4 intents/s ceiling. It is a kernel scaling item, outside the gateway.
- `STALE_PROJECTION` refusals occur for a client that posts its next intent immediately after the
  previous response, because the relay publishes the case cursor on the first retained frame and a
  mutation emits several more frames after. The protocol (refetch and retry) absorbs it; the rate is
  budgeted (0.20) rather than treated as an error.
- The pre-existing `TestLiveSubscriptionResumeAcrossRestart` (sfwp, `-tags live`) failed once in
  about seven runs (`events.get_range ... already-delivered frame`) and passed the other times,
  including the full package twice; it was run under host contention and looks like a test-ordering
  flake (it is unrelated to the ledger lock), but that is not proven.

## Not done

- Budgets are calibrated on one shared host; they are not production capacity numbers, and CI hosts
  will need their own calibration run (the latency budgets are 1.7-2.1x the worst observed run).
- No run at a scale above the default (the knobs exist: `LOAD_CLIENTS`, `LOAD_INTENTS`,
  `LOAD_CASES`, `LOAD_TOOTH_CLIENTS`); only 1 and 10 clients were compared.
- Memory is measured as RSS over runs of tens of seconds; a long soak for slow leaks is not part of
  this task.
- The gateway's `/metrics` was scraped for gauges, but SSE write-side lag per client is only the
  gateway histogram (record to write), which includes the relay wait; there is no separate
  store-append to socket-write timing.
