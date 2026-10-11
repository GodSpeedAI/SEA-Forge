# Private Next projection algorithm source review

Date: 2026-10-08  
Verdict: **REJECT — not SOURCE-READY.** This read-only review identifies a
fail-closed validation gap in the bounded pure assembler. It approves no
runtime, lifecycle, Next integration, public wiring, or T09 completion.

## Governing authority and candidate identity

Reviewed the complete original grant
(`run-observation-next-projection-assignment-oct08.md`, SHA-256
`c6bd03f372140f0542f92c3c15a7e6b3f2471d311ef9bd705f0a2acc3c832c9d`), its
complete malformed-input/gap clarification
(`run-observation-next-projection-root-clarification-oct08.md`,
`1a3685eb42c287c54ecbe09830827fbe6edce85aa96349b0f9f724269c526fee`), the
governing revision 6 proposal (`60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`),
revision 6 addendum (`9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`),
retention initializer correction revision 3
(`b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`), the
revised SOURCE-READY test review, recorded focused RED, and complete algorithm
result (`run-observation-next-projection-algorithm-result-oct08.md`, SHA-256
`c5cf4fe9b22ee0cf007abeb1d120ae6ece9e0ae114434bf9238c70bb131ebe79`).

Current implementation identity: `run_observation_delta.go` SHA-256
`3c81aa06b4f2062e0d3af314635c4c141c9b407d6cee32afe579a03a391cbbb4`.
Frozen test identity: `run_observation_delta_test.go` SHA-256
`af862f3bd18e866db8ccb8c84ef784a80d8a3d0d49e5dc8f47f3969d78a77cef`.
All eleven other frozen observation source/test hashes match the preparation
result. The recorded RED has exactly the six expected test failures caused by
the generic unavailable stub, exit code 1, and empty stderr; no compile/setup
error is present. No verification was run during this review.

## Finding

**P1 — Retained-frame cap is not validated at the pure projection boundary.**
The assembler rejects nil/non-current state, negative totals, totals below
retained length, and `P > H` (`run_observation_delta.go:53-56`). It does not
reject `len(state.Frames) > runObservationRetainedFrameLimit`. The actual cap is
1,024 (`run_observation_retained_version.go:12-14`), and candidate construction
rejects a source window over that cap (`run_observation_retained_version.go:79-81`).
The clarification expressly requires malformed captured inputs with
impossible total/retained counts to return an error and zero outputs.

Concrete counterexample: supply a `retainedCurrent` state with 1,025 unique
frames, `TotalFrameCount == 1,025`, `HighestOrdinal == 1,025`, and a copied
ledger mapping those frame IDs to the exact ordinals 1 through 1,025; use
`P == 0`. This satisfies every check currently present at `delta.go:53-104`
and returns a successful 1,025-frame delta. Such a retained window cannot be
produced by the approved candidate constructor, so it is malformed input to
this seam and must fail closed.

The frozen malformed-input table has cases for negative total, total below
retained length, prior above H, and ledger/frame inconsistencies
(`run_observation_delta_test.go:234-251`), but no retained-window-over-cap
case. The implementation's extra full-ledger ordinal validation is consistent
with accepted candidate construction, which assigns increasing first ordinals
and carries forward the full ledger (`run_observation_retained_version.go:97-151`);
it does not resolve this missing cap check.

## Required next step

Fresh bounded repair should add only the over-cap guard and a focused malformed
input test that asserts typed unavailable plus zero outputs. Preserve all
existing assertions and the current implementation preimage; run the separately
authorized expected RED before adding the guard. Then freeze the new result for
independent review. This review did not edit source/tests and did not run tests,
compiler, formatter, scanner, gates, build, or Git.
