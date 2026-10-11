# Physical run.get admission repair proposal

Date: 2026-10-06  
Status: proposal for root decision and fresh independent review; no implementation released.

This repairs the six gaps listed in `run-trace-physical-admission-independent-review-oct06.md`. Root retains all architecture and semantic decisions. Every API below is proposed, not an existing interface. It adds no dependency, kernel verb, public HTTP shape, or identity change.

## Current source facts

- `internal/adapters/sfwp/client.go:35-74` defines `Config` with `SocketPath`, `RequestTimeout`, connection limits, and `Dial`; `New(cfg Config) (*Client, error)` is at `:131-137`. There is no admission hook today.
- `Client.Do(ctx context.Context, req *Request) (*Response, error)` is at `:395-417`. One logical call may invoke `roundTrip` initially, retry a transport-safe read once inside `roundTrip`, then retry a busy refusal in `Do`, whose second `roundTrip` can itself make a safe retry: up to four physical attempts.
- `roundTrip` creates its `RequestTimeout` context before `acquire` (`:419-430`), calls `conn.call` (`:428`), then discards/releases the connection on its error/success branches (`:430-461`). `conn.call` performs the request-byte `Write` and newline `WriteString` at `:224-230`; a failed/partial write closes the connection. Its deferred callback join (`:197-215`) finishes before `call` returns.
- `NewRunGet(runID)` constructs the request at `frame.go:308-313`. Direct run_get sends occur in `ReadRunTrace` at `run_trace.go:72` and in artifact ownership resolution in `Authority.GetArtifact`, `authority.go:758-765` after `artifact_get`.
- Production CLI assembly creates a client for every configured live capability in `cmd/godspeed-casework/main.go:86-100`; serve mode selects one later. Test assembly creates clients in `internal/livetest/harness.go:245-259` and `internal/livestack/stack.go:61-82`.
- `ReadRunTrace(ctx, caseID, runID, planItemID)` is the adapter method; its port declaration is `internal/ports/run_trace.go:5-8`. The reviewed source has no production hydration caller, so this proposal does not claim hydration or T09 completion.

## Proposed API and physical boundary

Proposed adapter-local declarations in package `sfwp`:

```go
type RunGetAdmission interface {
    AcquireRunGet(ctx context.Context, runID string) (RunGetPermit, error)
}

type RunGetPermit interface {
    WriteAttemptStarted()
    WriteAttemptFinished()
    Release()
}
```

`Config` would gain `RunGetAdmission RunGetAdmission`. The field is required in every production SFWP client construction; nil remains supported only for existing isolated test clients that do not represent production wiring. `AcquireRunGet` is called only for `req.verb == "run_get"`, using the exact run ID from the already-encoded request. All other verbs bypass it. Admission errors map to the existing typed unavailable error path; no public refusal reason is added.

For each physical `roundTrip` send, acquire one permit. Immediately before the first request-byte write in `conn.call`, call `WriteAttemptStarted`; after the request-byte and LF write attempts have both returned (or the first write failed), call `WriteAttemptFinished` exactly once. The finish event is the cooldown origin. A partial request write, a partial LF write, or a write error therefore consumes the attempt and starts cooldown when that write returns. Cancellation before the first `Write` has no started attempt and incurs no cooldown. The connection remains unusable after any write error, as today. Each safe retry and busy retry obtains a fresh permit; no retry inherits an earlier permit.

This counts attempted socket writes, not successful complete protocol lines. This distinction avoids assuming that a failed `Write` sent zero bytes. The permit implementation must be idempotent against duplicate finish/release calls and treat finish without start as no attempt.

## Two-record algorithm and bounded state

One process-shared admission owner stores exactly two records. Each record contains only `{runID string, busy bool, nextEligible time.Time}`; no unbounded map, pending queue, or per-session timer is kept. A short mutex protects the records and a replace-on-signal channel. Waiters hold neither mutex nor connection while waiting.

Acquisition under the mutex is deterministic:

1. If any record names this exact run ID, that run is unavailable until that record is idle and `now >= nextEligible`; a second record cannot bypass its same-run exclusion.
2. Otherwise select the first empty record. If none exists, select the eligible idle record with the earliest `nextEligible` (slot index breaks ties). A busy record or idle record whose cooldown has not expired is never evicted or overwritten.
3. If the selected record is eligible, write `{runID, busy:true}` and return its permit. Otherwise wait on the current change channel, the nearest `nextEligible` deadline among idle records, or `ctx.Done()`, then retry under the mutex. Signaling closes/replaces the channel whenever permit state changes.
4. On `WriteAttemptFinished`, set `nextEligible = finishTime + 1s`, retaining the run ID and busy state. `Release` sets busy false and retains both run ID and cooldown. Empty records remain empty until assigned. Thus no unexpired per-run cooldown is forgotten, and exactly two run identities at most are retained. A third distinct run waits until an idle record's cooldown expires before replacing it.

This two-record retention and one-second post-write cooldown is intentionally conservative: aggregate throughput may be limited to about two attempts per second, and a third distinct run may wait behind retained cooldown. That is a deliberate bounded-memory compromise, not an original throughput requirement. Counts for candidate runs omitted by the independent selection policy remain honest and are not fabricated as read/unavailable results.

## Cleanup and cancellation order

The call helper that owns one physical attempt owns both permit and connection lifecycle. Register the permit-release defer first, then register connection cleanup; Go's LIFO defer order consequently runs connection cleanup before permit release. Cleanup covers `conn.call` return (therefore its `context.AfterFunc` callback join), then `Client.discard(cn)` or `Client.release(cn)`, and only then permit `Release`. For safe retry, the first attempt helper returns only after this sequence; the retry then acquires its own permit/connection. Busy retry likewise enters a new helper after its existing context-aware backoff. No release goroutine is used.

The current `roundTrip` acquires a connection before `conn.call`. Acquiring admission after that would occupy a pooled socket while queued. The proposed order is admission first, then connection acquisition, still under the existing `RequestTimeout` context (created before either wait). This is a material deviation from the root proposal's after-connection placement and needs explicit root adjudication before fixture release. If root chooses after-connection ordering instead, it must accept and document socket occupancy while waiting; the hook/permit protocol and cleanup proof still apply.

An acquired permit canceled before any write is released without cooldown. After a write begins, finish accounting and connection cleanup still complete before release. Cancellation stops future cohort candidates and the caller drains/joins any outstanding call under its existing context; no new cohort deadline is introduced.

## Production injection and logical budget

Production assembly creates one process-shared admission owner before the live-client loop in `cmd/godspeed-casework/main.go:86-100` and supplies that same owner in each `sfwp.Config`, including clients for capabilities not selected by serve mode. The selected client's `Authority` receives no special bypass. `Authority.GetArtifact`'s internal `run_get` at `authority.go:758-765` is covered by the same `Client.Do` hook. The `livetest` and `livestack` constructors are test assembly and may use a test-owned owner or nil according to fixture purpose; they are not production coverage. Any future production `sfwp.New` site must provide the shared owner.

The manager/cohort owner, not this physical limiter, enforces the approved cap of eight **logical** `ReadRunTrace` attempts per connection cohort. Each invocation of `ReadRunTrace` by that manager consumes one logical unit when invoked, even if it times out waiting for physical admission. Its transport or busy retries remain within that unit. Admission waits and retries do not consume extra logical units. Once eight calls have been invoked, the manager starts no additional candidate; it does not silently fan out or select extra candidates. Same-run spacing applies to retries as well as initial attempts.

If an admission wait or read fails, the cohort records that selected run as unavailable under the existing result contract, stops starting further candidates on session cancellation, and drains outstanding calls. It reports only actual completed, omitted-by-selection, and unavailable counts; partial hydration stays partial and is never represented as a complete cohort. No automatic extra candidates or new cohort deadline are introduced. The per-call `RequestTimeout` includes admission wait because the existing timeout context is established before `Client.acquire`; retain that same parent context before admission in the proposed ordering. Root must confirm these manager-result semantics against the approved cohort contract before fixtures.

## Root decisions required before fixture release

1. **Admission placement:** approve admission-before-connection (recommended to avoid holding scarce pooled sockets while queued), or retain after-connection and accept socket occupancy while queued.
2. **Strict two-record retention:** approve waiting to replace a record until its cooldown expires, including delaying a third distinct run; the cost is lower aggregate throughput and possible ordinary per-call timeout/unavailable outcomes.
3. **Cohort failure representation:** approve marking a selected timed-out/admission-failed read unavailable, stopping further candidates only on session cancellation, and preserving partial counts without a new deadline. Confirm this conforms to the approved cohort response contract.
4. **Hook surface:** approve the adapter-local `RunGetAdmission` / `RunGetPermit` method signatures and `Config` field, or provide exact replacements. These names are proposed, not existing API.

After root decisions, a fresh independent review should verify the source anchors, exact defer ordering and all construction/send sites before any fixture or implementation work. This proposal records no approval, test result, or completion claim.
