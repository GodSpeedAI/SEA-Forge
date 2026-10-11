# Observation session revocation edit inventory

Date: 2026-10-05. Source-only audit of the current checkout against the
approved T09 observation lifecycle proposal and the current safe-trace port
proposal. This is an edit/test inventory, not an architecture decision. No
source, test, status, debt, or Git files were changed; no tests/compiler/scans
were run.

## Current behavior and deletion paths

- `internal/auth/session.go:22-47` stores shared `*Session` values under one
  `SessionStore.mu`. `Resolve` (`:114-132`) checks absolute and idle expiry,
  deletes expired entries, otherwise mutates `LastSeen` before returning the
  stored pointer after unlock. It is sliding and its returned pointer is not a
  detached snapshot.
- All current removal paths are `Resolve` expiry (`:124-127`), explicit
  `Destroy`/logout (`:134-138`, called at `internal/server/session.go:376-385`),
  `Sweep` (`:149-161`), and `evictOldestLocked` on capacity in `Start`
  (`:102-105`, `:164-175`). These only delete map entries; none signal a live
  request. `Start` evicts the oldest `LastSeen` when full. Expiry is strict
  `now.After(ExpiresAt)` / `Idle` (`:32-34`), and production sweep cadence is
  one minute (`cmd/godspeed-casework/main.go:301-316`). Thus exact-deadline and
  sweep-detection timing are observable contract choices, not immediate
  revocation today.
- `GET /api/events` is session-or-dev-bearer wrapped (`server.go:138`;
  `session.go:118-136`). `handleEvents` (`server.go:293-376`) verifies kernel
  perspective once, then only subscribes to case revisions and request
  cancellation/heartbeat. Logout does not cancel the HTTP request. `Store`
  subscription cancellation (`projection/store.go:185-210`) removes only the
  revision subscription; Store overflow closes that channel. Neither is a
  session or trace-read cancellation signal.
- The current `RunTracePort` declaration exists, but SFWP implementation is
  still an unavailable stub (`internal/adapters/sfwp/run_trace.go:10-15`). No
  observation worker/read lifetime is currently owned by `handleEvents`.
  Existing cancellation assignment requires a blocked `run_get` read to be
  interrupted on context cancellation and drained before read/pool capacity is
  released (`observation-cancellation-test-first-assignment.md`); this is a
  prerequisite, not proof of current behavior.

## Smallest source seams to evaluate

1. **Non-sliding current lookup:** add a `SessionStore` membership/expiry check
   adjacent to `Resolve` that runs under `mu`, does not update `LastSeen`, and
   returns no alias to `*Session` (or only a detached immutable scalar view).
   It must apply the same idle and absolute expiry policy and remove/notify on
   expiry. `Resolve` cannot serve this purpose because every call slides idle
   expiry. Avoid reading `requestIdentity.Session.LastSeen` outside the lock.

2. **Stable revocation signal:** session watch registration and current check
   need to be atomic with respect to deletion; otherwise deletion can occur in
   the check-to-register gap. Every deletion route above must revoke the same
   per-session signal: `Destroy`, expiry in `Resolve`, `Sweep`, and capacity
   eviction in `Start` via `evictOldestLocked`. Repeated removal/close must be
   safe. The store currently has one mutex; keeping registry membership and
   one-shot signal state under that same mutex avoids introducing a second
   lock order. Do not call user callbacks or wait for read drains while holding
   it. After unlocking, cancellation may signal worker contexts; the handler
   must own and join/drain its reads before dropping subscriber/poller
   references. Any implementation using a separate registry lock must state
   and test one global order; current sources do not define one.

3. **Observation branch in `handleEvents`:** preserve existing perspective
   verification and snapshot replay. For cookie sessions, own a revocation
   watch and a per-connection cancellation/drain boundary around initial
   `run.list`/`run.get` work and poll reads; revalidate current session and
   effective kernel perspective before protected reads and before fanout as
   required by `observation-runtime-source-review-notes.md`. Select behavior
   for dev bearer separately (`Session == nil`). No current code provides
   observation fanout, so no observation may be claimed to stop on logout yet.

## Minimal test inventory for the eventual implementation

- `internal/auth/local_test.go`: current check leaves `LastSeen` unchanged;
  valid current membership; idle and absolute expiry; concurrent current check
  versus touch/expiry; watch/delete race; one-shot revocation on each of the
  four deletion routes (including capacity eviction through `Start`); repeated
  destroy/sweep is safe. Existing sliding `Resolve` tests stay intact.
- `internal/server` observation/SSE tests: cookie-session logout, expiry
  discovered by check/sweep, and capacity eviction each cancel a blocked owned
  trace read, wait for its completion, and prevent later observation fanout;
  request disconnect also drains; unrelated session/read remains live; bearer
  behavior stays distinct; a revoked perspective still fails closed. Assert
  observation side-channel behavior separately from legacy snapshot replay
  and real case-cursor `id:` semantics.
- SFWP cancellation prerequisite: actual blocked `run_get` cancellation,
  typed context-canceled error, no retry under canceled context, isolated
  connection retirement, pool slot release only after read exits, and no late
  callback after connection reuse, per the frozen Unit5A assignment. Server
  tests must join the HTTP handler/read owner rather than infer drain from a
  cancellation signal alone.

## Lock/lifetime constraints and gaps

Today the only auth lock is `SessionStore.mu`; session fields include mutable
`LastSeen` and `sessions` map. A notification sent under this lock should be a
nonblocking one-shot close, not callback execution or a wait. Cancellation,
read drain, and poller removal belong outside the auth lock. Root must settle
whether a shared poller is keyed by session or shared among authorized
subscribers: canceling one subscriber must not abort work still owned by
another, while no revoked subscriber can receive buffered frames. The approved
T09 proposal explicitly leaves shared read/poller authorization and revocation
semantics for integration; current sources establish neither. A non-sliding
check alone only detects revocation at its next check; a watch alone does not
enforce expiry at the exact deadline given the one-minute sweeper. No source
review here selects the required cadence or ownership model.
