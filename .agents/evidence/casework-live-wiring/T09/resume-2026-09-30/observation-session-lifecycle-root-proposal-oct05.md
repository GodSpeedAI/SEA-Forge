# Unit5C prerequisite: non-sliding session lifecycle proposal

Preparation only; no implementation or compiler token is released. This internal
prerequisite implements the already approved observation lifetime boundary,
without changing identity mappings, expiry policy, CSRF or public HTTP schemas.

## Root decision

Keep one canonical SessionStore owner. Add an internal application method
`Current(id)` returning a detached state value and presence flag under store.mu.
The state contains only copied actor/role claim, earliest current idle/absolute
deadline, and a receive-only revocation channel. No session ID, CSRF token,
mutable Session pointer, raw identity profile or wire JSON DTO is returned.
Current does not touch LastSeen. It uses existing strict expiry semantics
(`now.After(absolute)` or `now.Sub(LastSeen) > idle`); equality remains valid.
Absent/expired state is absent, never an invented bearer/session identity.

Create one stable private channel per Start session. Every removal path must
close that channel exactly once under store.mu before removing the map entry:
Destroy/logout, Resolve expiry, Current expiry, Sweep expiry and capacity eviction.
Use one deletion helper, not repeated independent revocation logic. Repeated
Destroy/expiry checks are harmless. Start and Resolve retain their existing
interfaces and sliding activity behavior. Other sessions remain untouched.

Current's atomic map lookup and channel snapshot removes the watch-registration
race without a second registry lock: lookup before deletion gets a channel that
deletion closes; lookup after deletion gets absence. No callbacks, waits, network
reads or drain joins occur under store.mu. Channel closure is notification only;
the later server coordinator owns cancellation and waiting for pending reads.
No claim is made that notification alone drains those reads or HTTP writes.

The later watcher must use the copied deadline and recheck Current at expiry
without sliding idle time; minute Sweep alone is insufficient. At exact boundary
it must honor the existing strict comparison, avoiding a zero-duration busy loop.
Activity from ordinary authenticated requests may extend idle time, never the
absolute deadline; a timer recheck obtains the new earlier-of-two deadline.
Session callbacks/claims never ride the observation wire or logs.

## Bounded implementation/test inventory after independent proposal acceptance

Own existing `internal/auth/session.go` and one new focused auth test file.
Preserve existing auth tests and nearby lifecycle patterns. No server, poller,
SSE, session login identity or dependency change in this unit. First add fixtures
and minimal declarations for a compiling assertion RED, freeze them, and let
an independent critic prove RED with the sole compiler slot. A different bounded
production builder then completes the existing store logic.

Required tests: Current does not slide expiry; detached actor/role values cannot
mutate stored state; earliest deadline is correct before/after legitimate Resolve;
exact deadline preserves existing validity while after it expires; every deletion
path closes the same observed channel, repeated deletion never panics; unrelated
sessions remain live; lookup/deletion races return either absent or a closed
signal without missed revocation; capacity and store cardinality remain bounded.
Use synchronized clocks for concurrent tests, no unsynchronized testClock writes,
no credentials or session-token values in evidence. Existing auth suite and
race-enabled focused/full module gates remain required after implementation.

## Independent source review

Read the original T09 lifecycle requirements and root scoped authorization,
current auth/store callers, source recon edit inventory, then this proposal.
Verify every removal and LastSeen reader, absence of identity/expiry changes,
deadline/lock/race semantics and all scope omissions. Cite direct source evidence
and explain every material difference from the original observation instructions.
Insufficient evidence means reject. Proposal approval alone does not settle the
server cancellation/drain boundary or release full unit5C implementation.
