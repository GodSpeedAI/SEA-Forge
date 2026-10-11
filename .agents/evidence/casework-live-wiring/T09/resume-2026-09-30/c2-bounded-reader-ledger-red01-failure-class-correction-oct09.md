# Bounded ledger reader RED01: failure-class correction

Date: 2026-10-09. This correction supersedes only the failure-class wording in
the immutable RED01 progress receipt. The original receipt and its independent
review remain unchanged. Root's accepted semantic result remains expected
unimplemented RED.

## Verified run result

The progress receipt is
`c2-bounded-reader-ledger-red01-progress-oct09.md` (SHA-256
`ce150cb82fccfc791df77b709bb14baf2172d9480b1332ecb95e1d6845a8cada`). Its
independent document review is
`c2-bounded-reader-ledger-red01-progress-independent-review-oct09.md` (SHA-256
`10ab732db68286d1c14333b43b1ffbdbe29d378096060a1e92b2fe474a47e8bb`). The
actual stdout is `/tmp/ledger-reader-red01-u7i7ieqs/stdout.raw`, SHA-256
`166167cecc8641ae53d3fea5d97a3d64bfdc0e48b3e4f545ef009556f01af641`.

The six archived captures were decoded and compared byte-for-byte with the
actual files under `/tmp/ledger-reader-red01-u7i7ieqs`; every archive metadata
hash, decoded length, and original-file hash matched:

| Capture | Bytes | SHA-256 |
| --- | ---: | --- |
| command | 60 | `5bce3ff739c14e791099bf482747d7d6941cd5fcb8e9861606f9b163dc8452cf` |
| preflight | 35,244 | `f358e7f42ffab1fa4d3f08abbc9ab35bd1001452c50c474d9c4ac1486b2e92dd` |
| preflight exit | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| stdout | 9,648 | `166167cecc8641ae53d3fea5d97a3d64bfdc0e48b3e4f545ef009556f01af641` |
| stderr | 6,663 | `5b88d22ef3dc552618593a1e48201b89ae29658aecfa394a8eba215d2311829b` |
| exit | 4 | `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c` |

Preflight exited 0. The Rust test child exited 101 after compilation completed
in 14.82 seconds. The focused result was 0 passed, 18 failed, and 14 filtered.
Stdout contains 18 `not yet implemented` messages. Seventeen test failures
directly surfaced an uncaught held-`todo!` panic. The lock test caught its
worker's held-`todo!` panic, then failed its own `failed == Some(true)`
assertion because the caught result was `None`. Thus there were 18 failed
tests, 17 direct TODO-panic test failures, and one assertion-failure test; it
is inaccurate to say all 18 test failures were uncaught TODO panics. Captures
show no compiler error or timeout.

This proves the scaffold compiled and produced the expected unimplemented RED
only. No test passed; no acceptance assertion or runtime behavior is approved.
The semantic RED result is unchanged.

## Current authorization boundary

The earlier cleanup-only request to remove the unused `READER_PAGE_BYTES`
declaration while preserving all assertions in the 18 tests is historical.
After accepting RED, root released the full ledger-reader implementation
scope under the bounded-reader grant, not merely that constant cleanup.
Implementation is not thereby accepted; independent GREEN review and runtime
proof remain pending.

This correction performed no tests, gates, compilation, or source edits. Only
CW-37 in `.agents/DEBT.md` was corrected to use the failure classes above and
to record the current full-grant release boundary. The original RED01 receipt
and independent review were preserved.
