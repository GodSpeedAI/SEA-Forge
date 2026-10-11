# Independent source review: session lifecycle production change

Date: 2026-10-05. Verdict: **APPROVE source correctness for the bounded
SessionStore prerequisite**. This does not approve server/SSE integration or
claim pending reads are canceled or drained. No source was changed and no
tests/compiler/scanner/Git/status/debt commands were run in this source-review
phase.

## Identity and scope

Current `internal/auth/session.go` SHA-256 is
`d1c9daabfb758de94ea20ea0b7db7fbce1fad3a4ec43f6dee461d428b9972aa5`, matching
the builder record. Frozen `session_observation_test.go` remains
`8de408be7efc31d6e19d73870bee541d82f127272b5c2ab048dbee6ba3212e3f`. The
builder's exact diff is limited to `session.go`; the accepted fixture is
unchanged. The input proposal and approval constrain this unit to the store
primitive and explicitly reserve server read cancellation/drain for a later
coordinator.

## Source checks

- `Start` creates one private `revoked` channel in each session (`session.go:
  95-120`) while preserving CSPRNG calls, timestamps, TTLs and bounded oldest-
  `LastSeen` eviction. Session map insertion and eviction remain under the
  existing `mu`.
- `Current` rejects blank IDs with exact zero/false, then holds `mu` through
  map lookup, strict expiry validation, deadline calculation and detached
  state creation (`:147-169`). It returns zero/false for absent or expired
  entries; uses `Identity.Claim()` for actor/role only; sets deadline to the
  earlier of `LastSeen + idleTTL` and absolute `ExpiresAt`; and never assigns
  `LastSeen` or returns a `Session` pointer. `Session.Idle` remains strict
  `now.Sub(LastSeen) > idle`, while absolute expiry remains strict
  `now.After(ExpiresAt)` (`:36-40,157-160`); exact-boundary validity matches
  the approved proposal.
- `removeLocked` is the one idempotent deletion helper (`:216-226`). It returns
  false for absent IDs, closes the signal once for a present session, then
  deletes its map entry, all while callers hold the existing `mu`. The five
  paths route through it: expired `Resolve` (`:124-140`), expired `Current`
  (`:157-160`), logout `Destroy` (`:172-176`), `Sweep` (`:188-199`), and
  capacity eviction from `Start` via `evictOldestLocked` (`:205-213`). No
  callback, receive, network action, cancellation, or join was added under the
  lock. No second registry exists; the one private channel field per stored
  session follows the bounded session map.
- Caller/source search found only the expected production `Start`, `Resolve`,
  `Destroy`, and `Sweep` sites. No external session-map insertion or additional
  `LastSeen` writer/reader exists. Private channel storage does not change
  `Session` JSON or identity/CSRF/session interfaces; the public methods keep
  their existing return shapes and semantics.

The frozen fixtures cover detached/no-slide claims, exact and just-after idle
and absolute boundaries, legitimate Resolve touch/fixed absolute deadline,
signal stability and all removal paths, idempotence, unrelated-session
preservation, capacity and lookup/delete races. Source review finds the
implementation consistent with those assertions. `removeLocked` relies on the
invariant that stored sessions come from `Start` and therefore have a nonnil
channel; repository search found no other insertion path.

## Limits carried forward

This approval is only for the bounded auth-store prerequisite. A copied
revocation channel is a notification, not proof of worker shutdown. The later
server owner still must use the copied deadline without sliding activity,
recheck at strict equality without a zero-duration spin, and cancel/join owned
reads before releasing capacity. It must separately preserve dev-bearer
behavior, effective kernel-perspective checks, shared-poller ownership and no
post-revocation stale fanout. No such behavior is implemented or approved here.

No material source-scope deviation was found. Proceed only with the separately
authorized focused auth race and full auth race gates; runtime results are
recorded independently.
