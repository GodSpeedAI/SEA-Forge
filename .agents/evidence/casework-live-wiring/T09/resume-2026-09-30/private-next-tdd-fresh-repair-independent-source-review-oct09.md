# Independent source review: fresh private Next TDD fixture repair

## Verdict

**REJECT source readiness pending a narrow fresh fixture repair.** The current
source is bounded to the authorized TDD scaffold and the repaired fixtures
substantially address the seven prior findings. Three proof gaps remain in
the recovery and sequencing oracles, the terminal-after-worker proof is not
deterministic, and a read-only format probe records one alignment correction.
Do not run the expected-RED gate on this packet.

This is a source-preparation rejection only. I ran no compiler, tests, race
suite, formatter, or gate. Nothing here claims lifecycle implementation,
behavioral RED, runtime results, Next completion, or T09 settlement.

## Source identities and scope

The frozen fresh builder packet is
`private-next-tdd-fresh-repair-builder-oct09.md`; its SHA-256 is
`a3e5d1fb8c82cf80d4bd9b602242f92d8858299e2dafb1b87ed55004dcc051f6`.
The current source identities match its table and the three unchanged
scaffold identities in the original rejection:

| File | SHA-256 | Review |
|---|---|---|
| `run_observation_manager.go` | `b45dc39bc7466160cec9bc441d2635d577ceae69f64f165f19dbca7f7605737d` | unchanged scaffold fields |
| `run_observation_poller_worker.go` | `2e5ef4ee58b1c821b6b5a9fdee83a2bce38f504dae3001f08e2b73306f350766` | unchanged notifier helper stubs |
| `run_observation_next.go` | `f362b461744b7b474f71a6d91c01f01eba76752c4c82761af5fa06ddb3db4015` | private aggregate/types and unavailable stub only |
| `run_observation_next_test.go` | `72c379991b6256ac38c77602e33127c9ac7a8671657a89d97406eee7e14b9620` | fixture source under review |

The read-only Git diff for the two tracked scaffold files contains only the
authorized lease field declarations and inert notifier-target helper stubs.
The new Next file contains the approved typed projector seam and an
unavailable stub, not lifecycle behavior. No public API, dependency, manager
algorithm, worker publication implementation, or pure delta helper changed.
The fresh repair packet says only the test file changed after the rejected
source. The old test file was untracked and its frozen SHA is recorded, but no
original test preimage or patch is present in this checkout; I therefore do
not claim an assertion-by-assertion byte diff against that old file. The
accepted manager/failure/delta tests are outside this change and the tracked
diff leaves them untouched.

## Repaired findings

1. **Static defect locations:** all four list callback sites now return both
   result and error; every cohort membership assertion uses comma-ok form.
   This is source inspection only, not a compiler claim.

2. **Hydration pruning:** the fixture now uses eight runs and 1,024 long-ID
   frames per run, checks aggregate DTO pruning, and expects terminal Next to
   return nonnil empty frame slices. This addresses the prior under-cap input
   and replay oracle. However, it does not wait for/assert `workerDone` before
   Next. `Prepare` waits on `entry.ready`, which the terminal worker closes
   before returning; its deferred `workerDone` close can occur later. The
   required “deliverable after workerDone” behavior is therefore not
   deterministically established. Wait for the actual worker JOIN before
   calling Next.

3. **All-or-nothing projector failure:** the two-run fixture obtains real H2
   publications beyond both ordinal-1 baselines, proves a genuinely advancing
   first candidate before injecting the second projection error, compares the
   complete watermark maps, and waits for terminal drain plus actual worker
   JOIN/removal. The approved projector-failure addendum classifies this as
   terminal; this test need not manufacture H3 recovery or hold an H3 read.

4. **Initial read failure:** the fixture uses a real accepted list and
   attached initial read, holds that read, and checks no watermark, no retained
   lease/entry reference, removal, and actual `workerDone` after Prepare's
   unavailable wrapper. This follows the approved correction and does not
   claim same-entry recovery after detachment.

5. **Gap ordering:** actual workers publish H2 with `middle` and H3 omitting
   that ledgered frame. The fixture asserts two nonempty exact gaps and runs
   sorted by run-ID bytes. This closes the empty-only oracle defect.

6. **Barrier/notifier cleanup:** projector/read/auth barriers use once-backed
   releases registered with `t.Cleanup`; the two-run helper has once-backed
   `releaseAll`, which manager cleanup calls to unblock its held worker reads.
   In Go's LIFO cleanup order the later manager cleanup runs before the
   helper's individual gate cleanups, and explicitly invokes `releaseAll`.
   The notifier release is scheduled once-backed immediately after real target
   registration. I found no remaining stranded barrier in the reviewed file.

7. **Scope and watermark integrity:** tests read actual Prepare/worker paths;
   fixtures do not write watermarks or retained state. The notifier ownership
   test calls the authorized real registration/release helper seam. Its
   separate claim is ownership accounting, while the capture fixture supplies
   real publication/coalescing evidence.

## Blocking findings

1. **Initial authorization ordering is not observable.**
   `TestRunObservationNextBlockedAuthCallbackIsOwnedByDetach` blocks the
   callback and proves `nextInFlight` was registered before detach. It does
   not check that the seeded wake remains present while that callback is
   blocked, so it cannot establish that authorization happened before waiting
   on/consuming the wake. Add the narrow assertion that the successful
   Prepare-seeded wake is still unconsumed at the blocked first callback.
   `TestRunObservationNextRechecksAuthorizationAfterWake` usefully proves two
   marked Next authorization calls, a consumed wake, and no projector call
   when the second fails. The cursor-mutation barrier checks final disclosure.
   Per root's source-phase boundary, the hidden capture ordering itself will be
   verified in implementation source, not demanded from this fixture seam.

2. **Transient-read recovery can lose its failure marker to a later read.**
   `TestRunObservationNextUnavailableListAndFailedProjectionDoNotAdvanceWatermarks`
   signals `thirdRead` and immediately returns fitting data. It does not hold
   that read until the failed Next has observed the unavailable marker. A
   scheduler delay can let the one-second poll interval publish the fitting
   state first, so the purported failure observation is timing-dependent.
   Before releasing a controlled later fitting read, assert the failed Next
   returned zero aggregate/unavailable, all prior watermarks remain unchanged,
   and the same poller reference and cohort/entry capacity remain attached.

3. **Nonterminal-retention recovery has the same overwrite race.**
   In `TestRunObservationNextNonterminalRetentionRecoversAndTerminalRetentionStops`,
   the third read likewise signals and immediately returns the fitting state;
   no gate prevents it from replacing the retention-unavailable marker before
   the failed Next observes it. Add a once-backed third-read gate, hold it
   through the zero-result/no-watermark and same-reference/capacity assertions,
   then release it to prove recovery from a real fitting worker publication.
   The terminal retention subcase remains a distinct drain claim.

4. **Canonical format correction remains pending.** The archived read-only
   probe `next-fixture-format-probe01-stdout-oct09.raw.json` decodes to a
   single gofmt alignment diff at `run_observation_next_test.go:87–96`
   (`runObservationNextControlledRun` fields). Apply only that formatting
   correction in the fresh repair; no formatter write or compiler run was
   performed by this critic.

## Required next step and limits

A fresh builder should make only the bounded fixture changes above, preserve
all other reviewed assertions, use once-backed cleanup for new gates, apply
the exact alignment-only format correction, and freeze new source hashes.
Then request another independent source review. Do not compile or run expected
RED until that review accepts source readiness and root grants the compiler
token. The old untracked test preimage is not available here, so the next
review should assess the new diff if the builder can recover it; no old source
or assertion is reconstructed from memory. The frozen original rejection and
builder identities remain unchanged. Graft retrieval saved about 90,555
tokens this turn across three calls (about $0.06).
