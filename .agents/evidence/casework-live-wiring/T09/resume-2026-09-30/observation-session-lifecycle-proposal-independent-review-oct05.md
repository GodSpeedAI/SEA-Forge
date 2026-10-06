# Independent review: session lifecycle prerequisite proposal

Date: 2026-10-05. Verdict: **APPROVE as the auth-store prerequisite plan only**.
This does not approve an SSE/poller implementation, claim read draining, or
release the broader Unit5C work. No source or test files were changed and no
tests/compiler/scans/Git operations were run.

## Evidence and review

The proposal at
`observation-session-lifecycle-root-proposal-oct05.md` preserves the existing
identity and expiry model. Current `SessionStore.Resolve` slides `LastSeen`
and returns the shared session pointer (`internal/auth/session.go:114-132`);
`Session.Idle` uses strict `now.Sub(LastSeen) > idle` (`:32-34`), and absolute
expiry uses strict `now.After(ExpiresAt)` (`:124-127,155-156`). A separate
`Current` returning copied actor/role, earliest deadline and a receive-only
channel under the existing mutex avoids both the sliding lookup and an
unsynchronized pointer read. `auth.Identity.Claim` contains only `ActorID` and
`Role` (`internal/auth/auth.go:47-57`), matching the proposed detached claim.
No session ID, CSRF token, profile or wire DTO is added to that state.

The proposal covers all source deletion paths in the current checkout:
expiry in `Resolve` (`session.go:124-127`), logout `Destroy` (`:134-138`,
called by `internal/server/session.go:376-385`), `Sweep` (`:149-161`), and
capacity eviction via `Start` → `evictOldestLocked` (`:102-105,164-175`). A
single deletion helper closing the stable per-session channel under `mu`
before map removal makes the lookup/delete race atomic without another lock.
No waits or callbacks under that mutex is consistent with the store's only
current synchronization boundary. Repository call-site search shows production
`Resolve` at `server/session.go:129`, `Destroy` at `:382`, `Start` at `:325,369`,
and `Sweep` at `cmd/godspeed-casework/main.go:313`; no overlooked deletion or
`LastSeen` reader outside `auth/session.go` appeared.

Scope is appropriately narrow. Current `handleEvents` verifies perspective
once and consumes only revision subscriptions (`internal/server/server.go:
293-376`); the run-trace SFWP adapter is still an unavailable stub
(`internal/adapters/sfwp/run_trace.go:10-15`). The proposal explicitly makes
channel closure notification only and reserves worker cancellation/drain for a
later coordinator. This respects the approved T09 requirement for logout,
expiry, disconnect, and pending-read cancellation without claiming the auth
store alone can deliver it. It leaves session/bearer semantics, CSRF, identity
mapping, HTTP schemas, revision replay, and public wire behavior unchanged.

The implementation/test scope covers detached claims, non-sliding checks,
deadlines, every deletion route, idempotent deletion, unrelated sessions,
registration/removal races, and bounded capacity. It correctly prohibits
unsynchronized manual-clock mutation in concurrent tests and excludes
credentials from evidence.

## Conditions to carry into the later server plan

1. The strict comparisons make a deadline instant itself valid. If a server
   timer fires exactly at that instant, `Current` must return the same active
   deadline. The later watcher plan must specify a positive re-arm strategy
   after that equality case so it neither expires early nor spins at zero
   duration. The proposal identifies this edge but does not choose scheduling
   details; that is outside this auth-store prerequisite.
2. Since idle and absolute expiry are separate checks, focused tests should
   pin exact-boundary and just-after behavior for each deadline independently,
   in addition to testing the returned earlier deadline before/after a
   legitimate `Resolve` touch.
3. Approval is limited to the session-store primitive. The independent
   cancellation assignment still requires actual blocked-read cancel/drain
   proof, and T09 still requires per-subscriber effective-perspective checks,
   shared-poller ownership, and no buffered stale fanout after revocation.

No material scope or identity deviation was found. Root retains the final
architecture decision and separate approval of the server lifetime boundary.
