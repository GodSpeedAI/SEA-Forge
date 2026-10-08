# Private Next projection source review

Date: 2026-10-08  
Verdict: **REJECT — not SOURCE-READY**. This is a source-only review of the
test-first types/stub and tests. It grants no projection algorithm, lifecycle,
Next integration, public wiring, runtime, or T09 approval.

## Reviewed authority and identities

Read the full original assignment (`run-observation-next-projection-assignment-oct08.md`,
SHA-256 `c6bd03f372140f0542f92c3c15a7e6b3f2471d311ef9bd705f0a2acc3c832c9d`),
the full root clarification (`run-observation-next-projection-root-clarification-oct08.md`,
`1a3685eb42c287c54ecbe09830827fbe6edce85aa96349b0f9f724269c526fee`),
revision 6 (`60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`),
its addendum (`9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`),
initializer retention correction revision 3 (`b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`),
the complete builder result, and the retained-state helper and tests.

Reviewed candidate identities:

* `run_observation_delta.go`: 1,651 bytes, SHA-256 `aff93e3a1b01aedd8075be1f59ff52cebb4dc36de59eaf4140cb939cdf148973`.
* `run_observation_delta_test.go`: 13,947 bytes, SHA-256 `6e6e3c0e8388104dca64ea66a3487e0b7174fa75235894bfd5ab669ac04629ce`.

All eleven frozen current-observation files match the exact hashes listed in
the builder result. I independently recomputed those hashes. The two authorized
destination paths are the only claimed additions in the builder result. No
test, compiler, formatter, scanner, build, Git operation, or Graft build was
run, as required; no RED or GREEN result is claimed.

## Finding

**P1 — Invalid settlement fixtures undermine the projection oracle.** The
tests assign and assert `Settlement == "settled"` at test lines 29, 51, 138,
154, 184, and 297. The contract's complete allowed settlement vocabulary is
`unsettled`, `accepted`, `rejected`, and `escalated`
(`internal/contract/contract.go:436`). Retained state is sourced from the
validated adapter projection (`run_observation_retained_version.go:36-50`),
and its successful candidate builder copies validated source standings. Thus
`settled` cannot represent a successful captured state. These fixtures should
use a valid terminal settlement (for example `accepted`) and assert that exact
value. Until repaired, the TDD cases assert behavior against impossible source
state and do not establish the requested standing preservation oracle.

## Coverage and deviations

The private type fields match the authorized per-run window, run delta, gap,
and watermark roles. The seam accepts only a retained state, copied exact-ID
ledger, and prior watermark. Its body is a typed-unavailable zero-output stub;
it contains no algorithm or public type substitution. The test suite has seven
focused functions and covers ordinal boundaries, captured window/time/order,
later ordinals above H, standing/window-only updates, opaque-ID reappearance,
optional pointer ownership/input immutability, malformed/unavailable inputs,
and an empty successful terminal capture. The clarified gap flags are asserted
true. The required no-RED-before-independent-review sequence was followed;
not running tests is authorized and is not a deviation.

No other blocking mismatch was established from source inspection. A fresh
builder should correct only the invalid settlement fixture values/assertions,
preserve the two frozen current candidate files byte-for-byte before editing,
return new hashes and a new immutable result, then request a different
independent source review. Do not infer algorithm/runtime/Next/T09 approval.
