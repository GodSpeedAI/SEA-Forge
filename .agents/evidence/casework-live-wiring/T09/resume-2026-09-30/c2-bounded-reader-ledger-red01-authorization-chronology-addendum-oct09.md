# Bounded ledger reader RED01: authorization chronology addendum

Date: 2026-10-09. This immutable addendum clarifies chronology wording only.

## Records corrected

- `c2-bounded-reader-ledger-red01-failure-class-correction-oct09.md`, SHA-256
  `970f18abae34e82e50a06ba6ce258867c4450bce05fe3eed00a246033ee5df87`.
- `c2-bounded-reader-ledger-red01-correction-independent-review-oct09.md`,
  SHA-256
  `9bd77ccd4ec430619cd5fbf91aac441e2d1b1225b5bc55a4595999c061ed66c4`.

The correction and its review incorrectly describe an earlier separate
cleanup-only production authorization that was later expanded. Root's
append-only `.agents/CURRENT_STATUS.yaml` revision 35 records the actual
sequence: after accepting RED, root released the bounded-reader implementation
assignment for `types.rs`, including removal of the unused
`READER_PAGE_BYTES` declaration while preserving the 18 tests' assertions.
There was no separate cleanup-only production release; an earlier record
described the cleanup without a full implementation grant. Revision 35 is the
authority for this chronology.

This addendum supersedes only that nonexistent cleanup-only phase wording in
the two records above and in CW-37. The 17 direct TODO-panic failures and the
lock test's caught TODO panic followed by assertion failure remain unchanged,
as do the six-capture comparison, exits, compile timing, current full-grant
implementation authority, and absence of GREEN or correctness claims. This
clarification changes neither approval policy nor the current implementation
release. The source remains pending independent GREEN review and runtime proof.
