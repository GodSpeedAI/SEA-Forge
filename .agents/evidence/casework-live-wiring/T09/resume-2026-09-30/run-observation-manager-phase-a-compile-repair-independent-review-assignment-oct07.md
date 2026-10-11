# Phase A one-binding compile-repair review assignment — 2026-10-07

The complete bounded repair assignment is
`run-observation-manager-phase-a-compile-repair-assignment-oct07.md` (SHA-256
`b5689c684013ab24a76289383f9a008a6efc364c08ef1f2db4cce12cf03da0e0`). It
preserves the full Phase A release, previous source approval/rejection, and
actual focused compiler failure. The current repair result is
`run-observation-manager-phase-a-compile-repair-result-oct07.md` (SHA-256
`7cb1bf55dc482eed7d9351be6bf9928ec23cfaa19b11bff51dc2e5b02c2a9ae1`).

## Complete review instruction received

> URGENT independent one-binding repair review SOURCEONLY, no compiler grant.
> Read the entire compile-repair instruction and result, the prior compiler
> result and Phase A approval, and the exact preimage JSON `source` field. The
> repair authorization was to remove the unused `caller` binding at line 323
> in `TestRunObservationManagerFailureGlobalStopReleasesDistinctPendingLeases`,
> while preserving the required caller binding in
> `TestRunObservationManagerFailureStopJoinsHeldActualRead`. Independently
> confirm the named function, hunk, and actual bindings. If the single hunk
> changed the wrong occurrence and leaves either compile error, reject; report
> the material deviation and any false result-scope claim. Do not run Go,
> format, change source, or mutate Git. Archive the full review instruction,
> actual preimage/result diff, and verdict as new immutable evidence.

## Review constraints

- Source-only, no compiler/test/formatter/scanner or Git mutation.
- Verify the exact preimage by writing the JSON `source` string to the hash
  stream without `jq -r`'s extra newline, then diff it against the actual final
  fixture.
- Preserve all prior assignments, reviews, preimage, and result. Do not repair
  source in this review.
- State whether the expected-RED precondition is met; do not claim any compile
  result beyond the already captured failure.
