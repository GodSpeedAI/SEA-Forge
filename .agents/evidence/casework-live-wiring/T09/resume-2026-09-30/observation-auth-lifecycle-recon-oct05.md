# T09 observation auth/session lifecycle — source reconnaissance

Date: 2026-10-05

## Scope

Source-only reconnaissance for root's T09 observation lifecycle planning. I read
the current T09 contract proposal, the T09 run-observation/auth preparation
records, the T07 session-bound retained-read specification and coverage notes,
and the current Go source cited below. No code, test, status, debt, or Git files
were changed; no compiler or test command was run. This records facts and
candidate insertion points, not an architecture decision or implementation
permission.

## Governing contract and prior preparation

The canonical T09 proposal is
`.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md`.
It proposes authenticated `execution_observation` frames as a side channel
without case-cursor `id:` values; frames may not mutate retained snapshots.
Its source-preparation records add that each subscription/read must be
authorized for the effective actor, that the current gateway must not assume
actorless kernel `run.list`/`run.get` reads prove disclosure invariance, and
that revocation/logout/expiry must stop stale observation fanout and drain the
subscriber's owned work. The prep notes leave session-sharing and per-subscriber
authorization as root decisions.

T07's `session-read-fix/original-spec.md` requires streamed and retained
snapshots to reflect the authenticated session actor/role, kernel perspective
verification before SSE starts, and fail-closed behavior for invalid/revoked
delegation. Its test notes anchor this behavior in
`internal/server/session_read_test.go` and preserve legacy same-perspective
snapshots. These requirements make the existing effective actor and role the
relevant projection scope; a case-only/global observation cache must not widen
that scope without a separate proof.

One preparation fact is stale relative to current source: `observation-runtime-
authority-source-facts.md` says no production `Sweep` caller was found. Current
`cmd/godspeed-casework/main.go:253-256,301-316` starts `sweepSessions` from
`serveLive`; it calls `SessionStore.Sweep` every minute until its context ends.

## Current store lifecycle

Source anchors: `internal/auth/session.go:22-47,82-108,111-138,147-175`.

- A `Session` contains fixed `ID`, `Identity`, `CSRFToken`, and `CreatedAt`,
  plus mutable `LastSeen` and fixed absolute `ExpiresAt`. The store holds
  `map[string]*Session` behind one `sync.Mutex`.
- `Start` creates the pointer, locks the store, and when at capacity evicts the
  least-recently-seen session through `evictOldestLocked` before insertion.
- `Resolve(id)` locks, rejects an absent ID, removes idle- or absolute-expired
  sessions, otherwise sets `LastSeen = now`, then returns the same shared
  `*Session` after unlocking. It is explicitly a sliding touch. Using it as a
  periodic liveness check would keep an otherwise-idle session alive.
- `Destroy(id)` only deletes under the lock. `Sweep()` deletes all expired
  entries under the lock and returns a count. Expiry in `Resolve`, `Sweep`,
  and capacity eviction also only deletes. None of these paths emits a
  revocation signal or exposes a subscriber channel.
- Existing deterministic tests cover sliding idle versus fixed absolute
  expiry and max-size eviction/sweep at `internal/auth/local_test.go:116-185`.

`SessionStore.Resolve` is the correct synchronization boundary for checking
membership and expiry, but its returned pointer is not a read-only snapshot.
Do not use it for stream revalidation: it changes `LastSeen`. Do not add reads
of `LastSeen` through `requestIdentity.Session` from a long-lived handler; the
store's other requests and sweeper may update/read that same field under the
store mutex while the handler would be outside it.

## Current request identity and SSE behavior

Source anchors: `internal/server/session.go:45-75,85-141,147-164,376-400`;
`internal/server/server.go:124-146,293-400`; `internal/projection/live.go:323-340`.

- `GET /api/events` is wrapped in `requireSession`, whose name is narrower than
  its behavior: `tryResolveIdentity` first resolves a session cookie, then
  accepts the configured static bearer identity. `requestIdentity.Source`
  distinguishes `session` and `bearer`; bearer identities have `Session ==
  nil`. The static bearer surface is documented as dev-only.
- Session resolution currently attaches a pointer returned by `Resolve`, while
  copying `sess.Identity` into `requestIdentity.Identity`. Existing event code
  uses the copied identity to build `ports.ActorClaim`; it does not need to
  consult mutable `LastSeen`.
- `handleEvents` rejects actor/role query overrides, calls
  `verifySessionPerspective` once at stream open, subscribes to case revisions,
  then waits for request cancellation, heartbeat, or revision. Every revision
  is rendered for the actor captured at open. A legacy revision without facts
  is sent only for its exact stored actor/role; otherwise the stream closes.
  There is no subsequent session-store lookup or perspective verification.
- `VerifyPerspective` resolves the gateway actor on behalf of the requested
  actor/role and refuses a denial or actor mismatch. It receives the request
  context. SSE's HTTP context is canceled on client disconnect/shutdown, but
  logout does not cancel it.
- `POST /api/auth/logout` is wrapped in `requireSession` and `requireCSRF`;
  `handleLogout` calls `Destroy(ri.Session.ID)`, expires both cookies, and
  returns. Since `Destroy` has no connection to existing SSE contexts, an
  already-open stream continues until its request context ends or its revision
  subscription closes.
- The existing SSE URL consumes `last`/`Last-Event-ID`; there is no case-ID
  selection in `handleEvents`. The current T09 proposal needs an explicit
  requested `case_id` seam for case-scoped observation hydration. Do not infer
  it from the latest global relay revision. Keep normal snapshot replay and
  case-cursor ordering unchanged; `writeSSE` omits `id:` when given an empty ID
  (`internal/server/http.go:184-198`).

`requestIdentity.Session` is also used by existing CSRF and limiter paths to
read immutable `CSRFToken`/`ID` values. That does not justify adding stream
reads of mutable lifecycle fields through the pointer. A fresh liveness path
should use a detached scalar identity/session snapshot returned while holding
the store lock, or a boolean membership/expiry check; it should not return a
new alias to the shared `*Session`.

## Candidate mechanisms and risks for root

These are options to evaluate, not a selected design:

1. **Non-sliding current check.** Add a `SessionStore` method near `Resolve`
   that checks presence and both TTLs under `mu` without changing `LastSeen`,
   returning either `bool` or a copied immutable identity/session view. It can
   remove an expired entry consistently with `Resolve`, but must not return
   the stored pointer. The stream can check before each protected run read and
   immediately before any observation fanout. Capture an internal session key
   at authentication time without logging or serializing it. This gives
   precise checks at those boundaries but detects logout between boundaries
   only at the next check.
2. **Revocation watch.** If root needs prompt logout/eviction notification,
   register a per-session watch atomically with the active-session check under
   the store lock. `Destroy`, expiry removal (`Resolve`/`Sweep`), and
   `evictOldestLocked` all need to signal it. Avoid a blocking callback while
   holding `mu`; define one-shot cleanup and make watch registration lose no
   delete-between-check-and-subscribe race. A watch alone does not enforce TTL
   at the exact expiry instant: the current background sweeper has a one-minute
   interval, so pair it with an expiry timer or non-sliding checks before reads
   and delivery if the contract needs tighter expiry enforcement.
3. **Per-subscriber ownership.** Cancel and drain a subscriber's observation
   read/worker on logout, expiry, request disconnect, or failed perspective
   verification. If pollers/buffers are shared, loss of one subscriber must
   not cancel a poller another authorized subscriber still uses; fanout must
   revalidate each subscriber's effective actor/session scope. Existing
   `Store.Subscribe` cancellation only removes a case-revision subscriber; it
   is not a session-revocation mechanism.

Scope boundaries to preserve while evaluating those options:

- Keep dev-bearer authentication distinct from cookie sessions. A nil
  `requestIdentity.Session` can mean valid dev bearer, not anonymous. Do not
  require a cookie watcher for the existing dev-bearer SSE path or change its
  configured identity semantics without a separate decision.
- Keep legacy `snapshot`, `hello`, `resync_required`, replay, and cursor rules
  on their current path. T09 observations are a separate no-`id:` side channel;
  session lifecycle work should not rewrite retained facts or attach
  observations to historical snapshots.
- Preserve the T07 effective actor/role projection (the delegated D-2
  identity boundary). A poller/cache keyed only by run or case is unsafe unless
  effective-session disclosure invariance is separately proved. The
  actor/role identity captured at open is not evidence that the session or
  kernel delegation remains current later.
- Current prep requires per-read and per-fanout authorization because
  `run.list`/`run.get` inspect operations do not themselves enforce session
  disclosure scope. A session-current check and a fresh kernel perspective
  check are distinct: neither substitutes for the other.

Minimal source seams are `internal/auth/session.go` adjacent to
`Resolve`/`Destroy`/`Sweep`/`evictOldestLocked`, `internal/server/session.go`
where request identity is resolved, and `internal/server/server.go` at the
observation-specific read/fanout branch under `handleEvents`. Root retains the
decision on polling cadence, identity keying, bearer scope, and whether legacy
SSE itself gains revocation behavior.
