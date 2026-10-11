# Creator/list final correction supplement — 2026-10-07

## Provenance and review status

This new immutable DOCONLY supplement follows the complete corrective
assignment archived in
[`run-observation-creator-drain-corrective-supplement-assignment-oct07.md`](run-observation-creator-drain-corrective-supplement-assignment-oct07.md).
It preserves, without rewriting, the original assignment (`cacbd074…`),
proposal (`3a5e5d70…`), correction assignment (`fca608e8…`), correction
supplement (`37d943ad…`), context erratum (`1efaaaea…`), and independent
rejection (`49bba9ae…`). Root's new decisions below supersede any inconsistent
wording in those records. This document releases no source or fixture work.
An independent semantic critic must approve the full chain before TDD source
release.

## Final admission correction

Change the private `reservePollerBatch` boundary to require a third argument:
`prepareDone <-chan struct{}`. Production obtains the channel from the
creator's local derived Prepare context outside `m.mu`, then passes it. The
four manual reservation-boundary fixtures pass their exact
`lease.leaseDone` channel. Both use the same helper; there is no nil/test mode.

Before any poller reservation or lease attachment, under `m.mu`, the helper
performs a **nonblocking receive-only** select on `prepareDone` and checks all
existing admission predicates: manager is not stopping, exact lease is not
draining, and `m.cohorts[lease]` still contains that exact lease. A closed
`prepareDone` returns the typed cancellation failure; a winning Stop/Detach
returns its typed failure. There is no wait, callback, context method, or other
I/O under the mutex. This check is the admission linearization point; a caller
may cancel immediately afterward, so it is not a promise that wall-clock
cancellation cannot occur after acceptance.

Immediately after reservation, before readiness waits or DTO construction,
the creator performs a nonblocking local-context cancellation check. If
cancellation won after the helper's linearization point, it claims owned
rollback and marks its now-ineligible batch for stop, but still launches every
entry in that immutable reserved batch exactly once. Stop-marked workers
resolve the existing code-4/phase-2-or-3 path without starting a port call.
Rollback cancels only watcherless workers; a surviving eligible lease keeps
shared work. The creator finishes all list/context/bridge/launch work, cancels
its local context, joins its bridge, closes its exact `creatorDone` and
balances the creator registration, then waits for its own drain. A final
local-context check before successful DTO return prevents publishing success
after cancellation. No new counter, token, callback, field, or serialized
control is introduced.

## Added held-list cancellation case

Keep all nine existing cases/assertions and both approved held-list cases.
Add this third test with a real `RunsListForCase` callback and bounded channel
waits:

1. Start Prepare with a callback that signals entry and waits on explicit
   release. Cancel the caller context while the callback is held; the callback
   observes cancellation but remains held, then is released to return `nil`
   error and a valid readable list candidate.
2. Before release, assert cancellation was observed, the actual callback is
   still held, creator completion has not occurred, and its admitted cohort
   remains counted. This is caller-context cancellation, not Stop or
   `leaseDone` cancellation.
3. After release, require a typed cancellation error, zero DTO, nil returned
   lease, zero RunTracePort calls, and no poller reservation/attachment from
   the returned candidate. Require the creator and bridge to complete before
   that cohort capacity is reusable. Do not treat an empty or unavailable-list
   successful DTO as satisfying the case.

This proves the missing caller-cancellation race: a list implementation that
returns a successful candidate despite its canceled context cannot authorize
post-cancel poller admission. The two earlier cases continue to prove Stop and
one-lease Detach signal their independent local bridges and preserve another
lease's context/work. In the successful B-empty path, observe its exact
`creatorDone` closed when Prepare returns; source review verifies that local
cancel and bridge JOIN precede that close.

The four manual `reservePollerBatch` call sites change only by passing the
genuine lease's `leaseDone`; the four function-qualified finish closures
continue to pass their exact lease. Preserve all existing identities and
assertions. The original manager fixture and six published primitive files
remain frozen.

## Stable completion ownership and retry

Reject the previous proposal's channel-nil sentinel. `creatorDone`,
`leaseDone`, `drainDone`, and `stopDone` remain stable, nonnil channel
identities once initialized; each is closed once, outside `m.mu`, only at its
specified transition or actual completion. A timeout never fabricates a
close.

- The first lease transition `draining: false → true` under `m.mu` owns the
  exact local entry capture, eligible `lease.pollers` removal, and the single
  close/cancel signal for `leaseDone`. After unlocking it closes/signals and
  cancels watcherless workers, then starts one bounded background drain job
  for that lease. That job waits for the exact `creatorDone` and required
  actual worker return/JOIN, removes exact reverse refs and capacity under
  identity checks, and closes stable `drainDone` outside the mutex only after
  actual completion. A later Detach waits on the same `drainDone`; it does not
  create another owner or repeat cleanup.
- The first manager transition `stopping: false → true` under `m.mu` owns
  `stopDone` initialization and the global stop job. After unlocking it issues
  **all** owned lease signals and poller cancels before starting/waiting on the
  global creator and worker joins. The stop job first waits for all creator
  launch registrations, then waits for the exact lease drain jobs and actual
  workers, and closes stable `stopDone` once outside the mutex after joins.
  Later Stop calls wait on that same channel.
- A caller's wait-context timeout returns an error to that caller but does not
  cancel the background owner, close a completion channel, or release
  capacity. The owner continues independently; a retry waits on the same
  stable completion. A request creator still finishes its exact creator
  registration before waiting for a drain that may be waiting for it.

These are bounded real lifecycle control jobs: at most one per admitted
cohort (cohort cap 16) plus one global Stop job. They do not create another
poller/read worker and require no persistent owner flag, counter, token,
callback, or extra manager/lease field. `draining` and `stopping` claim the
single owner under the existing mutex; captured locals preserve exact entry
identity. Existing stable completion channels make concurrent callers
waiters, including after an initiating caller times out.

## Material amendments and limits

Compared with the rejected correction, this adds the required Prepare-context
admission channel to the private reservation helper, updates its four direct
fixture call shapes, adds a third real held-list cancellation case, and
replaces the rejected channel-nil completion sentinel with one bounded
background owner per lease plus one Stop owner using existing booleans and
stable channels. These three scope amendments are explicitly authorized as
root design choices for review; they do not authorize source edits.

The approved reverse-membership rule remains: an eligible ref requires an
exact manager entry and `!lease.draining && lease.pollers[key] == entry`;
`entry.refs[lease]` may remain only as noneligible drain ownership until actual
JOIN. Reservation and full combined-image checks still precede pointer
publication; code-4 stopped initialization makes no actual port call or
successful Prepare, while a real initial unavailable call remains successful
A for its owner. All-read accounting remains actual-call-based. Existing
selection, no-backfill, parent guard, 1 MiB complete-image cap, fixed-width
phase/result markers, recurring interval, and original 9+2 test obligations
are unchanged.

No source facts are upgraded to implementation claims. No Go/test edit,
compiler, formatter, scanner, explicit Graft build, Git action, or runtime
gate was performed for this supplement. Root retains the architecture
decision; independent review is the next required step.
