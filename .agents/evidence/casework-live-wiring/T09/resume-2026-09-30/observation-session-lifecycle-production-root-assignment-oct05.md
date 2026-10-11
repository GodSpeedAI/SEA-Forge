# Session lifecycle production assignment

Prepared by root; RELEASE ONLY after root accepts the independent actual focused
assertion RED for fixture8de408be. A source readiness approval alone is insufficient.

## Binding instructions and frozen inputs

Read root/scoped AGENTS, the lifecycle root proposal and its independent proposal
approval, original Phase1 assignment, all fixture rejections/repairs and final
independent source/actual RED evidence. Preserve the approved scope and existing
identity, CSRF, expiry and authentication behavior. Current production file is
`internal/auth/session.go`, SHA256
`f8f0df88d283ef0c28057f08fcdd6b23c7111c89abd68e23b52f754eb9d17b48`.
Frozen fixture `session_observation_test.go` is SHA256
`8de408be7efc31d6e19d73870bee541d82f127272b5c2ab048dbee6ba3212e3f`.

## Bounded builder specification

Own ONLY `apps/godspeed-casework-go/internal/auth/session.go`. Read the full file,
nearby auth patterns/tests and all removal callers before editing. Implement the
already declared `Current(id)` with one atomic lookup/deadline/claim/signal snapshot
under `store.mu`. Absent, blank or expired IDs return exact zero state and false.
Use the same strict expiry tests as Resolve/Sweep: equality remains valid. Copy
actor/role through the existing Identity.Claim mapping; return no Session pointer,
bearer ID, CSRF or profile. Set ValidUntil to min(LastSeen+idleTTL, ExpiresAt),
without changing LastSeen or any other activity/absolute-expiry state.

Start creates one stable private revocation channel per stored session. Centralize
removal under the existing mutex: close that channel once before deleting its
entry. Route Destroy, Resolve expiry, Current expiry, Sweep and capacity eviction
through that helper. Repeated removal is harmless; unrelated sessions are unchanged;
channel storage cannot outgrow the bounded session store. Keep exported Session,
Start and Resolve behavior compatible; private storage is permitted. Do not create
a second independent lifecycle owner or watcher registry. No callback, channel
receive, network read, cancellation/drain join or other wait under store.mu.

Preserve CSPRNG behavior, idle/absolute defaults, current capacity/eviction policy,
existing interfaces and every existing auth test. No server/SSE/poller/UI/client,
public schema, identity model, dependency or unrelated edit. Do not change fixtures.
Notification closes do not claim cancellation or draining of pending reads.

## Evidence and independent verification

Builder runs no compiler/scanner/Git/status/debt commands. Native apply_patch only.
Write a NEW immutable builder record with original instructions, complete exact
actual diff, full source/fixture hashes, deviations and unverified runtime status.
Root gives the independent critic this assignment plus the full result. Critic must
verify all five removal routes and locked atomicity, no activity sliding, exact
boundary behavior, detached claim and absence semantics. Run focused lifecycle
race tests, full auth race tests, then relevant whole Go gates under individually
granted sole compiler ownership with RAM/swap preflight and actual joined exit.
Socket-bearing full auth tests need permitted loopback access; a sandbox socket
failure is not a behavior failure and never justifies weakening a test. Preserve
failed attempts and exact command/raw/exit captures. Insufficient evidence rejects.

This prerequisite cannot settle Unit5C, authenticated stream revocation/draining,
the trace port or full T09. Root accepts only the verified bounded auth change.
