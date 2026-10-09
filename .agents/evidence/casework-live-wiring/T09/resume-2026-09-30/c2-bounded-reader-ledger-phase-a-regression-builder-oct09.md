# Bounded ledger production repair Phase A receipt

Date: 2026-10-09

## Authority and source identity

This test-only phase follows the root grant
`c2-bounded-reader-ledger-slice-root-grant-oct09.md` (SHA-256
`f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`) and
ACK-boundary addendum
`c2-bounded-reader-ledger-slice-ack-boundary-addendum-oct09.md` (SHA-256
`1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`). The
prior production builder receipt is
`c2-bounded-reader-ledger-production-builder-oct09.md` (SHA-256
`103cf4777a4ea2c08c716f8d131ceaa0195f781fc294f4f9f973102fda0a7bce`). Its
source was frozen at SHA-256
`3045c6d3b089e119ac2ddf9cb186249d726fa3e543da1ed8766223584b8e4425` and
preserved in `c2-ledger-production-initial-frozen-source-oct09.raw.json`.
Decoding that xz+base64 before-image reproduced the current source exactly
before this phase's edit.

The controlling follow-up review
`c2-bounded-reader-ledger-production-independent-source-review-followup-oct09.md`
(SHA-256
`0eea00a7edc4cdf77a4bef2b16261818892319a6c4a38b5357d51d46fdd90608`)
rejects the prior source for three findings: resume does not prove the
boundary probe plus claimed row fit the remaining page budget before
allocation; resume accepts interior LF bytes in a claimed physical row; and a
session can yield a row after an earlier `next_step` error.

## Phase A change

Only tests were added to `crates/sea-forge-ledger/src/types.rs`, with the
required `bounded_reader_regression_` prefix:

- `bounded_reader_regression_rejects_pretty_json_resume_position` builds a
  pretty-printed typed entry with interior LF, confirms it parses as the same
  entry, constructs the actual position through `position_for_row` (including
  its exact raw checksum and offsets), writes those bytes, and uses the same
  position as pinned head and acknowledged row to exercise resume validation.
- `bounded_reader_regression_errors_remain_terminal_after_bad_physical_row`
  writes valid rows 0 and 1 with `garbage\n` between them, reads row 0, then
  requires the malformed-row error and requires two repeated calls to remain
  errors.

The before-image source SHA-256 was
`3045c6d3b089e119ac2ddf9cb186249d726fa3e543da1ed8766223584b8e4425`; the
Phase A source SHA-256 is
`84ee1cba001defd573f93a71f6465b77e7e41d5770395a3d6b580782b7e16ed3`. The
unified diff against that before-image is 2,347 bytes with SHA-256
`ee2e64c308c399096addcbb2e4cf5d8237f4840fe4f27bcdf204c0dc61743c14` (labels
`types.rs@3045c6d3` and `types.rs@phase-a`, context 3, LF line endings). The
existing 18 tests and their assertions are unchanged. No production source,
helper, error type, limit, server boundary, or test policy changed.

## Verification and disposition

No tests, compiler, formatter, gate, or Git operation was run for this phase.
These tests are staged for root's focused expected-RED run and independent
source review. They exercise the physical-row and post-error session findings;
they do not prove allocation ordering. All three review findings remain open,
no reader defect is closed, and runtime approval remains pending. Phase B
production repair remains held until root accepts the focused RED and grants
the next phase.

The associated CW-37 progress note is in `.agents/DEBT.md` (SHA-256
`1db732fe7f86c2d329cf637b4363dac5d529e7687e00f4f22dce7f6b20f72cba`).
