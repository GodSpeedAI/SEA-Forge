# Unit5C cohort dependency-injection reconnaissance — 2026-10-05

Read-only architecture map for root decisions. No coordinator, API, or production
behavior is proposed or implemented here. The response-line cap is implemented;
the historical contract proposal text saying otherwise is stale (see the
append-only `observation-cohort-runlist-response-cap-correction-oct05.md`).

## Existing dependencies and seams

* `main.go:226-229` builds one `projection.LiveSource` around the production
  `authority` and gateway actor. The same `source` is passed to `NewRelay` at
  `:238-241` and as both world and template provider to
  `server.NewWithArtifacts` at `:258-270`. The same concrete `authority` is
  separately passed as the artifact getter and readiness probe, and is
  available at assembly time. No run-observation dependency is currently
  supplied to `server.Options`.
* `LiveSource` stores only `ports.CaseAuthorityPort` and gateway identity
  (`projection/live.go:21-39`). That application port already includes
  `RunsListForCase` (`ports/ports.go:449-459`); the scoped adapter builds the
  existing `run_list` request with `case_id` and validates each returned row
  against that exact case (`adapters/sfwp/authority.go:254-315,317-363`).
* `LiveSource.Facts` calls `RunsListForCase` once on every facts capture
  (`projection/live.go:43-79`), then checks case identity, duplicate/overlap,
  standings, and that each readable run's `PlanItemID` is an actual current
  horizon item (`:80-123`). `Snapshot` calls `Facts` (`:145-151`). The relay
  captures `Facts` on each kernel event (`server/relay.go:101-140`, observed
  in source during this review). Therefore repeatedly obtaining cohort
  candidates through `Snapshot`/`Facts` would re-read the list and fetch the
  rest of the case views. The one-list-per-cohort requirement needs a cohort
  path that consumes a single `RunsListForCase` result rather than recapturing
  `Facts` for each run or each frame.
* Unit5B has introduced the application-level `ports.RunTracePort` with
  `ReadRunTrace(ctx, caseID, runID, planItemID)` and safe semantic result types
  (`ports/run_trace.go:5-29`). It is separate from `CaseAuthorityPort` today.
  The SFWP `Authority` has the method as a typed-unavailable stub
  (`adapters/sfwp/run_trace.go:10-15`); the adapter package remains the proper
  place for wire decoding. The server/projection packages already import
  `ports`, so a future assembly or narrow internal capability can remain
  adapter-agnostic. No current `RunTracePort` field exists in `Options`.
* `server.New` and `NewWithArtifacts` receive the required providers directly;
  `NewWithArtifacts` stores `world`, dispatcher, templates, artifacts, store,
  relay and `Options` (`server/server.go:72-104`). `Options` has an optional
  `Ask ports.AskPort` but no cohort/list/trace provider (`server/http.go:23-50`).
  The current read extension seam is the optional `PerspectiveVerifier`
  interface asserted on `world` (`server/server.go:38-43,385-400`); production
  `LiveSource` implements it by forwarding delegated identity verification
  through `CaseAuthorityPort.ResolveIdentity` with the configured gateway
  actor (`projection/live.go:317-339`). This preserves one world object for
  snapshot and verifier behavior. Adding a required method to `WorldSource`
  would affect its fakes; optional capability assertion or explicit `Options`
  injection are existing patterns to evaluate, not a selected design here.
* To avoid both repeated list reads and raw SFWP imports, keep the coordinator
  at an application/server/projection boundary and depend on the existing
  `ports.CaseAuthorityPort` scoped-list method plus the new `ports.RunTracePort`
  contract. Whether those dependencies are surfaced by a `LiveSource`
  capability or injected explicitly through `Options` is unresolved in this
  reconnaissance. Do not make server/projection import `internal/adapters/sfwp`.
  Whichever seam is selected must call the scoped list once per cohort, then
  pass the actual listed `(case, run, plan-item)` identities to the Unit4
  validated trace read; it must not call `Facts` merely to repeat ownership
  checks already established by the assigned Unit4 contract.

## Session and SSE state that integration must preserve

`requireSession` resolves the cookie/bearer identity once and attaches it to
the request context (`server/session.go:85-94,126-141`). `/api/events` rejects
identity query overrides, derives the actor from that attached session and
calls `verifySessionPerspective` before subscribing (`server/server.go:293-318`).
The stream subscribes before `hello`; retained snapshot replay follows, using
the original kernel cursor as each snapshot event's SSE `id` (`:315-324,327-374`).
`writeSSE` already omits an `id:` when passed an empty ID
(`server/http.go:184-198`).

The stream loop ends when the request context is cancelled, its revision
subscriber is evicted, or it otherwise returns; it defers the Store unsubscribe
(`server/server.go:317-318,345-375`). Logout destroys the session and clears
cookies (`server/session.go:374-385`), but there is no session-revocation signal
to an already-open SSE request. Expiry is enforced on a later `Resolve`; the
periodic sweep deletes expired records but does not cancel streams
(`auth/session.go:114-160`, `main.go:253-256`). Thus logout/expiry after SSE
connection establishment does not currently close that connection. New cohort
reads can use the current verified request actor and `r.Context`, but any
long-lived observation subscription must not imply stronger immediate logout
revocation than the present SSE/session machinery provides; lifecycle semantics
remain a root decision.

The existing snapshot SSE event owns `id=<kernel cursor>` and `Last-Event-ID`
replay (`server/server.go:308-374`). The approved cohort contract requires the
observation side channel to omit SSE `id` so it cannot advance that resume
position (`t09-contract-extension-proposal.md:31,35`). Do not append trace data
to `projection.Store` revisions or captured `CaseFacts`; retained facts are
historical cursor snapshots (`projection/store.go:26-48`).

## Likely fixture/test touchpoints and risks

* Projection list-call semantics are covered by
  `projection/scoped_live_test.go:11-90` and real-stack scoped checks in
  `projection/scoped_runs_live_test.go`. Add/adjust any cohort test at the
  coordinator seam to count exactly one scoped list and reject the unscoped
  method; do not weaken existing facts/horizon checks.
* Adapter trace assertions are in `adapters/sfwp/run_trace_test.go`; the current
  `Authority.ReadRunTrace` implementation is explicitly still unavailable.
  Unit5C integration must preserve the validated exact run and real parent
  identities from Unit4 rather than re-decoding adapter wire data elsewhere.
* Server test fakes are centered on `server_test.go:27-60` and construct through
  `New`/`NewWithArtifacts`; adding a required `WorldSource` method would force
  unrelated fake changes. `intent_authorization_test.go:141-178` deliberately
  wraps a world to hide `PerspectiveVerifier` and asserts fail-closed behavior;
  capability composition must not accidentally make this wrapper pass
  verification.
* Session-perspective and historical/SSE behavior is exercised in
  `session_read_test.go:76-199`; SSE resume/resync is exercised in
  `server_test.go:640-693`; live native stream coverage is in
  `native_events_live_test.go`. Observation frames should be tested as separate
  no-ID events while snapshot `id` replay, actor verification, and revision
  history remain intact.
* Main assembly is exercised in `production_test.go`; `main.go` currently
  creates the same `LiveSource` for relay and HTTP and the same Authority for
  artifacts/readiness. Any new dependency should be assembled there without
  bypassing preflight or changing fixture-only build boundaries.

Material limits remain: scoped `run.list` is still a global filesystem scan
before filtering, so one request bounds logical list-call count but not kernel
enumeration work. The client cap bounds each response line only; neither that
cap nor per-run read counts bound total cohort transport/journal work. The
existing list response has arrays and no source-enumeration completeness proof;
do not label a successful response as proof that every upstream directory or
journal was read.

Graft retrieval preceded direct source validation; its first query refreshed
one changed file before answering. Three successful queries saved approximately
95,086 tokens (~$0.07) this turn. No compiler, tests,
source changes, status/debt updates, or Git mutations were made.
