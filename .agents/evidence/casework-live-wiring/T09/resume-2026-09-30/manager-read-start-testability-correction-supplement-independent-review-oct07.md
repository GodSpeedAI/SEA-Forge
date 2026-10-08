# Independent review: read-start lifecycle correction supplement

Date: 2026-10-07  
Verdict: **APPROVE the corrected bounded lifecycle proposal and its future test-first source/fixture release boundary.** This is architecture/readiness approval only; root retains source release and implementation review.

## Reviewed records and identities

I read the complete three-finding repair assignment (SHA-256
`05be98816e173eb93fbb42f32b825708404e0ea484bc5d04cf8df78d73ff0da2`), the
complete correction supplement (SHA-256
`4752e00fa7cc4001d220635cb4aebda4cd63df61b43b2219d60beddff5e114c0`), and
its full target, corrected proposal SHA-256
`979fbdf521377c58f2615a2b3111c089fa9e0869fd350a0e4c973ae3c9166f8b` with
assignment `28313e82372adfc7ce55cb008594da796c3c43cb06ce9682b1e5bd0fd45de25f`.
I also read the complete prior rejection, original Unit 1 assignment,
lifecycle preregistration, revision 6/addendum/retention corrections, root
initial-failure and worker-launch clarifications, root read-start decision
and review, root signal/ref-ownership decision, combined-image decisions,
and per-Prepare lease-cardinality supplement and review.

Current source hashes match the correction's intended frozen boundary:
manager scaffold `ab9f1c35757aecd8c8c02de300a95205a1bd07aa5e39ebe8ae056dbc8878fe62`,
existing manager fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`,
key `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b`,
encoder `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`,
encoder fixture `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`,
retained helper `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`,
base helper fixture `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`,
and formatted policy fixture `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28`.
The corrected supplement was written while the formatted policy hash was
pending, so it carefully labels `4c70...` as the former pre-format identity
and does not assert it as current. The exact `e156...` identity is now
available and should be used by any follow-on source assignment.

## The three rejected gaps are resolved

1. **Cohort admission occurs before list/read.** The supplement makes the first
   manager transition after caller/auth/guard checks an atomic decision under
   `m.mu`: reject if stopping or 16 preparing/active/draining cohorts already
   exist; otherwise reserve an exact cohort lease and register its creator
   operation together. The manager-owned lease registry is the cardinality
   source; a WaitGroup alone is explicitly insufficient. This matches the
   original preregistration requirement to reserve before list/read and the
   frozen 17th-preparing/17th-active zero-call assertions
   (`run_observation_manager_test.go:488-553`). The manager scaffold currently
   has only a poller map and per-lease membership, so adding the exact
   manager-owned cohort membership in the one authorized manager source file
   is a necessary private lifecycle change, not a fake test seam
   (`run_observation_manager.go:66-78`).

2. **The two missing successful-A manager cases are explicit.** For one real
   shared first-read failure, the supplement requires distinct successful
   Prepare leases, one poller/worker/read, owner/waiter actual-start counts
   `1/0`, per-Prepare DTOs and counts, no Runs row, each exact failed ref
   removed once, and real worker return/JOIN before reuse. This follows the
   root clarification that the same fixed poller failure result does not mean
   byte-identical DTOs; successful A returns nonnil leases and is distinct
   from failed Prepare rollback (`manager-initial-failure-root-clarification-oct07.md`).
   For the oversized nil-current image, the row is a valid selected candidate
   with a bounded guard identity and overlong opaque `planItemID`; expected
   counts are `R=1,V=0,A=1,C=0,O=0,U=0`, zero actual trace reads, no poller
   entry/ref/worker, and a successful nonnil lease. The supplement correctly
   says the pure encoder case cannot prove manager classification or read
   suppression. The phrase “no port call” in §23 should be implemented as
   **no `ReadRunTrace` call**: the valid selected row necessarily comes from
   the list operation. That is a wording clarification for the fixture
   assertion, not a design gap.

3. **Stop and cancellation obey the mutex boundary.** The revised sequence
   takes `stopping`, single stop ownership, and cancellation/work snapshots
   under `m.mu`, then unlocks before invoking cancellation or closing signals.
   It waits for Prepare operations outside the lock, ensures creators launch
   every owned worker exactly once before any readiness wait, and joins workers
   after creator launch completion. The corrected proposal already requires
   `finishPrepareOperation` before a creator waits on a drain that includes
   its own operation (`manager-read-start-testability-corrected-proposal-oct07.md:85-106`).
   These rules preserve no self-join, no positive operation registration
   after Stop closes admission, real worker completion, and actual JOIN before
   capacity release. No callback/test hook or mutex-held cancellation is
   introduced.

## Protocol completeness and release boundary

The amended matrix preserves the accepted mutex-linearized read claim and
the real `runPollerFromClaim` runtime continuation. It distinguishes
stop-before-claim and claim-then-cancel-before-invocation from an actually
started held read; only an actual `ReadRunTrace` call increments
`ReadsAttempted`. Code 4/phase 2-or-3/nil current/ready-once/zero calls remains
failed Prepare with typed error, zero wrapper, nil lease. A real initial
unavailable read remains successful A. The worker alone resolves readiness
and closes `workerDone`; cancellation, close, waits and JOIN stay outside the
mutex (`corrected-proposal-oct07.md:108-149`; `manager-signal-ref-ownership-root-decisions-oct07.md:26-39`).

The earlier corrected matrix remains binding alongside this supplement:
deterministic real reservation-to-launch Stop ordering, claimed cancellation,
held-call return/JOIN, global Stop across separate leases, one-Prepare
rollback while another lease survives, detach-versus-A exact-membership race,
and all-eight-before-wait. The lease-cardinality supplement repairs the
earlier ambiguity by distinguishing global Stop across distinct Prepare
leases (3A) from one Prepare rollback while a distinct lease survives (3B),
and preserves per-lease ownership, per-lease drain completion, operation
finish-before-join, one shared worker, and cancellation only when watcherless.
The checked-in manager fixture already has the required 16 preparing/active
cohort and all-16-initializers-before-wait cases at
`run_observation_manager_test.go:488-639`; the correction keeps that fixture
frozen.

The proposal's exact future path boundary remains the existing
`run_observation_manager.go`, one new private worker source, and one new
`run_observation_manager_failure_test.go`; the existing manager fixture and
the six published pure primitive files remain byte-identical. Private names,
signatures, and compile-safe plumbing may be finalized by the released
builder, but must implement the required real reservation, launch, read
eligibility, and worker-completion methods. No generic test hook, new public
API, new initializer code, serialized field, cap change, helper modification,
or test weakening is allowed.

No unresolved architecture defect remains in the three corrected findings.
One implementation-relevant wording point is to assert zero trace reads for
the oversized-key A case while still allowing its necessary successful list
call. The pending policy hash is provenance only; follow-on instructions
should use `e156...`.

The isolated canonical recipe and full-module race gates have now passed
under the separately authorized normal listener permissions, with outputs
archived in `run-observation-primitives-canonical-retry-escalation-result-oct07.md`.
Those gates cover only the six primitive files. They do not test a lifecycle
worker, manager reservation, A cleanup, ready signaling, cancellation, or
JOIN. This approval is for a future test-first lifecycle source/fixture
release only; it approves no lifecycle implementation or runtime behavior.
Root retains the release decision and implementation must receive a fresh
source review before any lifecycle GREEN claim.
