# Next projection over-cap fixture repair — result

Date: 2026-10-08. This is a source/test fixture result only. It does not authorize or implement an algorithm guard and makes no test, runtime, Next integration, or T09 completion claim.

## Grant and source identities

The full original grant is archived in `run-observation-next-projection-over-cap-fixture-repair-assignment-oct08.md`. The governing unit assignment is `run-observation-next-projection-assignment-oct08.md` (SHA-256 `c6bd03f372140f0542f92c3c15a7e6b3f2471d311ef9bd705f0a2acc3c832c9d`); its malformed-input clarification is `run-observation-next-projection-root-clarification-oct08.md` (SHA-256 `1a3685eb42c287c54ecbe09830827fbe6edce85aa96349b0f9f724269c526fee`). The independent rejection is `run-observation-next-projection-algorithm-source-review-reject-oct08.md`; it identifies the missing 1,024-frame cap check and cites the retained candidate constructor's rejection condition at `run_observation_retained_version.go:79-81`.

Only `apps/godspeed-casework-go/internal/server/run_observation_delta_test.go` changed:

- Pre-edit identity measured before applying the patch: 13,953 bytes, SHA-256 `af862f3bd18e866db8ccb8c84ef784a80d8a3d0d49e5dc8f47f3969d78a77cef`.
- Post-edit identity: 15,232 bytes, SHA-256 `66e368ffdea1839aca74a6c242cb947f6d97fca45a2e228574ec4d1c3adeedbc`.
- The exact byte delta is the `strconv` import and one new test, `TestRunObservationDeltaRejectsCapturedFrameWindowAboveRetentionCap`. Removing those additions from the postimage reproduces the pre-edit byte count and SHA-256 exactly. The original preimage was not separately archived before the edit; this is a post-edit reconstruction verified against the measured pre-edit identity, not an earlier capture.

Frozen algorithm identity: `run_observation_delta.go`, 5,541 bytes, SHA-256 `3c81aa06b4f2062e0d3af314635c4c141c9b407d6cee32afe579a03a391cbbb4`. Frozen constructor identity: `run_observation_retained_version.go`, 10,456 bytes, SHA-256 `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`.

## New fixture oracle

The test constructs a `retainedCurrent` state using the existing valid key, timestamp, and frame helpers. It creates 1,025 unique opaque IDs and pairs each frame's `FirstOrdinal` with the same unique exact-ID ledger entry at ordinals 1 through 1,025. It sets `HighestOrdinal` and `TotalFrameCount` to 1,025, and uses the existing frame helper's valid command-finished enum and optional values. Thus the copied ledger and retained frames are internally coherent; the sole invalid property is exceeding the authoritative `runObservationRetainedFrameLimit` of 1,024.

The oracle requires a typed `apperr.KindUnavailable` and all-zero delta/watermark plus nil gap. This directly covers the missing malformed retained-window boundary from the critic. Existing assertions and test bodies were not edited or weakened. The expected focused RED has not been run or claimed. No guard was added to `delta.go`; an independent source review and root's separately authorized actual RED must precede any implementation change.

No compiler, test, formatter, scanner, build, Graft build, or Git operation was run for this repair.
