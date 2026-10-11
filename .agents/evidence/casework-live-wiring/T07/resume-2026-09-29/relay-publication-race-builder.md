# T07 relay publication race builder handoff

Root recorded this original assignment and delivered result because the fresh builder's
evidence write was blocked by the read-only `.agents` mount. No builder test pass is claimed.

## Original bounded instructions

Change only relay cursor publication and focused server tests. Preserve observed Head and
CursorForCase semantics used to refuse stale intents. First add deterministic blocked-capture
tests: WaitForCaseAdvance must not return an observed cursor until its revision is successfully
appended and immediately queryable by Store.At/Trajectory. Cover capture error and append
refusal without publishing either failed revision, and eventual wakeup on a later success.
No dependencies, public interface, security model or unrelated files may change.

After independent red baseline, implement a separate published per-case cursor, updated and
notified only after successful store append. Never hold the relay mutex across source I/O.
Context/feed termination must return only known retained publication. Preserve the red tests;
add termination coverage. Builder must not compile/test/vet/build or operate runtime processes.

## Independent red evidence

The independent critic reproduced the live accepted-cursor race and then ran the focused
race tests. Queryability, capture-error and append-refusal assertions all failed with premature
observed cursor 01BBB. Exact command, output, resource checks and the original test snapshot
are in `../resume-2026-09-30/relay-cursor-publication-race-red/`. Root authorized phase two only
after this test result. The initial Go-cache permission failure remains separate from that red.

## Delivered implementation

`internal/server/relay.go` retains observed `caseCur` and adds published per-case state.
Observed Head/CursorForCase still advance before capture; successful Store.Append precedes
published advancement and waiter notification. Capture and append failures do not publish.
WaitForCaseAdvance predicates and timeout/feed-end return paths use published cursors.
The new regression test file also covers termination while a newer capture is blocked.

Builder reports the current production constructors create an empty store before NewRelay,
so no preloaded-store bootstrap was introduced. Existing observed staleness behavior stays
intact. Builder ran only gofmt and diff whitespace checks. Production correctness, focused
green tests, global gates and rebuilt-binary immediate history reads require independent review.
