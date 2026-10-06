# Selection exact-ID adjudication and fresh fixture repair

The independent rereview's remaining tie-order finding is a FALSE POSITIVE.
The required ordering compares entire exact run-ID bytes, not the suffix letter.
`run_terminal_completed_tie_z` sorts BEFORE `run_terminal_failed_tie_a`: the first
different bytes are `c` (99) and `f` (102). Root executed deterministic UTF-8 buffer
comparison; Buffer.compare returned -1 and full-string a<b returned true. Both IDs
are ASCII, so this is also their Go string byte ordering. Do NOT swap these rows.
The current terminal expected order correctly follows the original assignment.
The incorrect independent rejection remains immutable and requires a corrected
evidence-based independent review, rather than distorting a correct fixture.

Root did identify a separate coverage omission in the fresh repair: the original
empty input was a nonnil zero-length slice, but the repaired empty subtest only
passes nil. Actual RunsListForCase success returns a nonnil empty Runs slice.
Both are empty inputs under the original selector contract. Retain the nil test
AND cover nonnil empty input; both assert successful len0, without prescribing
the output representation. Do not weaken or remove either input shape.

## Different fresh builder original bounded repair assignment

Read original selection/fixture-repair assignments, both independent rejections,
this adjudication and the whole current fixture/stub. Edit ONLY the selection TEST
file: add nonnil-empty coverage alongside existing nil-empty coverage. Keep exact
terminal tie order unchanged, and preserve ALL other tests/assertions. Verify its
full-string byte ordering with a deterministic standard-library command; no Go
compilation is needed. Source stub1e5cdf40 unchanged. Native patch; read-only gofmt-d
allowed. No compile/tests/scanner/Graft build/Git/status/debt/evidence edits.
Return hash, complete bounded diff, byte-order evidence, deviations and final freeze.

Independent critic must receive ALL ORIGINAL assignments, the root adjudication
and complete fresh result. It must check exact FULL IDs with direct byte-comparison
evidence, acknowledge the false finding explicitly, and independently verify both
empty input shapes plus all prior invariants. Source-only approval cannot prove RED.
