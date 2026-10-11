# T09 reconnect and resource budget proposal review

Date: 2026-09-30

## Assignment

Revise the nonnormative T09 contract proposal following independent review. Allowed changes were limited to `t09-contract-extension-proposal.md` and this evidence note. No source, test, contract, normative spec, runtime, build, compile, or test changes were made.

## Source findings

- Go SFWP client `internal/adapters/sfwp/client.go:175-205,231` reads a response line using `bufio.Reader.ReadBytes('\n')` and `bufio.NewReader(nc)`. There is no current response-line byte cap. The earlier proposal's implication that an existing cap could remain unchanged was incorrect.
- `sfwp.Config` already has per-client tuning fields (`MaxConns`, timeouts, backoff, etc.) at `client.go:26-55`. The revised proposal asks for a new 32 MiB default response-line ceiling, optionally lowerable by per-client config and never raisable above the ceiling. It treats this as an operator-approved resource-contract prerequisite, not existing behavior.
- `Client.roundTrip` at `client.go:315-367` retries inspect transport failures once and routes correlated mutation transport failures through request-status recovery. Proposed capped reads must close/poison the connection, return typed `unavailable` before JSON decode, preserve safe inspect retry, and never re-send a mutation.
- Rust defines `MAX_RECORD_BYTES = 4 MiB` and `MAX_JOURNAL_BYTES = 64 MiB` per file at `crates/sea-forge-server/src/sfwp/mod.rs:55-74`. `run_views.rs:425-432` applies the journal cap independently. `run.get` assembles trace, declarations, evidence, records, plan, settlement, and other projections, so no 64 MiB aggregate response or upstream aggregate read bound exists.
- Existing kernel `run_list` accepts `case_id` (`crates/sea-forge-server/src/lib.rs:867-872,2404-2407`) and scoped conformance verifies owned-run filtering (`tests/conformance_run_views.rs:387-404`). Go currently calls `NewRunList()` without that filter (`internal/adapters/sfwp/frame.go:288-295`) then filters locally (`internal/projection/live.go:71-83`). Case scoping can use the existing field; Rust still walks all run directories (`run_views.rs:452-500`) and exposes no pagination.
- `run.list` is newest-first after summary construction (`run_views.rs:777-833`). Execution standing values are `pending`, `enabled`, `active`, `completed`, `failed`, and `terminated` (`case_views.rs:55-69`). The proposal orders active first, other nonterminal second, terminal newest-first, then validates each selected run's returned `run_id` and `case_id` before projecting metadata.
- The UI reconnect closure preserves `lastCursor` across internal reconnects, but current routing uses ordinary case cursor dedupe (`httpCaseworkAdapter.ts:297-377,380-445,530-549`). Run observation dedupe must be separately bounded and must not alter case resume behavior.

## Revised proposal limits

- Per hydration cohort: one scoped `run.list`; at most eight selected run summaries and eight initial `run.get` reads; no automatic fanout for omitted runs; deterministic active-first/recent-terminal selection; exact known omitted/unreadable/unavailable counts when listing succeeds.
- If `run.list` cannot complete or exceeds the approved 32 MiB response-line cap, expose typed `unavailable` with counts absent, never an empty complete result. `run.get` denial, mismatch, unreadable or over-cap response contributes unavailable state and no projected frames.
- Process-wide: at most 16 shared `(case_id, run_id)` pollers, two concurrent `run.get` calls, and a one-second minimum interval per run. Pollers stop on terminal standing/no subscribers and cancel/drain outstanding reads.
- Client buffer bound for approved 32 MiB responses: eight initial reads mean at most 256 MiB cumulative response-line bytes per hydration cohort; two simultaneous lines mean at most 64 MiB raw response buffers, excluding JSON decoding expansion. This is not an upstream read/CPU bound: Rust can read several independent capped files to construct a response, and run-list directory enumeration remains global/unpaginated.
- Projected initial hydration envelope: at most 1 MiB across selected runs; trim oldest metadata deterministically and expose exact omitted counts. Keep at most 1,024 metadata frames per run; poll deltas contain only new IDs. An indefinitely open SSE connection can still transfer unbounded cumulative bytes over time.
- UI subscription cache: at most 32 run keys, 4,096 `(run_id,event_id)` identities total, and 1,024 per run. Evict terminal keys/old identities by LRU; preserve active entries, surface a capacity notice and drop unadmittable run frames when active capacity is saturated. Dispose/case switch clears the cache. Deduplication is only guaranteed while the identity remains cached; a replay after eviction may be delivered again.

## Corrections and limitations

- Removed the incorrect 512 MiB run.get read-work claim. 8 x 64 MiB is not a valid aggregate bound because multiple journals and records are independently capped and the Go response line was previously unbounded.
- The 32 MiB response-line cap, cohort/poller/UI limits, and Ask rate/body limits remain proposed and require operator approval. The cap changes behavior for all Go SFWP responses. If the cap is not approved and implemented, T09 must not claim bounded response allocation or start run hydration.
- No Rust request verb, schema, pagination, dependency, policy, authority, or identity redesign is proposed. The existing `run_list` optional case filter and `run_get` are reused.
- Ask proposal details remain: actual `verb: "ask"` wire tag, finite QuestionKind allowlist, 500 UTF-8-byte purpose bound, authenticated delegated actor, complete Thoth disclosure DTO, dedicated limiter, and explicit response/body limits.

## Result

Updated the proposal only. No compile, test, build, vet, runtime, browser, or HTTP command was run. This is a design proposal awaiting a fresh independent review and operator decision; it is not implementation evidence or approval.
