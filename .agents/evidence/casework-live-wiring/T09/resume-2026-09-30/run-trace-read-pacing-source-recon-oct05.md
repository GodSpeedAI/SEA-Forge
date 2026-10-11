# Run trace read pacing source recon (2026-10-05)

## Scope

Read-only call-path inventory for the approved T09 hydration budget and read pacing. No runtime, tests, schema, status, or Git state was changed. This identifies source hooks and physical-send behavior; it does not choose an architecture.

## Current call graph and gaps

`ports.RunTracePort.ReadRunTrace(ctx, caseID, runID, planItemID)` is declared at `apps/godspeed-casework-go/internal/ports/run_trace.go:5-8`; the SFWP adapter implements it at `internal/adapters/sfwp/run_trace.go:66-83`. It invokes `a.client.Do(ctx, NewRunGet(runID))` once, then validates identity/status/trace and projects allowlisted frames (`:84-176`). Repository search found no production caller from the server, projection, or command wiring. The method is therefore an adapter capability today, not an active hydration manager with a session/global limiter. The Go server has injected world/intents/template/artifact dependencies but no `RunTracePort` field (`internal/server/server.go:72-84`); the current live world path only fetches scoped `run.list` summaries (`internal/projection/live.go:76-78,124-132`). Canonical UI types describe `hydration_read_budget.limit = 8` and counters (`.agents/reports/interface-contracts/typescript/types.ts:353-372`), but this recon found no Go server/manager implementing that logical budget.

## Logical calls versus physical sends

`ReadRunTrace` makes one logical `Client.Do`. `Client.Do` first sends via `roundTrip`, then has a separate bounded retry for an explicit busy refusal (`client.go:395-416`). `roundTrip` gives each call a request-timeout context (`:419-428`). On a transport failure, safe read requests reconnect and resend once if that context remains live (`:438-448`). `Request.IsTransportRetrySafe` is true for non-mutations except `ask` (`frame.go:93-99`), so `run_get` uses this retry. Consequently a per-run interval enforced only before `ReadRunTrace`/`Client.Do` does not pace its immediate transport retry. Because the outer busy retry calls `roundTrip` again, and that call has its own single safe transport retry, the source permits up to four physical sends in one `Do` (first call plus its safe retry, followed by the busy retry call plus its safe retry). A physical-send hook would need to account for those paths if the intended bound is per actual `run.get` send. No such hook/admission callback exists in `Client` currently.

Cancellation propagates through `ctx` to acquire/call, and a canceled transport failure skips retry (`client.go:421-428,438-449`). Existing tests pin this: `TestManualCancellationInterruptsRunGetAskAndMutationWithoutRetry` (`client_cancellation_test.go:190-238`) and `TestCanceledPartialRunGetConnectionIsNotReused` (`:310-370`). Any send gate must preserve context cancellation while waiting and avoid changing the request’s retry-safety classification.

## Nearby blast radius

The shared `Client.Do` serves inspect reads and Ask. Ask is explicitly not transport-retry-safe because it writes a disclosure record without a correlation ID (`frame.go:93-98`); it still receives the explicit busy retry in `Do` (`client.go:404-412`). A client-wide callback must therefore discriminate `run_get`, or existing Ask/mutation behavior and cancellation tests are in scope. Existing inspected retry controls are also pinned by `client_test.go:312-318` and response-cap retry coverage at `response_limit_test.go:258-284`. Server Ask has separate per-session and per-IP limiters (`server.go:81-84`), unrelated to an adapter-level run-read interval.

The T09 UI contract’s eight-read hydration budget is a logical cohort count, while the requested one-second rate is a physical `run.get` concern; current source does not join them because no hydration caller exists. No new wire fields or runtime implementation are inferred here.
