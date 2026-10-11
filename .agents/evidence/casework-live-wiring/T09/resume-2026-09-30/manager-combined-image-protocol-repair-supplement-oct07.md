# Combined-image lifecycle protocol repair supplement

Date: 2026-10-07  
Status: immutable source-only supplement for independent review; no source/test release.

## Purpose and records

This supplement repairs exactly the two blocking protocol ambiguities in
`manager-combined-image-testfirst-assignment-repair-oct07.md`
(SHA-256 `477ee9aa26d3b68d6cc2a01ef72d0bed3a4cf358345950129c3068f40b937fdc`).
The complete new-builder assignment was archived first as
`manager-combined-image-protocol-supplement-assignment-oct07.md`.
Preserve that assignment and every earlier record unchanged.

The fresh independent repair review
`manager-combined-image-testfirst-assignment-repair-independent-review-oct07.md`
(SHA-256 `c010c8ee2bc05bdafd07e25b286e786da10e483d84258085c902cbecfb572f88`)
identified: (1) two possible owners for releasing a selected poller reference
during lease detach and waiter wakeup; and (2) no existing initializer code
assigned to a poller stopped before its first read.

Root resolved both findings in
`manager-signal-ref-ownership-root-decisions-oct07.md`
(SHA-256 `5b9888e5a7212b8d0696f790822f08e718dd03a71f51b28c2bec2ea0098f8df4`).
The controlling initial failure rule remains
`manager-initial-failure-root-clarification-oct07.md`
(SHA-256 `34750ceb5e7dc99b0a7c3755c514f6a7fbdb6fd31ac60d99833fad181ed117c2`).
The combined image contract remains
`manager-combined-image-root-decisions-oct07.md`
(SHA-256 `48ed5d0b01e25a080b8b2560462bfd6bdf4b196d9769ff917b6a9cb25b32fe96`).

Other governing records read for this repair are the frozen combined-image
correction `manager-combined-image-testfirst-assignment-correction-oct07.md`
(hash `5f5418f16193d41476f69554e366ff83d7490c68a917462f3a76252f050466ee`),
combined-image proposal `run_observation_manager_combined_retained_image_design_proposal_oct07.md`
(hash `e436298c9822a7e405e723ebba505ead8ad49006e246ab983a02b2ad88f9760c`),
durable lifecycle preregistration (hash
`f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5`),
frozen Unit 1 original assignment (hash
`de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`),
revision 6 (hash `60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`),
its addendum (hash `9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`),
and retention corrections 2 and 3 (hashes
`3494e3de99b2f5d3fe725a85971dccd0d70cef226da257ce237cce7fba2d72e1` and
`b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`).

All earlier scope and image decisions stay in force: pure retained helper is
unchanged; current-value image is the explicit combined wrapper; only the
exact nil-current key or the raw helper image is encoded; phase and initializer
result stay fixed one-digit codes; no helper generation is reused as an entry
token; no stop/error fields or enum code 7 are added. The image and ref counts
are not heap/RSS bounds. The pure image encoder fixture remains separate from
future manager lifecycle tests.

Frozen source identities remain: manager
`fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`, manager
fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`,
formatted pure helper
`2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`, base
helper fixture `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`,
and policy fixture
`4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.
Do not edit any of them.

## Exact replacement clause: one owner for ref removal

This clause replaces the ambiguous lease-detach/waiter wording in the prior
repair. The actual manager mutex is the only arbiter of reference ownership.
No extra drain-owner counter, persistent field, JSON field, or enum value is
introduced.

1. **Lease drain wins.** The first caller that changes a preparing/active lease
   to draining while holding the actual manager mutex is the sole lease-drain
   owner. In that same critical section, it removes every still-present exact
   ref owned by that lease from both lease membership and poller membership.
   It snapshots the workers that have become watcherless and claims the
   existing lease-done notification. After unlocking, that owner closes the
   notification once, cancels only the captured watcherless workers, and
   performs operation/ref/notifier waits and worker joins outside the mutex.
   It removes/releases capacity only after the real work is joined. Timeout
   leaves the lease/pollers counted and draining.
2. **A-result cleanup wins first.** While the lease is still preparing, the
   Prepare doing cleanup for one selected A row locks the actual manager
   mutex and checks that its exact `(lease,poller)` membership still exists.
   If present, that Prepare removes only that exact ref in the same critical
   section. It captures a worker for cancellation only if that removal leaves
   the poller with no other eligible lease refs. It unlocks before any cancel
   or wait. A later lease drain owns only refs that remain.
3. **Lease drain wins against A cleanup.** If the lease is already draining or
   the exact membership is absent when A cleanup checks under the mutex, A
   cleanup is a no-op. It must not decrement a count, cancel a worker, or
   attempt a second ref removal.
4. **Awakened waiters.** A waiter awakened by lease-done, ready, or manager
   stop reacquires the actual mutex and rechecks state. If the lease is
   draining, it owns no ref cleanup: the lease-drain owner already removed all
   remaining refs atomically. The waiter completes its own Prepare-operation
   registration and then joins/waits on the same drain's existing completion
   handles outside the mutex. It never starts a second drain owner or removes
   an already-absent ref. If the lease remains preparing and the wake is an
   ordinary shared initializer result, the waiter may perform only its own
   exact A-ref cleanup using rule 2.
5. **No cancellation under lock.** Reference removals are decided from actual
   membership under the manager mutex. Cancellation occurs after unlocking
   and only for a poller that has no remaining eligible ref. A surviving
   authorized lease keeps shared work alive.
6. **No self-join.** A Prepare that fails and performs internal rollback must
   not wait for a drain that is waiting for that same Prepare operation.
   Finish/drop the failing Prepare's operation registration before awaiting a
   drain completion that includes operation quiescence. Preserve the original
   typed failure, zero wrapper, nil lease, and internal rollback. The same
   Prepare that returns a successful A DTO is not a failed-Prepare rollback;
   its returned cohort lease remains valid, including when it has no poller
   attachments.

The transition to draining itself identifies the unique drain owner; there is
no separate owner flag. Competing detach callers that find the lease already
draining only join the same completion path. Signal close/cancel/wait/JOIN
remain outside the manager mutex.

## Exact replacement clause: stopped before first read

This clause replaces the prior phrase “generic non-observable stopped result.”
Use only the existing initializer result code **4** (`invalid/unavailable`)
and existing poller phase code **2** (`stopping`) or **3** (`draining`):

- A reserved worker that has not yet started its first `ReadRunTrace` checks
  phase under the actual manager mutex. If stop/drain already won, it retains
  `Current == nil`, starts no port call, records `InitResult=4`, and leaves
  phase at the existing stopping (2) or draining (3) code selected by the
  winning transition. It records no key/trace-derived error text and invents
  no current value.
- The sole manager-owned worker resolves its existing ready signal exactly
  once after installing that fixed result, then unlocks and closes ready
  outside the actual mutex. A stop caller does not also close ready. The worker
  closes `workerDone` on its own return. A reserved entry must still have its
  worker launched if stop races immediately after reservation, so readiness
  and worker completion resolve rather than stranding waiters.
- Every awakened Prepare reacquires the actual mutex and rechecks manager and
  lease stop state. If stopped, stop wins even if ready also resolved: return
  the existing typed stop/cancellation error, zero DTO, and nil lease, with
  internally owned rollback. This path is failed Prepare, not a successful A
  result. No run read starts and no candidate data is disclosed.
- The real worker completion and JOIN remain required before entry/slot release.
  Capacity remains counted during worker startup, ready notification,
  cancellation, and drain. All close/cancel/wait/JOIN operations occur outside
  the actual manager mutex. No seventh initializer result or new phase code is
  permitted.

A genuine first read that runs and returns unavailable/invalid/unretainable is
a different case. Under the existing initial-failure root clarification it is
successful A: normal unavailable initial DTO, all successful-list counts
present, no Runs row, no retry, and nonnil cohort lease. Owner Prepare counts
one actual logical start; a shared waiter counts zero. Each Prepare releases
only its own failed poller ref using the membership rule above. Do not turn
code 4 for a stopped-before-read path into this successful A result.

## Required deterministic future lifecycle tests

These are fixture-first requirements for a separately released manager
lifecycle unit. They do not belong to the pure image encoder's test file. Use
channel-controlled barriers, bounded waits, and the actual manager mutex;
never use sleeps to manufacture a race. Extend only a new lifecycle fixture
after independent review and a new root release. Frozen manager, helper, and
policy fixtures remain byte-identical.

1. **A-ref cleanup before detach.** Attach a lease ref to a poller also held by
   a second authorized lease. Cause the selected result to be A; force the
   Prepare's exact-membership cleanup to win before lease drain. Assert only
   that ref is removed, the survivor ref remains, cancellation count stays
   zero, and later removal of the survivor cancels exactly once after unlock.
2. **Detach before A-ref cleanup.** With the same shared setup, force the
   first transition to draining to win. Assert it removes all refs for that
   lease atomically. Let selected-A cleanup run afterward and prove it observes
   absence and causes no second decrement/cancel. The other authorized ref
   remains usable. Assert final ref membership/count is exact and no
   underflow/double release occurs.
3. **Awakened waiters use the one drain owner.** Hold multiple Prepare waiters
   at ready, then transition their lease to draining. Prove the owner removes
   their refs once; every waiter wakes, rechecks draining under the mutex,
   performs no ref removal, finishes its own operation registration, and joins
   the same drain completion. Assert no waiter is stranded, no second owner
   appears, and a surviving other lease prevents worker cancellation.
4. **Failed Prepare does not self-join.** Block an owned read, then cause a
   Prepare failure/cancellation whose internal rollback owns drain. Assert the
   failing Prepare drops its own operation registration before waiting on
   operation quiescence; it returns the original typed error with zero DTO and
   nil lease without deadlock. The poller/cohort slots remain counted until
   the blocked read returns through retirement and the owner joins workerDone.
5. **Stopped before the first port call (phase table).** Reserve/launch a
   manager-owned worker behind a channel gate. In one case make global stop
   win under the mutex (phase 2); in another make lease teardown win (phase
   3). Release the gate. For each case assert init code 4, current nil, no
   `ReadRunTrace` call, ready closed exactly once outside the actual mutex,
   Prepare's post-wake stop recheck returning typed error/zero DTO/nil lease,
   and no successful A result. Assert workerDone closes on actual worker
   return and no capacity is released before its JOIN.
6. **Ordinary shared first-read failure remains A.** Two Prepare calls share
   one initializing poller whose one real first read returns unavailable.
   Assert exactly one logical read, no retry, owner `ReadsAttempted=1`,
   waiter `ReadsAttempted=0`, both successful unavailable DTOs with counts
   `R=1,V=0,A=1,C=0,O=0,U=0`, no Runs row, and nonnil cohort leases. Assert
   each caller's failed ref is removed once; if detach wins a ref, cleanup is
   a no-op under the same membership rule.
7. **Eight starts before waiting remains intact.** Preserve the frozen
   `TestRunObservationManagerPollerCapCountsInitializingEntries` behavior:
   reserve and launch all 16 selected reads across two eight-row cohorts
   before either Prepare waits for readiness; the third cohort is capacity
   limited and starts no 17th read. Keep the existing test file frozen.
8. **Stop/ready/ref drain idempotence.** Race manager stop, lease detach, ready
   publication and waiter wake using controlled channels. Assert the actual
   mutex transition selects a single owner, each signal closes once outside
   the lock, no read begins after stop, waiters recheck state, only watcherless
   workers are canceled, workerDone follows actual port return/retirement, and
   timeouts retain capacity.

The pure image encoder can prove JSON branch shape, fixed-width codes, exact
combined length, immutable marker bytes, and no fake nil-state value. It cannot
prove any ref owner, no-read-after-stop, A-result cleanup, no self-join, ready
wake, actual port return, or JOIN. Keep those lifecycle assertions separate.

## Material differences and release limit

This supplement supersedes only the two ambiguous protocol clauses in the
rejected repair:

- Ref release is no longer assigned both to a detach owner and awakened
  waiters. The actual mutex winner owns the exact removals; Prepare-only A
  cleanup is a separate exact-membership removal, and a detach winner makes
  later cleanup a no-op.
- A worker stopped before its first read now has the explicit existing result
  code 4, current nil, and phase 2 or 3. Prepare's post-wake stop recheck is a
  failed-Prepare result, not successful A.
- New lifecycle tests are explicitly separate from pure encoder tests and
  preserve the initial actual-read-failure A outcome and all-eight-starts
  constraint.

All other clauses of the rejected repair, its combined-image proposal, the
durable lifecycle preregistration, revision 6/addenda/corrections, and root
decisions remain unchanged. In particular, no new serialized controls, raw
error values, helper behavior, APIs, Next/delta/SSE behavior, source continuity
claim, or public integration is introduced. Existing current manager/source
and frozen fixture hashes listed above must remain exact.

This supplement requests review from the different critic. It does not release
even the separate pure image fixture/source phase; root must make a new
explicit release after the critic accepts the exact documents. No source/tests,
compiler/build/scanner, runtime, or Git operation was performed for this
documentation repair.

