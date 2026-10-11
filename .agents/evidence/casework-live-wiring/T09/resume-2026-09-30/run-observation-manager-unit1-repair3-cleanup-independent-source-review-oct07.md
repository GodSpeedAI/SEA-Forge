# Run observation manager Unit 1 repair 3 — independent source review

Date: 2026-10-07  
Disposition: **APPROVE only one focused expected-RED review; no manager implementation or runtime approval**

## Scope and provenance

Reviewed the complete Unit 1 original assignment, frozen revision 6 proposal and addenda/corrections/response-line-cap erratum, root private-design decision and its review, repair 2 record/hash receipt and full independent rejection, repair 3 cleanup instructions/record, full current manager source and test file, and exact repair 3 pre-edit copies.

Candidate identities verified directly:

| Artifact | SHA-256 |
|---|---|
| `run_observation_manager.go` | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| `run_observation_manager_test.go` | `b1b585320ef65ed1cb27a2b618bcca4a8b2e368c9646e96a6229e6855e91c72a` |
| `run_observation_manager_repair3_preedit.raw` | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| `run_observation_manager_test_repair3_preedit.raw` | `6bc6666ee176a56e233636bca0d72fcda439e430a03c0642e4fd2ad695af826a` |

The private manager source is byte-identical to its repair 3 pre-edit copy. A direct unified diff of the test copy shows only `cleanupRunObservationManager` changed. The repair 2 review documented that the immediate repair 1 preimage (`aad810...` / `64dc...`) was unavailable and that no exact repair1-to-repair2 diff could be claimed. Repair 3 now preserves the immediate repair 2 preimage exactly; it does not retroactively remove that historical provenance limitation.

## Repair 2 finding and cleanup review

Repair 2's remaining blocker was that cleanup accepted any `KindUnavailable` from `stopAndDrain`, even with owned entries or unjoined workers. The current helper at `run_observation_manager_test.go:177-257` addresses it:

1. Cancels supplied caller contexts, snapshots actual manager-owned entries, cancels, and `workerDone` channels under `manager.mu`, then unlocks.
2. Calls captured worker cancels outside the mutex, starts a two-second bounded `stopAndDrain`, and releases controlled fake reads outside the mutex.
3. Awaits stop, then awaits every captured non-nil `workerDone` under the same deadline.
4. Rechecks the real entry map under the mutex and requires it to be empty.
5. Accepts typed `KindUnavailable` only if the pre-stop snapshot and post-stop map are both empty; unavailable with captured work, missing `workerDone`, remaining entries, other errors, or timeout fails cleanup.

The deliberately unwired stub owns no poller entries and returns typed unavailable from `stopAndDrain`, so this cleanup does not mask the intended semantic RED when no work exists. It does not cancel, wait, release fake reads, or join while holding the manager mutex. All cleanup call sites were checked (`:487,599,697,808,1012,1087,1159`); their held reads have release functions and/or caller cancellation registered before goroutines start. The relevant tests use buffered result channels, explicit channels for read start/return/release, bounded timers only as failure deadlines, and `runtime.Gosched` only while observing actual mutex-protected ref membership. The cleanup diff adds no callback, duplicate ownership counter, or production hook.

I found no remaining repair-3 blocker in the cleanup helper or the prior fixture repairs. The helper's ownership snapshot is not an independent lifecycle implementation; future `stopAndDrain` must itself enforce stop-before-removal and actual join-before-capacity-release. The helper detects captured workers that do not join and entries left in the map. This review does not claim tests compile or execute.

## Unit 1 boundary and expected-RED suitability

The current source is an unexported, unwired compile seam. It defines only the approved private ownership representation (`mu`, exact-key `pollers`, reciprocal lease/ref fields, readiness, manager-owned context/cancel, and worker completion channel), identity snapshot without bearer/session pointer retention, and existing-source authorization/present-context seams. `prepare`, `stopAndDrain`, and `detachAndDrain` remain explicit typed-unavailable stubs. No manager algorithm, `Next`, public API/SSE, selector/hydration/guard implementation, auth policy, dependency, or caller wiring was added. The source and tests do not claim full behavior.

The focused `TestRunObservationManagerCanceledPartialPrepareJoinsOwnedReadBeforeRetry` fixture has valid current session/claim, verifier, present-context cursor, and matching parents. Its expected-RED helper distinguishes `prepare` returning the unwired typed-unavailable stub before the controlled trace read from either a timeout or successful attachment (`run_observation_manager_test.go:970-1055`, helpers `:110-132`). That is a semantic stub RED, not a fixture/setup failure. On this early return, cleanup sees zero owned pollers, accepts only the stub's typed unavailable drain result, and terminates without held fake work or an unjoined goroutine. The other lifecycle fixtures remain unexecuted source test cases; their presence is not claimed as RED proof.

The four repair 2 fixture findings remain repaired in the full candidate: retry start has its own signal before its gate (`:982-1004`); shared-initializer cancellation waits for actual mutex-protected lease refs (`:1068-1115`); the draining poller-cap fixture waits for all 16 exact unique later reads and verifies A/B ownership/cancellation before probing C (`:751-939`); and cleanup cancels callers, releases fake reads, bounds stop, joins workers, and checks the real map. The earlier four repair 1 findings are reviewed through the full repair 2 report and current fixture: the retry deadlock, initiator-cancellation ref race, incomplete 16-read premise, and release-only leaked-worker cleanup are addressed.

## Approval boundary and limits

Approve this frozen source/test candidate only for root to select and run one separately authorized focused expected-RED case. This is no claim that the expected RED actually occurred, and it approves no algorithm, production implementation, integration, manager lifecycle correctness, source behavior, GREEN test, or T09 settlement. No compiler, test, formatter, typecheck, scanner, build, Git, or runtime command was run for this review. The old repair 1 preimage gap remains explicitly disclosed above.

Graft first-pass retrieval saved approximately 109,638 tokens ($0.09) this turn.
