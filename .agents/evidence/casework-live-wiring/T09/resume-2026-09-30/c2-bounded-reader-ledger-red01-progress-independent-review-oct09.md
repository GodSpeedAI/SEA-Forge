# Bounded ledger reader RED01 progress: independent document review

Date: 2026-10-09

## Verdict

**HOLD the progress record for one failure-class correction.** The six
archived captures decode byte-for-byte to the six `/tmp/ledger-reader-red01-u7i7ieqs`
captures, and the reported preflight, timing, exit, and test totals match.
However, the claim that all 18 failed tests were explicit held-`todo!` panics
is too broad: the lock test catches its worker's stub panic and then fails its
own `failed == Some(true)` assertion. Correct that description before relying
on this immutable progress note as exact failure evidence.

## Verified evidence

I read the complete progress note and checked all six archived capture JSONs
against the originals under `/tmp/ledger-reader-red01-u7i7ieqs`. For every
capture, the decoded length and SHA-256 match both the archive metadata and the
original file:

| Capture | Bytes | SHA-256 |
|---|---:|---|
| command | 60 | `5bce3ff739c14e791099bf482747d7d6941cd5fcb8e9861606f9b163dc8452cf` |
| preflight | 35,244 | `f358e7f42ffab1fa4d3f08abbc9ab35bd1001452c50c474d9c4ac1486b2e92dd` |
| preflight exit | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| stdout | 9,648 | `166167cecc8641ae53d3fea5d97a3d64bfdc0e48b3e4f545ef009556f01af641` |
| stderr | 6,663 | `5b88d22ef3dc552618593a1e48201b89ae29658aecfa394a8eba215d2311829b` |
| exit | 4 | `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c` |

The captures show preflight exit 0, child exit 101, compilation finished in
14.82 seconds, and `0 passed; 18 failed; 14 filtered out`. The output contains
18 `not yet implemented` messages. Seventeen test failures directly surface a
held stub panic. The lock test's worker panic is caught; its test then fails at
the `Some(true)` assertion because the caught result is `None`. Thus the
unimplemented stub caused the worker panic, but the reported test failure is
an assertion failure, not an explicit uncaught TODO panic. There is no
compiler-error or timeout indication in these captures.

The attached CW-37 progress correctly limits RED01 to compiled scaffolding
with unimplemented stubs; it does not claim acceptance assertions passed,
reader correctness, or runtime approval. It records the unused
`READER_PAGE_BYTES` warning and the root-approved, source-only removal request
while retaining all 18 assertions. Root-accepted source and review identities
match the progress receipt. The production source was not reviewed here for
runtime behavior; the builder owns it, and independent GREEN remains pending.

## Requested correction

Preserve the capture identities and result, but describe the outcomes
precisely: 18 tests failed; 17 test failures directly surfaced held TODO
panics; the lock test caught its worker's TODO panic and failed its own
assertion. Keep the RED-only limitation and pending independent GREEN status.
No tests were run and no source, debt, status, or Git files were edited in
this review.
