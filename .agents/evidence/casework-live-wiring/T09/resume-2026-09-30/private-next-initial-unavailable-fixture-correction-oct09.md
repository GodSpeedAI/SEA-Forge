# Initial unavailable fixture correction

Root decision, 2026-10-09. Preserve the accepted initializer contract; do not
implement a new initial-read recovery policy to satisfy finding 4 of the static
Next fixture rejection.

The original assignment requires preserving existing assertions and attachment
semantics. Actual initial read failure publishes no retained current value and
stops the worker (`run_observation_poller_worker.go`, finishPollerRead initial
readErr branch). Prepare releases a selected entry without retainedCurrent
(`run_observation_manager.go`, validatedStates selection). The existing
TestRunObservationManagerFailureSharedFirstReadErrorIsSuccessfulA explicitly
asserts successful unavailable initial wrappers, zero owner/waiter/entry refs,
actual worker JOIN and capacity removal. Root read these source branches and
the complete existing test after independent read-only recon.

Consequently, the rejected review's requested accepted initial-read-unavailable
attachment followed by same-lease worker recovery is not reachable. The earlier
zero-baseline note was conditional on the actual attachment path retaining refs;
that condition is false for this initializer error. Finding 4 is superseded in
this narrow respect, with the rejected review preserved unchanged.

Fresh fixture repair must preserve the established initial-unavailable cleanup
test and add a bounded actual Prepare proof of absent initial watermark and no
retained poller refs after initial failure. Do not claim a later publication or
recovery for that detached entry. Initial list-unavailable remains distinct from
an accepted list whose selected read failed. No extra poller selection, relist,
fake current state or watermark mutation is authorized.

Recoverability coverage applies to actual later transient read or nonterminal
retention markers after an accepted retainedCurrent attachment. Those require
real later fitting worker publications, unchanged prior watermarks on failure,
and the same retained references/capacity. All other rejection findings and the
projection-failure addendum remain binding. This correction changes test
expectations for an unreachable setup, not production initializer behavior or
the public contract. Independent semantic review is required before release.
