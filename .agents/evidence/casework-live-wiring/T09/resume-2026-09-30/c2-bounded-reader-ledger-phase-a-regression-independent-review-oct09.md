# Bounded ledger production Phase A regressions: independent source review

Date: 2026-10-09

## Verdict

**APPROVE the Phase A fixture as source-ready for root's focused expected-RED
run only.** Both new regressions target the confirmed resume-row and
post-error session defects and are constructed to fail against the frozen
implementation. This does not resolve any implementation finding, approve
production source, or authorize Phase B.

## Reviewed identities and exact scope

- Grant: `c2-bounded-reader-ledger-slice-root-grant-oct09.md`, SHA-256
  `f1e242fdf6e22d61036348ef90d9addd7fa7249fb636dbd8e68d0c3d17a02e7c`.
- ACK addendum SHA-256
  `1d7f60e575be7035849214bee029d692091df4a1ebb90b601a19a4e93818fa6e`.
- Prior source rejection:
  `c2-bounded-reader-ledger-production-independent-source-review-followup-oct09.md`,
  SHA-256
  `0eea00a7edc4cdf77a4bef2b16261818892319a6c4a38b5357d51d46fdd90608`.
- Phase A receipt:
  `c2-bounded-reader-ledger-phase-a-regression-builder-oct09.md`, SHA-256
  `dc2a182a25ab20e573b99c0c96870a89dfb2d84309e55bde8e6d690192a563ac`.
- Frozen pre-change source: `types.rs` SHA-256
  `3045c6d3b089e119ac2ddf9cb186249d726fa3e543da1ed8766223584b8e4425`.
- Phase A source: `types.rs` SHA-256
  `84ee1cba001defd573f93a71f6465b77e7e41d5770395a3d6b580782b7e16ed3`.
- Diff metadata independently reproduced: 2,347 bytes, SHA-256
  `ee2e64c308c399096addcbb2e4cf5d8237f4840fe4f27bcdf204c0dc61743c14`,
  with the receipt's source labels and three context lines.

The production prefix is byte-identical to the frozen source. Removing the
two regression test blocks and the root-authorized unused test-constant
removal makes the entire test module byte-identical to the frozen module.
Thus all prior test assertions remain unchanged. The current `.agents/DEBT.md`
SHA-256 `1db732fe7f86c2d329cf637b4363dac5d529e7687e00f4f22dce7f6b20f72cba`
records only the two regressions as Phase A progress, states allocation
ordering is not tested, and leaves all defects and runtime approval open.

## Regression strength

`bounded_reader_regression_rejects_pretty_json_resume_position` serializes a
valid typed entry as pretty JSON, confirms the interior LF and typed equality,
and builds the exact checksum and offsets with `position_for_row`. It uses
that same position as both original pin and acknowledged position. Against the
frozen code, resume parses the multi-line JSON, validates the typed hashes and
position, and returns a session; the test's expected error therefore fails.
The test directly targets the resume constructor path that bypasses ordinary
forward LF splitting.

`bounded_reader_regression_errors_remain_terminal_after_bad_physical_row`
places `garbage\n` between valid rows 0 and 1. It consumes row 0, observes the
malformed-row error, and requires two later calls to remain errors. Against
the frozen code, extraction advances past the malformed line while
`next_offset` remains at row 0's end. The next call can validate row 1's
ordinal and predecessor against row 0, then return a row with a stale start
offset. The first repeated-call assertion therefore fails, exposing the
post-error continuation defect.

The prescribed scenario named rows 0, 1, and 2; this fixture contains rows 0
and 1, with the malformed line between them. I assess that omission as
nonmaterial for expected-RED coverage: the same stale-offset candidate is
returned for row 1, and because its computed end is short of the actual pin
end by the garbage length, the existing exact pinned-head comparison does
not prevent the return. No assertion is weakened or removed.

The allocation-before-budget finding is deliberately not covered by these
two tests. The receipt and CW-37 note say so accurately; that defect remains
open for Phase B source repair and later direct source review. The only
allowed next step is root's focused expected-RED run. Phase B remains held
until root accepts that evidence and grants it.

No compiler, test, gate, formatter, source edit, debt edit, Git operation, or
runtime correctness review was performed here.
