# Private Next TDD repair3 independent source review

Date: 2026-10-09  
Verdict: **APPROVE bounded fixture source readiness**

## Scope and identities

This review is limited to the fresh source grant in
`private-next-tdd-repair3-builder-oct09.md` (SHA-256
`b98a320cd27e58495ab110d4c77d8a1aa1a524e11ee8f727f027197d3356daa0`) and the
sole remaining requirement from accepted review
`private-next-tdd-repair2-independent-source-review-oct09.md` (SHA-256
`4754d97759c6140acb77b97bdab5e150c68f353b48317efaa769e357de79204d`). The
governing original assignment and sequencing, notifier, projection-failure,
and approved initial-unavailable addenda were previously read in full and
remain controlling.

The reviewed final test file
`apps/godspeed-casework-go/internal/server/run_observation_next_test.go` has
SHA-256 `36dab5455a24521a6715f5ca8f9597a2cad28a070f7c5f09df56598c28386780`.
Its comparison base is the root-saved repair2 snapshot
`private-next-current-repair2-1f23ab0e-source-snapshot-oct09.json` (artifact
SHA-256
`c4029f3b9f3b8c1f3b6d8ce886199915413950e011039a4320f3b13cdbef2dc6`), whose
embedded source identity is `1f23ab0e55e30fe95a38fc67733a5d961209dd3874f82ce4966e444ccc8c7112`.
The original untracked `c8e825...` preimage remains unavailable; this review
makes no preservation claim against it. The three scaffold files retain their
prior identities: manager `b45dc39bc7466160cec9bc441d2635d577ceae69f64f165f19dbca7f7605737d`,
poller worker `2e5ef4ee58b1c821b6b5a9fdee83a2bce38f504dae3001f08e2b73306f350766`,
and Next stub `f362b461744b7b474f71a6d91c01f01eba76752c4c82761af5fa06ddb3db4015`.

## Repair3 finding and direct diff review

I reconstructed the complete read-only unified diff from the decoded snapshot
payload to the final source. In the transient-read and nonterminal-retention
recovery fixtures it adds a once-backed second-read gate and release callback,
combines second- and third-gate releases in the manager cleanup callback, and
makes each second trace callback wait for its gate or worker-context
cancellation before returning the transient error or oversized snapshot. In
each fixture the existing initial `Next`, prior-watermark capture, and exact
initial ownership checks precede the call that releases the second gate. The
failed `Next` is started first and the test waits for the real second-read
signal before releasing it. Thus neither second candidate can publish before
the successful seeded result and baseline ownership state have been observed.

The existing third-read gates and all assertions around typed unavailable,
zero aggregate, unchanged watermarks, retained marker, exact references,
capacity, and fitting recovery remain present. The terminal retention branch
still tests drain behavior. This directly resolves the sole repair2 rejection.
The complete diff contains only these authorized gate declarations, callback
waits, combined cleanup callbacks, and release points; the three scaffold
identities above are unchanged. No other requirement was added.

The builder packet's inline diff excerpt is not a complete rendering of every
unchanged source context line, despite its “exact” label. I used the actual
decoded snapshot and final file for this review; that full reconstruction
supports the bounded diff summary above. Root reports its independent actual
format probe 03 returned zero with empty stdout/stderr and all six archived
captures byte-compared successfully. I did not run the format probe.

## Prior proof coverage and boundaries

The prior accepted repair2 oracles remain in the final source: blocked-auth
seeded-wake presence, actual worker JOIN before terminal hydration `Next`,
initial unavailable cleanup with no watermark/references and worker JOIN,
atomic candidate discard and worker JOIN, genuine nonempty sorted two-run
gaps, auth recheck/no-projection behavior, and once-backed cleanup. The
atomicity fixture has no H3 requirement. No claim is made here about hidden
capture timing that the fixture cannot observe; implementation-source review
owns that check. The accepted pre-Next test file scope and limits are as
recorded in the builder packet.

This is source-readiness approval only. No compiler, tests, runtime, gate,
formatter write, behavioral RED, implementation correctness, Next completion,
or T09 settlement was performed or established. Root owns the next compiler
and expected-RED decision. No source, status, debt, or Git state was changed by
this review.
