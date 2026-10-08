# Independent watcher/terminal TDD source-preparation review

Date: 2026-10-08

## Review assignment and limits

Review the full watcher/terminal TDD source-preparation assignment, its test-2
clarification, the source result, both exact source preimages, and all new test
source. Compare the actual changes with the original Phase B algorithm grant,
its independent rejection, the watcher/terminal proposal and matrix approval,
root clarifications, and revision 6 lifecycle/authorization requirements.
Verify exact diffs and current identities. Assess static callsite compatibility,
fixture reachability, assertions, cancellation/JOIN order, cleanup, terminal
cap proof, and shared-watcher ownership. Report every material deviation. This
is SOURCEONLY: no compilation, tests, formatting, scanning, build, Graft build,
Git operation, or runtime claim is authorized. Approval, if any, is limited to
source readiness for a later separately granted focused RED gate.

## Inputs read

- Full TDD source-preparation assignment:
  `run-observation-watcher-terminal-tdd-source-assignment-oct08.md`,
  SHA-256 `bfcfa5cd59f53bf5d81de7a349eac669e8899a8b842c863cb4cdb0f580a5ec82`.
- Recurring terminal outer-cap clarification:
  `run-observation-watcher-terminal-tdd-test2-root-clarification-oct08.md`,
  SHA-256 `2f9b7471518b44a24cf49484c269e9387eeeb55c577f95a83938b1ebdcd7efb4`.
- Cleanup clarification:
  `run-observation-watcher-terminal-tdd-cleanup-root-clarification-oct08.md`,
  SHA-256 `7fe1dfde618e2740ab16911be13ae5fa6a2362cc82d99f0c7d94391ba2892d9d`.
- Actual source result:
  `run-observation-watcher-terminal-tdd-source-result-oct08.md`,
  SHA-256 `b17053846faa8b5540ede3665c47070a3832da216d7bf2103bd9594e260d8537`.
- Original algorithm assignment:
  `run-observation-phase-b-algorithm-source-assignment-oct08.md`,
  SHA-256 `e5fc9e0aa9a8549aa75bcf78040b7012e79e8a882a176965dc80ce6eace8ef42`.
- Original independent algorithm rejection:
  `run-observation-phase-b-algorithm-independent-review-oct08.md`,
  SHA-256 `ecf2e06f34a57257cc99480dc17a41b40cdaff0a34248ceeb7c940c91473b644`.
- Watcher/terminal repair assignment/proposal/root clarification:
  `run-observation-phase-b-watcher-terminal-repair-assignment-oct08.md`
  (`7ffdd48ccbfd69d4ff0e8bf5d7812405b7385651c5b5396e8b4c30b4e8deb852`),
  `run-observation-phase-b-watcher-terminal-repair-proposal-oct08.md`
  (`de8c1647153e06a5fc492671d14c36d3e562f1cd1f079086b62546304e90f0ad`),
  and `run-observation-phase-b-watcher-terminal-root-clarification-oct08.md`
  (`92c0a5bcbdc309c1f0e09c3ac4bf61dc350aee685985a5a824ee2dff72cabba5`).
- Matrix assignment/supplement and independent DOC approval:
  `run-observation-phase-b-watcher-terminal-matrix-supplement-assignment-oct08.md`
  (`f31d15ef7f611322cc4941f4b1bb6a29660edb5b8ea0845c04345cf4c1bfedf5`),
  `run-observation-phase-b-watcher-terminal-matrix-supplement-oct08.md`
  (`fe1e9e0502bf861b4153887734007e6db1486aea9a731d308e7abc58d0e18116`),
  and `run-observation-phase-b-watcher-terminal-matrix-independent-review-oct08.md`
  (`b5acb054253562fcfa81f7e901dcf9442004639bfbc5a68839bd64d588fc3f0e`).
- Revision 6 and referenced lifecycle contracts, including caller identity and
  as-of cursor, per-watcher checks, shared references, terminal ordering,
  immutable image and actual worker join.

## Source identity and exact diff verification

Both archived JSON preimages were read and decoded. Their declared byte lengths
and SHA-256 values match the embedded source strings: manager preimage
`d7be1d7a724ed65575ccaaf35cdf1aa5af133dbdd4952d592699b7eb443b6350`, 26,384
bytes; failure-fixture preimage
`ef587b0bd64fe42d5fb56369f620f969610a5bac3d04cd7578911472b1a4893c`, 46,429
bytes. A read-only unified diff confirms the manager changes consist of the
two authorized lease fields, required `beginPrepareOperation` parameters and
storage, and the one production call passing the already resolved caller and
guard cursor. The failure-fixture diff changes exactly the four authorized
manual begin setups: each derives caller from its existing request identity,
obtains the cursor through `manager.guard`, handles errors, and passes both
required values. No assertion body was changed in that fixture.

Current hashes verified from the actual source bytes:

| File | Current SHA-256 |
|---|---|
| `run_observation_manager.go` | `47c95f3ba90abb4355f02d626e664a49027f602e9ac47771c40eda0ba8f22780` |
| `run_observation_poller_worker.go` | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` |
| `run_observation_manager_failure_test.go` | `ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| New `run_observation_manager_authority_terminal_test.go` | `4934e9c5e8a5506f219f2c873856452e97f4c971f4c7a233dbc480fa79df7367` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

The eight pre-existing frozen files retain their approved identities. The new
fixture has seven focused top-level groups; its final-handoff group has all
four requested auth/cursor × empty/error subcases. No source outside the three
authorized paths changed.

## Verdict

**REJECT source readiness pending bounded repairs to the new fixture only.**
The metadata/preimage changes and most test boundaries follow the grant. Three
material proof gaps remain: the recurring outer-cap case does not establish
that the canonical failure is caused by size; the shared-survivor test only
invalidates the creator after a read has already begun; and the failed-Prepare
groups do not inspect owned cleanup before global test teardown.

## Required findings

1. **The recurring outer-cap case does not prove the size cause.** The helper
   and group 2 accept `outerErr != nil` as sufficient evidence (`new test
   file:262–297, 349–355`). The existing canonical encoder intentionally
   returns `nil` bytes and `errRunObservationPollerImageUnavailable` for an
   oversized wrapper (`run_observation_poller_image.go:90–95`). A non-nil
   canonical error therefore cannot supply the byte length; as written, the
   test would also treat an unrelated encoding/key error as the required
   over-cap condition. The test-2 clarification requires a candidate after a
   fitting initial state whose inner image fits but complete outer image
   exceeds the fixed cap. Keep the real recurring port call and candidate.
   Independently `json.Marshal` the existing
   `runObservationPollerImageWithCurrent` type with the actual schema version,
   running phase, accepted initializer result, validated matching key, and the
   exact canonical inner bytes. Require that raw marshal to succeed and its
   length to exceed `runObservationRetainedImageMaxBytes`; also require the
   canonical bounded encoder to return nil bytes and its exact unavailable
   sentinel. Do not hand-write an alternate schema or lower the cap.

2. **The shared-survivor test does not prove the surviving watcher can
   authorize a read after its creator becomes invalid.** Group 6 starts the
   port read at line 521 and only then attaches the survivor and revokes the
   owner (`:520–538`). It proves the worker can preserve a valid survivor for
   publication after a read already authorized by the then-valid creator. It
   does not prove the root clarification's separate rule: an invalid/canceled
   creator alone cannot authorize a shared read, while a different eligible
   watcher can. Group 3 revokes before read but has no survivor. Add a bounded
   subcase inside group 6 that blocks the worker at the existing injected
   authorization/Current dependency before the actual port call; attach a valid
   second lease to the same initializing entry; invalidate/cancel the creator;
   then release the dependency barrier. Require the creator to fail closed and
   the real read to proceed only after the valid survivor's current
   auth/cursor/parent checks pass. Preserve the current during-read scenario.
   The seam is the existing authorization dependency, not a production hook;
   do not manufacture lifecycle state or authorize the worker from the invalid
   creator's cached result.

3. **Failed-Prepare cleanup is not asserted before test cleanup.** The cleanup
   clarification requires groups 3–5 to observe both `cohorts` and `pollers`
   empty immediately after failed Prepare returns and before `t.Cleanup`;
   groups 4–5 must additionally observe the captured actual entry's
   `workerDone` already closed. Both group-6 invalidation cases must prove the
   invalid lease is absent from `cohorts` and forward/reverse entry membership,
   while the authorized survivor's cohort, exact entry, and valid retained
   state remain registered. Current groups 3–5 assert the error/zero DTO/no
   read or no publication, but not pre-teardown registry cleanup
   (`:394–498`). Group 6 checks the invalid reverse ref and survivor membership
   but not cohort membership, and currently has no pre-read case (`:500–556`).
   `cleanupRunObservationManager` runs later through `t.Cleanup` and can stop
   and drain a leaked registration before its own final checks; it is not proof
   of per-Prepare cleanup. Add only read-only mutex/channel assertions before
   teardown. Do not use global Stop or cleanup as the evidence for failed
   Prepare ownership.

The final-handoff table itself reaches all four real list-boundary cells,
mutates only synchronized injected dependencies, expects typed unavailable,
zero DTO/nil lease and zero trace calls, and checks cohort/poller cleanup before
teardown (`:558–615`). Reporting the cursor-invalid/empty-list cell as an
actual pass is correct and avoids fabricated RED. The fitting-terminal group
asserts exact final DTO fields, retained key/current state, owner/shared counts
1/0, real worker completion, stable trace-call count, refused later claim, and
valid refs (`:180–260`), consistent with root's no-worker-wait clarification.
Held callbacks use explicit channels and idempotent release; waits are bounded,
and `t.Cleanup` releases held calls before stop/join. The new source uses
existing test dependencies and current manager/worker boundaries without
sleep, private state mutation, cancellation spy, or production hook.

## Material scope deviations and limits

The receipt was written as
`run-observation-watcher-terminal-tdd-source-result-oct08.md` instead of the
assignment's requested
`run-observation-watcher-terminal-tdd-result-oct08.md`. This is a nonmaterial
evidence-name deviation: the actual source-result receipt is present and its
full hash is recorded above. Do not describe the requested filename as
present.

The source result claims only presence/identity and correctly makes no compile,
RED, or runtime claim. This review likewise makes no such claim. The four
fixture-only repairs above must be made and independently reviewed before any
focused RED execution grant.
