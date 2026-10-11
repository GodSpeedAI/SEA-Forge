# T08 native retention-index defect: independent source review

## Verdict

**REJECT T08.** The native run reached phase-one terminal-state collection and exposed a
production `projection.Store.At` panic after bounded retention evicted an older revision.
This is a real store-index defect, not an EventSource assertion or test-fixture mismatch.

## Evidence

The recorded run is `/tmp/sea-t08-native-artifacts/native-go-race-after-transform-fix.log`.
It reports `runtime error: index out of range [1] with length 1` at
`internal/projection/store.go:119`, reached from `nativeEventPhase.state` and the
`/__native-test/stable-state/phase1` handler. The browser received HTTP 500 while requesting
the state after a successful execution receipt. The test exited nonzero; this is not a green
native EventSource proof.

Source anchors:

- `internal/projection/store.go:112-119`: `At` trusts `byCursor[cursor]` and indexes
  `revisions` without validating the retained slice position.
- `internal/projection/store.go:162-169`: `Append` records the new cursor's pre-eviction
  index, deletes only the evicted cursor, then removes the first slice element. It leaves
  the indexes of every retained cursor unchanged.
- `internal/server/native_events_live_test.go:515-528`: state collection calls `Store.At(head)`;
  the runtime stack shows this is where phase one panicked.

For retention 1, appending a second revision stores its index as 1, then slices the revisions
to length 1; `At` on the new head panics. For retention 2, appending a third revision leaves
the retained cursors mapped to their old indexes; one lookup can return the wrong revision and
the newest lookup can panic. This breaks the retained-history contract, including the T08
`Store.At` comparisons and HTTP historical reads.

## Scope and status

I independently read the saved runtime log and current store/test source. I ran no commands,
made no production edits, and did not approve T08. A fresh builder must repair the retained
cursor index behavior and add focused eviction/lookup coverage before the native and shared
live gates are repeated.
