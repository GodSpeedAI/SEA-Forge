# Independent Phase B algorithm source review

Date: 2026-10-08

## Review assignment and limits

Independently review the complete Phase B manager/worker source assignment and
result, governing revision 6 contracts, the frozen failure fixture and published
primitive files. Decide whether the actual two-file implementation is ready for
focused GREEN verification. Record concrete source evidence and every material
deviation. This is a source-only review: no compile, test, formatter, scanner,
build, Graft build, or Git operation is authorized. Do not approve runtime,
integration, or lifecycle correctness from static inspection.

## Inputs and identity verification

The complete source assignment is
`run-observation-phase-b-algorithm-source-assignment-oct08.md`
(SHA-256 `e5fc9e0aa9a8549aa75bcf78040b7012e79e8a882a176965dc80ce6eace8ef42`).
The implementation result is
`run-observation-phase-b-algorithm-result-oct08.md`
(SHA-256 `a964dc9c036050fa97f0b49c2ae5628d020043dbbc9b3fe47416df0f07326e48`).
I read the full assignment and result, the revision 6 proposal, and the current
manager and worker sources. The prior focused RED acceptance is bounded to the
12 compiled fixture cases and is not evidence that the algorithm is correct.
The original RED result's identity correction is in
`run-observation-manager-phase-b-focused-red-result-identity-erratum-oct08.md`
(SHA-256 `4429faf138cf39abc0fcbcf83ea29e4273f18c1803ff07a127b14721f8139c6c`);
the superseded receipt's incorrect source-path/hash labels are not reused here.

The assignment's exact preimages were checked against decoded JSON source bytes:

| File | Decoded preimage SHA-256 | Decoded bytes | Current SHA-256 |
|---|---|---:|---|
| `run_observation_manager.go` | `b7f0511b5a1690253d3adb0132beafc106d3b03f5218c6bde3f1c40f4217884a` | 7,052 | `d7be1d7a724ed65575ccaaf35cdf1aa5af133dbdd4952d592699b7eb443b6350` |
| `run_observation_poller_worker.go` | `f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f` | 1,056 | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` |

All eight frozen fixture/helper identities plus the existing manager fixture
and its Phase A failure fixture were independently hashed:

| Frozen input | SHA-256 |
|---|---|
| `run_observation_manager_failure_test.go` | `ef587b0bd64fe42d5fb56369f620f969610a5bac3d04cd7578911472b1a4893c` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

The last identity is materially omitted from the implementation result's
“Frozen inputs checked” table, although it is unchanged and its current hash
matches the expected fixture. This is a result completeness defect, not evidence
that the fixture changed.

## Verdict

**REJECT source readiness for focused GREEN.** The current source does not meet
the terminal lifecycle and per-watcher authorization/present-context contracts
that govern this implementation. The required watcher identity is also missing
from the lease. The release assignment's assertion that the existing fields
are sufficient conflicts with revision 6's explicit private identity state;
the root has confirmed that this no-identity assumption was erroneous. A
separate corrected architecture/source instruction is required before changing
source. This review does not authorize that change.

## Concrete findings

1. **A fitting accepted terminal trace continues polling.** In
   `run_observation_poller_worker.go:160-166`, `retainedCandidateAccepted`
   unconditionally sets `continuePolling = true`; no terminal-state check is
   performed on the accepted state. `runPollerFromClaim` then waits one second
   and claims another read (`:75-89`). Revision 6 says an accepted fitting
   terminal state is disclosed and then polling stops, with the worker joined
   before slot release (`run-observation-manager-concrete-proposal-revision6-oct06.md:467-468`).
   The result's claim that terminal markers initiate lease drains does not
   cover this accepted-terminal branch.

2. **The outer-image overflow terminal fallback misses the drain transition.**
   `finishPollerRead` computes `terminalStop` at worker `:205-206`, before
   validating the complete serialized poller image. If that later image check
   fails for a terminal snapshot, the fallback changes `phase` and `state` to
   terminal at `:216-228`, but does not recompute `terminalStop`. The entry is
   published as stopping (`:246-260`), while the cancel and per-lease drain
   loop is guarded by the stale value (`:248-270`). The worker exits because
   the phase is no longer running (`:272`), leaving refs and lease membership
   without the required drain owner. Revision 6 requires an unretainable
   terminal candidate to stop only after the read returns and the actual worker
   joins (`revision6-oct06.md:387-390, 467-468`). This code does not establish
   that lifecycle.

3. **Per-watcher reauthorization is absent and the required identity is not
   retained.** Revision 6 defines `cohortLease.identity watcherIdentity` and
   requires independent checks before list, each selected trace read/attach,
   initial handoff, and later disclosure (`revision6-oct06.md:246-258,
   331-338`). The current `runObservationLease` has only manager, pollers,
   draining, and lifecycle channels (`run_observation_manager.go:81-88`).
   `prepare` derives `caller` locally and calls `m.authorize` once at
   `:326-343`; that caller is neither stored nor available to the worker.
   There is no authorization check before each `ReadRunTrace` or at final
   handoff. The no-new-field/sufficient-existing-state assertion in the source
   grant cannot satisfy the governing watcher-identity requirement.

4. **Present-context/cursor checks do not bracket selected reads or handoff.**
   `prepare` calls `m.guard` before admission (`run_observation_manager.go:339-343`)
   and once after the list returns (`:405-407`). A selected trace read can
   occur after that check, and a cursor/present-context change during a held
   read is not checked before the initial event is returned. The final test at
   `:549-553` checks only cancellation and `leaseCanPrepare`, not authorization
   or the captured cursor. Revision 6 requires checks before each selected read
   and attach/publication and immediately before disclosure, with exact cursor
   equality (`revision6-oct06.md:331-348, 463`).

5. **The required cancellation check is not immediately before the port call.**
   `runPollerFromClaim` checks `entry.ctx.Err()` at worker `:60`, then calls the
   mutex-protected `pollerCurrentForRead` at `:64-68`, and only then invokes
   `ReadRunTrace` at `:69`. Cancellation can occur after the first check while
   the eligibility/current-state check runs. The released contract requires an
   outside-mutex context check at the actual read boundary. The current order
   leaves a cancellation window without that final check.

6. **Unreadable-count distinctness is inherited from the production adapter,
   not enforced by the manager.** The manager passes
   `len(list.UnreadableIDs)` as the unreadable count (`run_observation_manager.go:524-538`).
   The actual SFwP adapter guarantees uniqueness and disjointness: it rejects
   duplicate readable IDs and duplicate/overlapping unreadable IDs using one
   `seen` set (`apps/godspeed-casework-go/internal/adapters/sfwp/authority.go:284-304`).
   Thus the production adapter path supplies a distinct count. The manager
   itself assumes that port contract; arbitrary alternate `runObservationListReader`
   implementations could violate it. This is a boundary assumption to retain
   in any later test/review, not an independent finding against the production
   adapter path.

## Source behavior that did match the narrow assignment

Static review found the intended bounded creator/cohort reservation before the
list call, worker-batch launch before readiness waits, exact forward/reverse
lease membership checks, and cleanup/cancel/wait work outside `m.mu`. The
manager's unreadable-count path is backed by the adapter guarantee described
above. These points do not cure the terminal or authorization findings and do
not establish runtime behavior.

## Material differences and evidence limits

The implementation changes only the two authorized source files, and the ten
listed fixtures/helpers retain their expected identities. The source result's
frozen-input inventory nevertheless omits the retained-image fixture hash. More
substantively, the implementation/result do not fulfill revision 6's accepted
terminal stop/drain and repeated watcher/present-context checks. The assignment
itself incorrectly ruled out the private watcher identity required by the
governing contract; resolve that instruction conflict before a new source pass.
No compilation, formatting, test, race, runtime, or GREEN claim is made here.

Repository context was retrieved with Graft before targeted source reads. The
reported retrieval savings were approximately 184,393 tokens across the
recorded Graft queries; this is context-retrieval savings only, not verification
evidence.
