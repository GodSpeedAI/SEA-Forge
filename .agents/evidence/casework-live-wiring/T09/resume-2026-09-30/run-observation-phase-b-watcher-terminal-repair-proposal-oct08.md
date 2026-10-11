# Phase B watcher and terminal correction proposal

Date: 2026-10-08  
Status: DOCONLY proposal for independent review. No source change or runtime
claim is authorized here.

## Provenance and governing inputs

This proposal follows the archived assignment
[`run-observation-phase-b-watcher-terminal-repair-assignment-oct08.md`](run-observation-phase-b-watcher-terminal-repair-assignment-oct08.md),
SHA-256 `7ffdd48ccbfd69d4ff0e8bf5d7812405b7385651c5b5396e8b4c30b4e8deb852`.
The original Phase B source assignment is
[`run-observation-phase-b-algorithm-source-assignment-oct08.md`](run-observation-phase-b-algorithm-source-assignment-oct08.md),
SHA-256 `e5fc9e0aa9a8549aa75bcf78040b7012e79e8a882a176965dc80ce6eace8ef42`.
Its implementation was rejected by
[`run-observation-phase-b-algorithm-independent-review-oct08.md`](run-observation-phase-b-algorithm-independent-review-oct08.md),
SHA-256 `ecf2e06f34a57257cc99480dc17a41b40cdaff0a34248ceeb7c940c91473b644`.
The review is the source for the concrete defects below; it is not a runtime
verdict. The prior algorithm result is
[`run-observation-phase-b-algorithm-result-oct08.md`](run-observation-phase-b-algorithm-result-oct08.md),
SHA-256 `a964dc9c036050fa97f0b49c2ae5628d020043dbbc9b3fe47416df0f07326e48`.

Revision 6 requires lease-local watcher identity and `asOfCursor` at clauses
246–258; independent auth and present-context checks before list, selected
trace reads, publication and disclosure at 329–348; exact cursor equality and
its non-atomic as-of limit at 343–348; and a fitting accepted terminal version
to be disclosed before polling stops and actual worker JOIN at 467–468. Its
unretainable terminal rule is at 387–390. The current helper
`checkRunObservationPresentContext` (`run_observation_present_context.go:25–76`)
returns case, cursor and validated parents, and performs Store/Relay checks;
`runObservationCallerFromRequest` (`run_observation_manager.go:301–316`)
reduces request identity to source, session ID and actor claim without keeping
a bearer token or session pointer. The current lease (`:81–88`) stores neither
watcher value. Current `prepare` authenticates and guards before admission
(`:318–349`), checks cursor after list (`:405–407`), then checks cancellation
and lease state before return (`:549–553`). The worker does not reauthorize
watchers: its read path is `run_observation_poller_worker.go:50–89`; accepted
terminal candidates set `continuePolling` at `:160–166`; `terminalStop` is
computed before outer-image fallback at `:205–228` and gates drain at `:248–270`.

The rejected grant's claim that current state suffices without lease identity
is superseded only on the private metadata and required-input point below.
Root has clarified that revision 6's caller identity and cursor are required.
The original manager fixture, six primitive files, and current Phase B source
remain frozen during this proposal. No implementation or test release follows
from this document; a separate reviewed TDD assignment is required.

## Minimal metadata and read/publication protocol

Permit exactly two additional private immutable fields on `runObservationLease`,
using existing types: `caller runObservationCaller` and `asOfCursor string`.
Make both required arguments of `beginPrepareOperation`. `prepare` must resolve
the caller with `runObservationCallerFromRequest`, perform the existing auth
and guard checks outside `m.mu`, then pass that caller and the guard's exact
cursor into atomic cohort admission. Admission stores them on that lease while
reserving its cohort slot. It must not accept nil/default/variadic identity or
cursor. No token, session pointer, callback, identity token, extra counter,
enum, or serialized field is added. The four manual begin setups in the
failure fixture must derive a real caller from their existing request identity
and obtain the cursor through `manager.guard`; retain their current test
behavior and assertions.

Before each selected actual trace call, snapshot under `m.mu` the exact current
entry/version, its forward membership, and candidate watcher leases. A watcher
is eligible only while it is not draining and the lease-to-entry membership
and entry-to-lease membership are both exact. Outside the lock, independently
reauthorize every relevant watcher and run the existing present-context guard;
require its case/cursor to match that lease's stored `asOfCursor` and validate
the selected parent against the returned parent set. Drain only an invalid
watcher through the existing drain owner, so an authorized shared watcher
remains eligible. Reacquire `m.mu` and revalidate exact entry identity,
captured prior pointer, and eligible membership before claiming the read.
After the last lock check, check the worker context outside the lock
immediately before invoking `ReadRunTrace`. A canceled/invalid caller must not
start a read. A later cancellation can still race an already-linearized call;
the count records only calls actually invoked.

Before publishing a returned candidate, repeat per-watcher authorization,
guard/cursor and parent validation outside the lock; then reacquire the lock
and check exact current entry, prior pointer and eligible membership before
the atomic image/pointer publication. Revalidate auth and present context for
each lease before initial DTO handoff, including unavailable-list and empty
list branches. Recheck the captured cursor immediately before handoff. On a
failed check, return typed unavailable with zero DTO and nil lease for a
failed Prepare, and let the exact owner roll back/join its work. For later
disclosure, preserve the existing zero-delta/no-watermark-advance error rule.
No auth, guard, port, cancel, channel close/send, wait or join runs under
`m.mu`. These Store/Relay checks remain non-atomic as-of checks; they do not
promise continuous truth or public readiness/frontier behavior.

## Terminal disposition and retention

For a candidate whose complete combined outer image fits, atomically publish
the final accepted retained version. If its validated state is terminal, keep
that current version available to initial Prepare (owner actual read count 1,
shared initializer count 0), finish any DTO handoff, then stop the worker.
`workerDone` closes only after the real worker returns. Keep entry/lease
capacity counted until existing detach/Stop ownership observes actual JOIN;
do not drain before the validated initial DTO is handed off, substitute a
failed terminal DTO, or infer settlement.

For terminal state that cannot be retained under the fixed combined-image
limit, base stop/drain on the final post-fallback phase/state, not an earlier
terminal snapshot. Publish only the safe fixed-width unavailable marker
representable within the existing format, bar subsequent read starts, and
signal/cancel the currently eligible leases outside `m.mu`. The worker exits
only after the current real read returns; capacity remains held until its
actual JOIN and owned cleanup. No candidate terminal data is disclosed and no
settlement is inferred. Nonterminal over-budget candidates retain the previous
immutable pointer and ledger, publish only the approved unavailable marker,
and may recover only through a complete later candidate that fits the full
ledger. Do not add state codes, counters or marker variants. These behaviors
use existing phase/result/current state and existing drain ownership.

The root read-count rule remains binding: successful Prepare may derive its
count from the creator-owned new-initializer batch only if every such worker
actually invoked the port. Shared/cache entries and oversized nil-image
refusals count zero; an actual initial unavailable read is successful A and
counts one for its owner, zero for waiters. Stop/cancel before any actual call
must fail Prepare with typed error, zero DTO and nil lease. Reservation is not
a read attempt.

## Bounded test-first follow-up

After independent approval, use one new focused authority/terminal test file
and make only the four manual-begin setup changes in
`run_observation_manager_failure_test.go`. Keep the original twelve fixture
assertions and existing manager fixture unchanged. Each new case must enter
through real `prepare`/worker boundaries and fail on current source because
the expected behavior is absent, not because it got a generic zero result:

1. A fitting terminal actual read is returned in the initial DTO; prove no
   later read starts and the real worker completion follows publication.
2. A terminal candidate whose complete outer image exceeds the cap is not
   disclosed, cannot start another read, and does not free capacity before
   actual worker completion.
3. Change current session authorization after list entry and before candidate
   read; Prepare fails closed before any trace read.
4. Revoke watcher authorization while a genuine trace port call is held;
   release it and prove no publication or successful initial DTO.
5. Change the guarded cursor while a genuine read is held; release it and
   prove no stale-cursor publication or successful DTO.
6. With two authorized leases sharing a real held initializer, invalidate
   one watcher and prove its rollback does not cancel or remove the authorized
   survivor's eligible work.

Use mutable session/guard fakes only as dependencies already accepted by the
manager; synchronize by actual list/read entry, callback-observed context
cancellation, explicit release channels, and existing bounded waits. Do not
sleep, write private state to manufacture a transition, add production hooks,
or use cancellation spies. The terminal tests must observe actual worker
completion; the one-second recurring interval is bounded by channels and a
test timeout, not a timing assumption. Tests and source require separate
review and focused RED before a new algorithm assignment.

## Boundaries and open review points

This proposal changes only the rejected private-state assumption: two
immutable lease values, their required begin arguments, four direct fixture
setup/call changes, and tests for the six behaviors above. It does not change
public APIs, DTO/schema, authorization policy, identity grammar, ports,
dependencies, retained-image cap, or lifecycle control codes. It does not
authorize source changes, GREEN, runtime claims, public V4/SSE, or T09
settlement. Independent review should confirm that the two values can be
captured and revalidated at all listed boundaries without an additional
persisted field, and that each proposed test reaches the intended production
boundary before evaluating the desired assertion. Root retains architecture
and release decisions.
