# Additional bounded-reader policy approval

The operator approved all recommended policies after independent proposal review `run-observation-bounded-ledger-reader-proposal-revision4-independent-review-oct08.md` (proposal-only approval). This approval supplements the earlier six-area C2 authorization.

Approved choices:

1. Generate a fresh memory-only Ed25519 continuation key at each server start. Entropy failure prevents startup. Restart invalidates prior tokens and requires safe bounded rebuild; never persist or log this key.
2. Admission limits: 4,096 encoded token bytes, 2,048 payload bytes, 2 MiB per raw ledger row including its newline, and 4 MiB raw input per page including non-events and lookahead. Preserve the existing approved 500-row, 500-frame, 1 MiB serialized response and 50 ms lock-wait limits. Oversize input returns unavailable. These are byte/admission limits, not an RSS or blocking-I/O latency guarantee.
3. A verified-empty v2 stream reports null head/frontier, no frames, no continuation, complete true and no synthetic cursor. A present requested cursor bound on empty history is an unknown-cursor input error. Empty success requires an actual registered absent/zero-byte readable stream under the cooperative lock, not corrupt or unreadable history.
4. Resolve v2 filter cursors during bounded validated paging. Reject unresolved requested bounds at the pinned head. Carry signed resolved filter ordinals only from acknowledged validated rows between pages; undelivered lookahead cannot advance either filters or frontier. Preserve legacy response shapes.

Implementation must first update the normative spec and ADR, then pass independent review and the applicable TDD/runtime gates. No runtime, migration readiness, task settlement, new dependency, or unrelated public-interface change follows from this approval.
