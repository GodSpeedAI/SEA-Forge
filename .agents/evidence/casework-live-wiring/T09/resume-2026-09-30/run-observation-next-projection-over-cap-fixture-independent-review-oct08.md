# Private Next over-cap fixture independent review

Date: 2026-10-08  
Verdict: **APPROVE SOURCE-READY for the additive over-cap fixture only.**
This approves no algorithm guard, runtime behavior, lifecycle, Next
integration, public wiring, or T09 completion. Root must run the separately
authorized actual RED before releasing the cap guard.

## Authority and identities

Read the complete repair assignment
(`run-observation-next-projection-over-cap-fixture-repair-assignment-oct08.md`),
complete repair result
(`run-observation-next-projection-over-cap-fixture-repair-result-oct08.md`),
the original projection grant (`run-observation-next-projection-assignment-oct08.md`,
SHA-256 `c6bd03f372140f0542f92c3c15a7e6b3f2471d311ef9bd705f0a2acc3c832c9d`),
and its malformed-input clarification
(`run-observation-next-projection-root-clarification-oct08.md`, SHA-256
`1a3685eb42c287c54ecbe09830827fbe6edce85aa96349b0f9f724269c526fee`).
The prior algorithm rejection is
`run-observation-next-projection-algorithm-source-review-reject-oct08.md`.

Current identities:

* `run_observation_delta_test.go`: 15,232 bytes, SHA-256
  `66e368ffdea1839aca74a6c242cb947f6d97fca45a2e228574ec4d1c3adeedbc`.
* Frozen `run_observation_delta.go`: 5,541 bytes, SHA-256
  `3c81aa06b4f2062e0d3af314635c4c141c9b407d6cee32afe579a03a391cbbb4`.
* Frozen `run_observation_retained_version.go`: 10,456 bytes, SHA-256
  `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`.

The test repair result reports a 13,953-byte, SHA-256
`af862f3bd18e866db8ccb8c84ef784a80d8a3d0d49e5dc8f47f3969d78a77cef`
preimage measured before edit, but not archived then. I independently
reconstructed that preimage from the current file by removing only the new
`strconv` import and the new test function; its byte count and SHA-256 match
exactly. This confirms the prior test content was preserved. The algorithm
file's identity matches the rejected implementation and remains frozen.

## Fixture oracle

The new test `TestRunObservationDeltaRejectsCapturedFrameWindowAboveRetentionCap`
(`run_observation_delta_test.go:294-321`) sets `count` to
`runObservationRetainedFrameLimit + 1` (`:296`), creates a current retained
state with valid existing key/time/frame helpers (`:295-299,309`), and sets
both `TotalFrameCount` and `HighestOrdinal` to that count (`:300-301`). It
builds 1,025 distinct exact IDs with matching first ordinals 1 through 1,025
in both the retained frame window and copied ledger (`:302-311`). Thus total
and ledger/frame ordinal structure are coherent; the sole impossible state
property is the retained window exceeding the 1,024-frame cap
(`run_observation_retained_version.go:14-16`). The accepted candidate
constructor rejects that cap at `run_observation_retained_version.go:82-84`.

The oracle requires typed unavailable and zero delta, nil gap, and zero
candidate watermark (`run_observation_delta_test.go:313-319`). It directly
targets the prior finding without adding an algorithm guard or changing
existing fixtures. This satisfies the bounded test-first repair assignment.

The repair result cites the constructor cap at lines 79-81; those lines are
the key-mismatch check in the current source. The correct cap citation is
lines 82-84. This is a citation correction only; the source and the finding
are otherwise accurately identified.

## Verification boundary

No tests, compiler, formatter, scanner, gates, build, or Git operation ran.
The expected RED has not been performed for this new fixture. Root should run
the authorized focused RED first; only then may a fresh bounded builder add
the cap guard. This fixture review does not accept or release the algorithm.
