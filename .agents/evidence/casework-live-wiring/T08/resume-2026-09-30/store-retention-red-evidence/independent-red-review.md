# Projection retention index tests: independent RED review

## Source review

The tests-only change in `internal/projection/store_retention_test.go` covers the observed
index-shift defect without weakening existing behavior:

- Retention 1 appends C then L, requires C to return `ErrUnknownCursor`, requires L to resolve
  to its own cursor/snapshot/case, and checks the one-entry slice/index.
- Retention 2 appends five ordered cursors across repeated evictions, checks typed errors for
  every evicted cursor, exact retained cursor/snapshot/case resolution, and bounded aligned
  slice/index entries after each append. It then subscribes from evicted C and checks replay
  contains exactly retained D then E.
- Existing `TestStoreRetainsDeepImmutableFactsForEachCursor` and
  `TestStoreTrajectoryIsCaseScopedBoundedAndCloned` cover nested fact/snapshot clone behavior;
  unique strictly advancing cursors in the new tests distinguish a shifted lookup from the
  requested retained revision.

No production source was changed by this verifier.

## Independent focused RED runs

Each test ran separately under Go race instrumentation, with `GOMEMLIMIT=256MiB`, `GOGC=50`,
`GOMAXPROCS=2`, `GOFLAGS=-p=1`, `-p=1`, `-parallel=1`, `-count=1`, and the existing writable
Go cache. Both compiled and failed on the reported defect rather than a build/setup error:

1. `TestStoreRetentionOneKeepsLatestAtAndEvictsPrevious`: exit 1, panic at `Store.At` because
   retained cursor `01BBB` has index 1 in a length-1 slice.
2. `TestStoreRetentionTwoKeepsEveryRetainedIndexAndReplaysInOrder`: exit 1, `At(01BBB)` returns
   revision `01CCC`, demonstrating stale retained indexes without relying on a panic.

Raw command/output/exit logs:

- `retention-one.log` SHA-256 `259f1ad0fc20391e669ef508b5420ea597cb89c3e006d1000eb8cda479552bff`
- `retention-two.log` SHA-256 `20c6c4680df8a3963bb35d7307b28db7272e8575b98d3b1ab09dd6ca06ac0811`

## Verdict

The focused tests provide the expected independent RED evidence for the production store-index
defect. T08 remains unapproved pending the authorized minimal production repair and GREEN
verification. The sole compile token was released to root after these runs.
