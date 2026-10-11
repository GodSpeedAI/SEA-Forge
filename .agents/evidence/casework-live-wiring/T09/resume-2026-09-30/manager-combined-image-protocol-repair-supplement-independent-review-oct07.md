# Independent review: combined-image protocol repair supplement

Date: 2026-10-07  
Verdict: **REJECT the combined package for test-first lifecycle readiness pending one test-matrix clarification.** The pure encoder assignment is independently well-scoped.  
Review scope: source/design only; no source or fixture edits, compilation, tests, build, scanners, or Git operations.

## Reviewed artifacts

- Full `manager-combined-image-protocol-supplement-assignment-oct07.md`.
- Full `manager-combined-image-protocol-repair-supplement-oct07.md`.
- Full `run_observation_poller_image_testfirst_assignment-oct07.md`.
- `manager-signal-ref-ownership-root-decisions-oct07.md`, initial-failure clarification and its independent review, previous assignment and rejection, combined-image proposal and decisions, full original Unit 1 assignment, lifecycle preregistration, revision 6/addendum, and retention corrections 2–3.
- Current unwired manager and relevant frozen lifecycle fixtures, read-only.

## Root’s two protocol decisions are correctly applied

The supplement resolves both prior defects without changing bounds or encoding:

- The first lease-drain transition under the actual manager mutex is the sole owner of that lease’s remaining ref removals. Prepare’s successful-A cleanup is a separate exact-membership removal while the lease remains preparing. Whichever transition removes membership first wins; the other observes absence and performs no decrement/cancel. Awakened waiters recheck state and do not release already-removed refs. This matches revision 6’s exact-reference rollback, shared-worker cancellation only when watcherless, and drain-before-capacity-release rules.
- A worker stopped before its first read uses existing initializer code 4 and phase 2 or 3, retains nil current, starts no port call, resolves ready once outside the mutex, and closes workerDone on its actual return. Prepare rechecks stop and returns the failed-Prepare zero/nil/typed-error outcome with owned rollback. This correctly differs from a real first read returning unavailable, which remains successful A under root’s clarification.
- The no-self-join clause is directionally correct: a failing Prepare must finish its own operation registration before awaiting drain quiescence that includes it. This is required by the original assignment’s owned rollback model.
- The pure image encoder and future manager lifecycle fixture remain separate. The encoder API/test assignment accurately limits its claims to deterministic branch encoding, code ranges, combined byte budget, marker bytes, immutable returned bytes, and fixed generic errors. It explicitly does not claim map admission, read suppression, A cleanup, readiness, cancellation, worker return, or JOIN.

## Remaining issue: lifecycle test 3 does not match lease ownership cardinality

The future lifecycle test list §“Required deterministic future lifecycle tests,” case 3, asks to hold “multiple Prepare waiters” and then transition “their lease” to draining; it further expects all of them to join the “same drain completion.” The Unit 1 `prepare` contract creates and returns one cohort lease per Prepare call. The frozen manager source has one `runObservationLease` value with its own poller-membership map; two distinct Prepare calls therefore have two distinct leases. The root decision’s sole drain owner is per lease, not shared across different leases.

This leaves the claimed race/test shape ambiguous: if several waiters are distinct Prepare calls, they do not share one lease drain completion. A lease-specific detach owns only that lease’s refs; it cannot clear the other Prepare leases’ refs. If the intended event is manager-wide Stop, that operation wakes multiple Prepare calls but drains each lease’s membership through its own per-lease transition, and there is no surviving authorized lease under the same manager stop. If the intended event is request cancellation/internal rollback, each Prepare has its own lease drain owner/completion; the test must not call these one shared drain. The current case combines these models and so does not precisely prove the protocol it names.

Please split or rewrite this case in a new immutable supplement. A precise plan could distinguish:

1. **Several pending Prepares stopped globally:** use multiple independent Prepare calls/leases sharing one initializing poller; manager Stop wakes all, each lease’s first transition removes only its own memberships, each waiter finishes its Prepare operation before joining its lease’s completion, and the manager drain joins the worker once. No surviving lease is expected after global stop.
2. **One pending Prepare rolls back while another lease survives:** two or more independent leases share one poller; cancel/fail one pending Prepare so its sole lease-drain owner removes only its refs; assert the other lease remains authorized and the shared worker is not canceled. Its surviving Prepare receives the single initializer result.

In either case assert one owner per lease, one removal per exact membership, no waiter removes absent membership, and no waiter waits on a completion that is waiting for that same unfinished operation. Avoid saying all Prepare waiters share the same lease or drain completion.

The individual detach-before-A and A-before-detach tests (§cases 1–2) otherwise express the root decision well. The stopped-before-read matrix (§case 5) explicitly tests phases 2 and 3, code 4, nil current, no read, ready-once, post-wake stop recheck, and capacity through JOIN. The genuine shared first-read A test (§case 6) preserves the root clarification’s per-Prepare counts and nonnil cohort leases. The all-eight-starts test names the frozen capacity fixture and respects its existing source boundary.

## Material differences from the full original direction

- This release remains only a pure private encoder assignment, narrowed from the original private Prepare/admission/rollback lifecycle work. The future lifecycle fixture remains a separate source/approval step; the supplement does not claim it is implemented or tested.
- The wrapper’s canonical JSON budget and helper composition come from the later preregistration and root decision, not the Oct 6 bounded assignment’s initial “do not implement JSON budget yet.” They are isolated in a new encoder file and do not modify the pure helper or existing fixtures.
- Pointer identity replaces the illustrative scalar initializer token only because root expressly approved exact entry-pointer identity plus single publisher and JOIN-before-reuse. The supplement preserves that proof condition.
- Initial actual-read failure is successful A with a nonnil cohort lease per the later root clarification; stop-before-read is instead a failed Prepare using code 4 with typed error/zero DTO/nil lease. The supplement keeps these branches distinct.
- Frozen manager, helper, base-test and policy-test identities are preserved. The proposed focused prefix excludes intentional unwired manager stub tests, and no broader package run is implied.

## Release recommendation

The pure encoder assignment itself is ready for its separately authorized test-first step: its source surface, exact API, seven-case matrix, fixed error/byte ownership, and exclusions are reviewable and evidence-supported. The combined protocol supplement’s remaining future lifecycle case must be rewritten before approving it as a complete lifecycle test-first plan. Root may release the pure encoder fixture only under that bounded assignment and only after its explicit separate decision; this review grants no source/fixture/compiler work.
