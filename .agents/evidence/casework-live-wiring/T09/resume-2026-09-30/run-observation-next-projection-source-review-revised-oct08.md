# Private Next projection revised source review

Date: 2026-10-08  
Verdict: **APPROVE SOURCE-READY for the bounded types/stub and TDD tests only.**
This review does not authorize the projection algorithm, actual RED execution,
manager capture integration, lifecycle/Next implementation, public wiring,
runtime, or T09 completion. Root retains those release decisions.

## Authority read

This review applies the full original assignment and additive root
clarification, the governing proposal revision 6, its addendum, retention
initializer correction revision 3, the prior independent review and this
repair's full assignment/result. The prior finding was that six fixtures used
an invalid settlement value, `settled`; the contract vocabulary is
`unsettled|accepted|rejected|escalated` (`internal/contract/contract.go:436`).

## Current identities

* `run_observation_delta_test.go`: SHA-256
  `af862f3bd18e866db8ccb8c84ef784a80d8a3d0d49e5dc8f47f3969d78a77cef`.
  This matches the repair result's stated final identity.
* Frozen `run_observation_delta.go`: SHA-256
  `aff93e3a1b01aedd8075be1f59ff52cebb4dc36de59eaf4140cb939cdf148973`.
  This matches the original source review and repair result.
* All eleven frozen observation source/test files match the exact identities
  recorded by the first builder; hashes were recomputed during this review.

The six repaired test assignments/assertions now use `accepted` (test lines
29, 51, 138, 154, 184, 297). No `settled` literal remains. This is an allowed
settlement standing. Inspection against the exact repair grant and prior
candidate confirms the authorized change is limited to those six literal
substitutions; the repair result likewise records the exact preimage/final
hashes and six-literal scope. The stub's identity is unchanged.

## Requirements and coverage

The four private types represent the captured window, per-run delta, observed-ID
gap, and scalar watermark using the required contract enum types. The pure
seam takes only the captured retained state, copied exact-ID ledger, and prior
watermark. Its implementation remains a generic typed-unavailable error with
zero outputs; it contains no projection algorithm.

The seven focused test functions cover P/H frame and gap boundaries, exact
opaque-ID gap output and byte ordering, captured source order and window/time/
identity, ledger IDs above captured H, standing/window-only changes in both
directions, opaque ID reappearance/payload rewrite, optional-pointer ownership
and input immutability, nil/impossible/unavailable captures and ordinal
disagreement, and empty successful terminal capture. Clarified reported-gap
flags are asserted true. Error cases assert typed unavailable and zero outputs.
This satisfies the authorized TDD source-preparation scope.

The builder made no claim that tests compile, that RED occurred, or that the
algorithm works. No tests, compiler, formatter, scanner, gates, build, or Git
command ran during this review. The expected-RED step remains a separate root
action after this source approval; this review grants no runtime or algorithm
release.
