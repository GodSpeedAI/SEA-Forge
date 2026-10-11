# Independent review: creator/list drain boundary proposal

**Verdict: REJECT the proposal as currently written.** Its successful Prepare path cannot complete the required bridge join because the local request context has no stated normal-path cancellation.

## Main defect: successful creator cannot join its bridge

Proposal §3 creates one bridge goroutine per preparing lease. It waits for
either `lease.leaseDone` or the derived local request context's `Done`, and the
creator must join that goroutine before `finishPrepareOperation(lease)`. On a
successful active Prepare, however, `leaseDone` must remain open until a later
detach, and the caller's request context may remain live for the entire active
lease. The proposal does not direct the creator to cancel the derived local
context after its last list/initial-readiness use.

Consequently the bridge remains blocked. The creator cannot finish, so
`creatorDone` never closes; later detach waits for that creator and cannot
complete. The new detach test's successful B empty-list path can hit the same
condition. The bridge is bounded in count but not lifetime, so the stated
“join before creator finish” does not itself solve the leak/deadlock.

**Smallest correction:** give the creator sole ownership of the derived
request-context cancel function. After all list and initial readiness work is
complete, explicitly cancel that local context on every return path—including
successful complete Prepare and successful unavailable-list/empty-lease
return—then join the bridge, then close `creatorDone` and balance the manager
WaitGroup. Ensure the local context is not the manager-owned shared-worker
context. The successful B empty-list test should remain and must complete
before B is detached; its completion proves the creator has joined the bridge.

## Additional required assertion and lifecycle precision

The Global Stop test requires a typed failure, zero DTO, and nil lease. Keep
that exact assertion so Stop-canceled list work cannot be confused with the
successful unavailable-list DTO/empty-lease outcome. The detach-A test only
says “Prepare failure”; strengthen it to assert the agreed typed
cancellation/stop error, a zero DTO, and nil lease. That distinguishes a
canceled creator from an ordinary list refusal.

After the list returns, reservation/attachment must recheck manager and exact
lease admission under the manager mutex. If Stop/Detach won while the callback
was in flight, no later poller may be attached outside the drain owner's
captured work. The test callbacks return cancellation errors after release,
but the production boundary still must not rely on a port always returning an
error merely because its context was canceled.

Preserve the original Unit 1 timeout rule for manager-wide Stop as well as
Detach: a bounded Stop wait may report `ErrDrainTimeout`, but may not close
drain completion, remove retained reverse associations, or release cohort/
poller capacity before the actual owned work and JOIN. A retry may resume the
same drain without another ref removal. The proposal states timeout retention
for Detach but does not state the Stop timeout result/retry behavior. Stop's
wait must honor its context without holding `m.mu`; the per-lease completion
channels can bound waits without inventing a serialized field or allowing
capacity reuse.

## Other clauses that are coherent

The exact lease parameter to `finishPrepareOperation`, one lease-owned
`creatorDone`, and an Add under the same mutex ordering as Stop's `stopping`
transition correctly let Detach wait for only A's creator while unrelated B
continues. Closing `leaseDone` after winning the drain transition and outside
the mutex avoids double close; Stop must skip signals already claimed by a
lease drain. Finishing the creator after list/bridge/worker-launch completion
but before its own drain wait prevents self-join. Stop signaling every lease
before waiting on the manager creator count allows held context-aware list
calls to return and preserves the immutable all-workers-launch-once rule when
Stop lands after reservation. The shared poller context remains independent
from each Prepare context. The two channel-controlled held-list tests use real
list callbacks and read-only cohort observation, preserve the nine existing
assertion bodies, and can prove exact-lease waiting versus manager-wide
waiting without scheduler hooks.

The proposal's new state remains private to the existing component:
`creatorDone` and `leaseDone` plus an exact-lease `finishPrepareOperation`
parameter. These do not add a public API, enum, image field, token, persistent
counter, or callback. The four fixture wrapper-shape updates are permitted and
the frozen manager fixture and six primitive files remain excluded. These
choices do not introduce the held C2/public boundary or authorize lifecycle
implementation.

## Evidence and scope

Reviewed the full original creator/drain assignment (SHA-256
`cacbd074f25446d155c57c1c1960a7839faad4158ee22168de501eed435d91f7`), its
complete proposal (SHA-256
`3a5e5d70a7adfd177e951dd0118f6f7f32ed2f906306695687fd8f7b7a293119`), the
original Unit 1 contract and preregistration, revision 6/addendum/corrections,
approved corrected read-start proposal/supplement/review, reverse-membership
clarification/review, and cited fixtures. The proposal materially revises the
old no-argument finish boundary and frozen new-fixture boundary as authorized,
but the missing local-context cancellation and incomplete timeout/typed-result
requirements leave its lifecycle incomplete.

This is a documentation-only rejection. No source/test edit, compiler,
formatter, scanner, Graft build, or Git operation was performed. No lifecycle
implementation or runtime claim is made.
