# Independent review: initial-unavailable fixture correction

## Verdict

**APPROVE this narrow fixture correction.** The requested same-entry recovery
case after a failed initial trace read is unreachable under the established
initializer lifecycle. The correction preserves that lifecycle and changes the
fixture claim, not production behavior, public contract, or architecture. It
does not reverse the broader static source rejection or clear any other finding.

## Source and evidence checked

I read the full private Next assignment, the full rejected independent source
review, the full builder result, and the new initial-unavailable correction. I
also checked the actual initializer and Prepare source paths plus the complete
existing regression test named by the correction. The assignment's shortened
reference `next-integration-root-decisions-oct08.md` resolves to the existing
`run-observation-next-integration-root-decisions-oct08.md`; I reviewed that
actual policy document and do not treat the shortened name as missing evidence.

The source supports the correction:

* The poller calls `finishPollerRead` with `initial=true` for the first read
  (`run_observation_poller_worker.go:121-125`). On `readErr`, that branch sets
  `runObservationInitializerReadUnavailable`, keeps `state` nil, and sets
  `continuePolling=false` (`run_observation_poller_worker.go:284-304`). The
  worker then returns, and its deferred `close(entry.workerDone)` records the
  actual worker completion (`run_observation_poller_worker.go:56-60`).
* Prepare only seeds a watermark when the entry has a current retained state
  with `retainedCurrent` (`run_observation_manager.go:542-558`). With the
  initial error's nil state, it counts the selected run unavailable, releases
  the lease's poller reference, and calls the selected-release join path
  (`run_observation_manager.go:559-564`). The release path removes the
  reference and, when no eligible watcher remains, marks the entry draining
  and cancels it (`run_observation_manager.go:257-282`). The selected-release
  owner waits for `workerDone` before deleting the final reference and entry
  (`run_observation_manager.go:294-309`).
* `TestRunObservationManagerFailureSharedFirstReadErrorIsSuccessfulA`
  exercises the real first read with two Prepare callers. It asserts successful
  unavailable wrappers and distinct leases, zero refs in the entry and both
  lease maps, actual worker JOIN, capacity removal only after JOIN, and exactly
  one trace read (`run_observation_manager_failure_test.go:570-649`).

Thus there is no retained current value, no attached poller ref, and no worker
left on that entry for a later fitting publication to recover. The successful
empty cohort lease may remain for its normal lifecycle, but it cannot recover
that detached poller by inventing a baseline, relisting, or reacquiring state.

## Material difference from the rejected finding

The rejected review's finding 4 requested a test for “accepted list and
attachment with no `retainedCurrent`, followed by a later fitting value.” Its
builder note said such a lease should have zero prior and deliver first genuine
frames. Source evidence shows that premise does not hold for the initial read
error: Prepare releases that nil-current attachment and joins/removes the
worker entry before returning the successful unavailable wrapper.

The correction therefore supersedes only that requested setup and recovery
claim. Fresh fixtures should instead prove that this actual initial failure
leaves no watermark baseline and no retained poller refs after the initial
Prepare cleanup. They must not claim later publication/recovery for the
detached entry. Recovery remains required for later transient read-unavailable
or nonterminal retention-unavailable markers only after an accepted
`retainedCurrent` attachment, using a real later fitting worker publication and
unchanged prior watermarks. Complete-list/initial-read failure remains distinct
from initial list-unavailable. The prior projection-failure addendum and all
other rejection findings remain binding.

## Policy consistency and limits

This is consistent with the original assignment's preserved-assertions and
existing-lifecycle constraint and with the actual root integration policy in
`run-observation-next-integration-root-decisions-oct08.md`: read/retention
recovery is tied to a later publication while references remain attached; it
does not authorize recovery after initial Prepare has released the failed
initializer. The broad assignment wording about recoverable read-unavailable
is clarified here for the initial-read phase. No new public behavior, API,
policy precedence, source continuity, or architecture boundary is introduced.

The correction authorizes only the fixture expectation change. A new source
review must still verify the repaired fixtures against the complete assignment
and all addenda before root's behavioral-RED gate. I ran no compiler, tests,
formatter, or runtime gate.
