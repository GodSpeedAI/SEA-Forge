# Private Next projection algorithm result

Date: 2026-10-08. The bounded pure run-delta assembler is implemented in its
released source file. This is source-result bookkeeping only; it does not claim
that the implementation compiles, passes tests, or integrates with Next,
manager lifecycle, public wiring, SSE/UI, or T09 as a whole.

## Authority

The complete original source grant is
`run-observation-next-projection-assignment-oct08.md` (4,655 bytes; SHA-256
`c6bd03f372140f0542f92c3c15a7e6b3f2471d311ef9bd705f0a2acc3c832c9d`). Its
additive malformed-input/gap clarification is
`run-observation-next-projection-root-clarification-oct08.md` (1,061 bytes;
SHA-256 `1a3685eb42c287c54ecbe09830827fbe6edce85aa96349b0f9f724269c526fee`).
The complete TDD source-preparation record is
`run-observation-next-projection-tdd-source-preparation-result-oct08.md`
(4,100 bytes; SHA-256
`0354cbbdaec2061e2d323eebe95e2da62a095102195cb8cb04ff75f7b4ae2a38`), with
independent source approval in
`run-observation-next-projection-source-review-revised-oct08.md` (3,104 bytes;
SHA-256 `99dccce430a62677653e9935134b71236c825bbe2cf5011b98456e82915c7f7f`).
That source review's test fixture identity remains frozen. The governing basis
is revision 6 (`60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`),
its addendum (`9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`),
and retention initializer correction revision 3
(`b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`).
Those original documents remain immutable in the T09 evidence directory.

## Source identity and scope

Only `apps/godspeed-casework-go/internal/server/run_observation_delta.go` was
edited by this implementation. The authorized stub preimage was 1,651 bytes,
SHA-256 `aff93e3a1b01aedd8075be1f59ff52cebb4dc36de59eaf4140cb939cdf148973`.
The resulting file is 5,541 bytes, SHA-256
`3c81aa06b4f2062e0d3af314635c4c141c9b407d6cee32afe579a03a391cbbb4`. The
frozen `run_observation_delta_test.go` remains 13,953 bytes, SHA-256
`af862f3bd18e866db8ccb8c84ef784a80d8a3d0d49e5dc8f47f3969d78a77cef`.
Its existing worktree difference from HEAD predates this implementation; it
was not edited. All eleven other observation source/test identities match the
TDD preparation result's table and were independently recomputed unchanged.

## Implemented projection

The function rejects nil or non-current state, impossible total/retained
counts, a prior watermark above captured `H`, and inconsistent captured
frame/ledger ordinals with a typed unavailable error and zero outputs. It
assembles only from the supplied immutable state, copied ledger, and prior
watermark. It ignores ledger entries above `H`, selects captured frames with
first ordinal greater than `P` in source order, and deep-copies optional frame
pointers into contract values. Window metadata and the candidate watermark
come from that same captured state. For IDs in `P < ordinal <= H` absent from
the captured window it returns a gap with byte-sorted exact IDs and both
uncertainty flags set.

As an additional fail-closed structural check, the captured portion of the
copied ledger must have unique nonzero ordinals covering `1..H`, and captured
window IDs must be unique. Entries above `H` are exempt from that check and are
ignored as required. This relies on the approved entry-lifetime exact ledger;
it does not parse, normalize, or order opaque IDs.

The first read-only `gofmt -d` showed only field alignment in the gap struct;
that whitespace was corrected with `apply_patch`. The final `gofmt -d` emitted
no diff. No tests, compiler, scanner, security gate, build, or Git mutation was
run for this result. The focused test-first RED and fixture are historical
inputs, not verification of this implementation. A separate reviewer and
root-owned serialized test grant remain necessary before any algorithm or
runtime approval.
