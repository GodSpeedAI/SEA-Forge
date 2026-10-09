# Private Next TDD repair2 independent source review

Date: 2026-10-09  
Verdict: **REJECT source readiness pending one bounded race repair**

## Scope and identities

This is a source-only review under the fresh repair2 grant recorded in
`private-next-tdd-repair2-builder-oct09.md` (SHA-256
`1d1b2d49d1868bf29039e6549bf2bda3bff792b3e3320864d13177eea8145c83`). I read
the full original assignment and its sequencing, notifier, projection-failure,
and approved initial-unavailable addenda, the accepted prior source rejection
`private-next-tdd-fresh-repair-independent-source-review-oct09.md` (SHA-256
`b569318089fdcc18c29e4070b31edb5511a614a0736bf2984458d4c10f1eff4d`), and
the complete repair2 test source.

The source reviewed is
`apps/godspeed-casework-go/internal/server/run_observation_next_test.go`,
SHA-256 `1f23ab0e55e30fe95a38fc67733a5d961209dd3874f82ce4966e444ccc8c7112`.
The comparison base is the root-saved decoded repair1 snapshot
`private-next-current-repair1-72c37999-source-snapshot-oct09.json`, artifact
SHA-256 `c20c046de680438d732c2f69803560c9b28170bea9796092f0aa9818e13ecb54`,
whose embedded source SHA is `72c379991b6256ac38c77602e33127c9ac7a8671657a89d97406eee7e14b9620`.
The original untracked `c8e825...` preimage is unavailable, so this review
makes no preservation claim against that preimage. The builder packet’s exact
diff summary against verified repair1 is consistent with the inspected source.

## Findings resolved

The five repair1 findings are addressed in the reviewed source:

* `TestRunObservationNextBlockedAuthCallbackIsOwnedByDetach` checks the seeded
  wake remains present with capacity one while the first authorization
  callback is blocked.
* The read and nonterminal retention recovery paths hold the third fitting
  read behind a once-backed gate. Before release they assert the typed
  unavailable result, zero aggregate, unchanged watermarks, retained marker,
  exact poller references, and counted capacity.
* Terminal hydration waits for all eight actual `workerDone` channels before
  calling `Next`, then checks the same retained current pointers and counted
  references.
* The controlled-run field alignment matches the recorded formatting diff.

The rest of the reviewed fixtures still provide the requested substantive
oracles: actual multi-run candidate discard and worker JOIN after projector
failure; a real two-read gap with nonempty, sorted gap records; initial
unavailable cleanup with no watermark/ref and a joined worker; auth sequencing
and no-projection assertion; and once-backed cleanup with release-before-drain
ordering. The atomicity fixture is not required to hold an H3 read. Existing
accepted pre-Next source was not changed by this repair. These observations
address the prior findings and do not motivate additional proof requirements.

## Blocking finding: ungated second reads can race initial `Next`

The transient read recovery test, beginning at
`TestRunObservationNextUnavailableListAndFailedProjectionDoNotAdvanceWatermarks`,
uses an ungated case-2 trace read: it signals `secondRead` and immediately
returns the transient error. The retention recovery subtest does the same for
its oversized case-2 snapshot. The tests then await the failed-candidate
`Next`, assert unchanged watermarks and ownership, and later launch recovery.
The added third-read gate protects the unavailable marker from the *next*
fitting read, but it does not control when the second read publishes.

In each fixture, the initial `Next` is called immediately after `Prepare`,
before the test captures its baseline. If the test goroutine is descheduled
long enough for the poller interval to elapse, the ungated second read may
publish its error/oversized candidate before that initial `Next` returns.
Then the fixture can observe or accept a different initial state than the
specified seeded no-replay result, or the second `Next` may consume the wrong
candidate. This leaves a timing assumption in both required recovery proofs.

Required bounded repair: gate the second read in both fixtures as well as the
third read. Use once-backed release functions for both gates and pass a
combined release callback to manager cleanup. Capture and assert successful
initial `Next`, baseline watermarks, and initial exact ownership first; then
release the second error/oversized read. Preserve the existing failed-result,
marker/ownership/capacity checks and third-gated recovery sequence. This adds
no new behavioral policy and does not alter accepted assertions outside these
two recovery fixtures.

## Limits

This rejection concerns only deterministic source readiness. Root-provided
format-probe-02 evidence reports an empty `gofmt -d` result across the six
archived captures; I did not run it. No compiler, tests, runtime, gate, or
behavioral RED was run or claimed here. The other three source identities and
the source-only/fixture-only boundary are as recorded in the builder packet.
No source, status, debt, or Git state was changed; this review is the sole new
artifact.
