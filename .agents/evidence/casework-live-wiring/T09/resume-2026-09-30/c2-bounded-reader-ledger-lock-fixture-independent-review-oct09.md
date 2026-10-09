# Bounded ledger reader lock-fixture repair: independent review

Date: 2026-10-09

## Verdict

**APPROVE source-ready for root's expected RED run.** The lock fixture now has
a 150 ms upper tolerance around the grant's 50 ms monotonic deadline while
retaining its 40 ms lower bound and cleanup ordering. This is source readiness
only: root must verify expected RED. This review makes no compile, test, or
runtime claim and does not approve reader implementation.

## Reviewed identities and exact change

- Governing grant: `c2-bounded-reader-ledger-slice-root-grant-oct09.md`,
  SHA-256 `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d4c83d17a02e7c`.
- ACK-boundary addendum:
  `c2-bounded-reader-ledger-slice-ack-boundary-addendum-oct09.md`, SHA-256
  `1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`.
- Previous source review finding: the 1-second ceiling could accept a
  materially late lock wait; review SHA-256
  `d3a78f7b5e65cfc655a2a6caa3f19d682c919f4b08c04b688a610287de62ad80`.
- Fresh repair receipt:
  `c2-bounded-reader-ledger-lock-fixture-fresh-repair-builder-oct09.md`,
  SHA-256 `2f88d754de94d4128c677750c994678920f93aa2b85fa325d930f95374a03715`.
- Fresh source `crates/sea-forge-ledger/src/types.rs`, SHA-256
  `6a22fbd14cafb5312cf81f2b2b694cb2413fbe21afaf35dcc4263a27d25e0c33`.
  The immutable repair-1 archive decodes to 105,930 bytes and SHA-256
  `587619ce307c1ee8ed996e0cad0106691c1bb798ec75c880c68d4c848a7f8619`;
  the current file is 105,934 bytes. Direct byte comparison shows exactly one
  source change, in `bounded_reader_fails_closed_when_cooperative_lock_is_held`:
  `Duration::from_secs(1)` becomes `Duration::from_millis(150)`.

The readiness, bounded receive, 40 ms minimum, lock-holder release, and both
thread joins before result assertions remain unchanged. The repair introduces
no implementation or new helper and leaves all explicit reader/arithmetic
stubs untouched. CW-37 progress correctly records the fresh repair and review
state; no additional debt or status edit was made for this review.

## Review against the grant

This one-line repair resolves the prior fixture blocker: an implementation
waiting hundreds of milliseconds can no longer pass the 150 ms assertion.
The 150 ms ceiling is the explicit scheduling allowance selected by root; it
does not claim exact 50 ms elapsed runtime behavior. Before implementation is
approved, source review must confirm that each lock retry is governed by the
actual monotonic 50 ms deadline, as required by the grant.

The previously reviewed fixture coverage remains unchanged: shared accounting
for auxiliary and forward raw reads, raw-row checksum binding independent of
typed entry hashing, bounded rows and row counts, resume pin/predecessor
validation, integrity failures, and distinct `LimitReached` versus complete
outcomes. The ACK addendum remains caller/server-owned: each event row must
pass exact known-shape validation before filtering or ACK/frontier/frame
advancement, including filtered-out rows. The ledger-only fixture does not
claim event ACK eligibility.

No compiler, test, formatter, gate, runtime action, Git operation, source
behavior change, dependency/schema/error/route/wire/token change, or status
update was performed as part of this review. Root owns expected RED and all
compilation. Permission-denied regular-file coverage remains absent as
previously disclosed; the fixture covers a directory at the entries path and
a missing-parent I/O failure.
