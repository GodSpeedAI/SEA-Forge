# Phase A lifecycle source and fixture result

Date: 2026-10-07  
Status: source-only TESTFIRST preparation. The lifecycle algorithm is still
unwired; this record makes no compilation, test, runtime, or GREEN claim.

## Instruction, approval, and scope

The complete release instruction was archived before source edits in
`run-observation-manager-phase-a-testfirst-assignment-oct07.md` (SHA-256
`7211787b2f4a07db3d8cddb031ee7f644f38ac5b5331f12ab0de54d5fec66565`). It
authorizes exactly three Go paths plus scoped evidence. The full Unit 1
assignment, preregistration, revision 6/addendum/retention corrections 1–3,
corrected proposal `979...`, correction supplement `4752...`, approved review
`a3b59162436f5031d5ace6127bf847f00458bc9c14ed55f3cf306522a15e17e7`, and
citation erratum were read with the worker launch, initial-failure,
signal/ref-owner, read-start, combined-image, and lease-cardinality decisions.
The source approval is for test-first readiness only; Phase B still requires a
different source reviewer, a root-authorized focused expected RED, and a new
release.

## Exact source identities and patch scope

| Path | Preimage | Current SHA-256 | Change |
|---|---|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `ab9f1c35757aecd8c8c02de300a95205a1bd07aa5e39ebe8ae056dbc8878fe62` | `e2b0dbf8b80c99d2f82d00c5b7a9df4b9fc2cadb0ea059603520c8933b9bc767` | Added private cohort membership, creator-operation/Stop ownership state, poller phase/result/current, lease drain state, and compile-safe unwired begin/reserve/exact-ref-release boundaries. `prepare`, `stopAndDrain`, and `detachAndDrain` remain the existing ordinary typed-unavailable stubs. |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | Absent before this unit | `f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f` | New file with compile-safe unwired batch launch, read-eligibility claim, worker entry, and mandatory worker continuation signatures. |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go` | Absent before this unit | `3b0f0653892cb8bcd9823a91779ffbd03e37d015c41242762091bdfad8233990` | New focused lifecycle fixture with bounded waits and the cases listed below. |

The manager source diff is limited to private lifecycle state and named
unwired methods. No old method body changed. The worker bodies intentionally
do nothing (and the read claim returns false); no success behavior is
fabricated. The exact-ref helper returns false and performs no map mutation.
These stubs are expected to fail positive assertions once the focused RED is
separately authorized; they are not a runnable lifecycle implementation.

## Fixture case-to-clause map

1. `TestRunObservationManagerFailureStopWinsBetweenReserveAndLaunch` stages
   real admission/reservation, observes manager Stop win, then launches the
   creator-owned batch and finishes creator ownership. It asserts no read,
   code 4, phase 2, readiness, worker completion, and Stop waiting through the
   launch/operation boundary.
2. `TestRunObservationManagerFailureDetachWinsBeforeFirstReadUsesDrainingCode4`
   stages detach before launch and asserts phase 3/code 4, no current/read,
   ready and worker completion before detach returns.
3. `TestRunObservationManagerFailureClaimThenStopUsesWorkerContinuation`
   requires a positive pre-Stop eligibility claim, performs real manager Stop,
   calls the mandatory production continuation, and asserts no actual first
   read, code 4/nil current, readiness, and JOIN.
4. `TestRunObservationManagerFailureStopJoinsHeldActualRead` waits for an
   actual recurring `ReadRunTrace` call to enter, proves the entry remains
   registered while held, checks the cancel function can acquire `m.mu`, and
   requires Stop/JOIN before entry removal.
5. `TestRunObservationManagerFailureGlobalStopReleasesDistinctPendingLeases`
   checks two distinct Prepare lease refs on one held initializer, then
   verifies Stop returns typed failure/zero DTO/nil lease for each and removes
   both refs before its single worker JOIN completes.
6. `TestRunObservationManagerFailureCanceledPrepareLeavesAuthorizedLease`
   cancels one pending Prepare after a second distinct lease attaches, checks
   that exactly one ref remains and shared worker context remains live, then
   releases the read and requires the authorized Prepare to succeed.
7. `TestRunObservationManagerFailureDetachAndSelectedAReleaseHaveOneOwner`
   races the production exact-ref release boundary against lease detach while
   an actual initializer call is held; it requires absence from both exact
   membership maps and worker JOIN after that call returns.
8. `TestRunObservationManagerFailureSharedFirstReadErrorIsSuccessfulA`
   shares one held, actual failing first trace call between distinct leases.
   It asserts successful A/non-nil distinct leases, all six DTO count pointers
   (`R=1,S=1,V=0,U=0,A=1,O=0`), no Runs, owner/waiter `ReadsAttempted=1/0`,
   one list per Prepare, one read, exact refs removed, and worker JOIN before
   the poller entry disappears.
9. `TestRunObservationManagerFailureOversizedNilKeyIsAWithoutAdmission`
   proves its selected valid long `planItemID` exceeds the combined
   nil-current image limit, then requires successful A/non-nil lease, zero
   reads, one allowed list, no entry/ref/worker, and retained cohort membership
   until detach.

The existing frozen manager fixture remains unchanged and still owns its
16-preparing/16-poller capacity coverage, including all eight starts per
selected batch before waiting (`run_observation_manager_test.go:488-639`). No
test in the new fixture uses a sleep, scheduler callback, fake pre-invocation
port barrier, generic hook, or test-only production state.

## Frozen-source verification and deviations

The following exact hashes were rechecked after edits and remain unchanged:

| Frozen path | SHA-256 |
|---|---|
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

The new worker file and the exact-ref boundary are the compile-safe names
chosen for the approved reserve/launch/read-claim/ref-release design. They are
not callbacks or persistent serialized fields. Manager cohort cardinality is
represented by exact lease membership rather than a duplicate scalar count;
the WaitGroup is separately reserved for creator operation ownership. No new
enum, code, ID grammar, cap, public interface, `Next`/C-2 behavior, or helper
algorithm was introduced.

No tests, compiler, formatter, scanner, Git operation, or runtime command was
run. In particular, no expected-RED result is claimed. A different source
critic must review this exact delta against the complete assignment and
approved design before root authorizes the focused RED. Any signature or
fixture correction belongs in a new bounded review/repair unit; Phase B must
replace the marked stubs with a wired implementation and preserve every
positive assertion above.
