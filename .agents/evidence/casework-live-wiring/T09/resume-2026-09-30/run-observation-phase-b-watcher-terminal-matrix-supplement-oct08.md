# Phase B watcher and terminal test-matrix supplement

Date: 2026-10-08  
Status: DOCONLY additive correction; no test, source, or runtime release.

## Provenance

This supplement answers only the test-matrix rejection in
`run-observation-phase-b-watcher-terminal-independent-review-oct08.md`
(SHA-256 `7108ed75`, full review preserved in the same evidence directory).
It adds to, and does not replace, the original watcher/terminal repair
assignment (`run-observation-phase-b-watcher-terminal-repair-assignment-oct08.md`,
SHA-256 `7ffdd48ccbfd69d4ff0e8bf5d7812405b7385651c5b5396e8b4c30b4e8deb852`),
the proposal (`run-observation-phase-b-watcher-terminal-repair-proposal-oct08.md`,
SHA-256 `de8c1647153e06a5fc492671d14c36d3e562f1cd1f079086b62546304e90f0ad`),
and root clarification (`run-observation-phase-b-watcher-terminal-root-clarification-oct08.md`,
SHA-256 `92c0a5bcbdc309c1f0e09c3ac4bf61dc350aee685985a5a824ee2dff72cabba5`).

The proposal's six cases and original twelve Phase B fixture cases remain
intact. The only additions are one four-cell final-handoff matrix and stronger
assertions in the existing fitting-terminal case. The two private immutable
lease fields (`caller runObservationCaller`, `asOfCursor string`) and required
`beginPrepareOperation` inputs remain exactly as proposed. Production source,
tests, and fixture identities remain frozen at manager `d7be1d7a`, worker
`84382ffd`, failure fixture `ef587b0b`, original fixture `af2dfcb8`, and the
six primitive identities recorded in the independent review.

## Added final-handoff matrix

Add one table-driven `prepare` test with the following four cells. Each cell
starts a real Prepare with a valid request identity and injected `Current` and
present-context guard dependencies. The actual list callback signals entry,
blocks on an explicit release channel, and then returns the specified list
result. Only after observing list entry does the test invalidate the selected
dependency, then release the callback. No sleep or production hook is needed.

| Changed dependency while list is held | List result after release | Required outcome |
|---|---|---|
| Current authorization becomes invalid | Empty readable list | Typed unavailable error; zero event; nil lease |
| Current authorization becomes invalid | Unavailable/error list | Typed unavailable error; zero event; nil lease |
| Present cursor changes from the admitted `asOfCursor` | Empty readable list | Typed unavailable error; zero event; nil lease |
| Present cursor changes from the admitted `asOfCursor` | Unavailable/error list | Typed unavailable error; zero event; nil lease |

The test changes the actual injected Current state or the actual guard's
returned cursor through test-owned synchronization; it does not mutate manager,
lease, or poller fields. After each Prepare returns, use read-only mutex
protected observation to confirm the admitted cohort and any owned poller
membership have been cleaned up. Also assert zero trace calls. The list-entry
signal proves admission and dependency invocation occurred before mutation;
the held callback proves the invalidation precedes the handoff. The unavailable
list cells must not be treated as successful unavailable DTOs, and the empty
cells must not be treated as successful `no_runs` DTOs.

Source-path review predicts that cursor-invalid plus an empty list may already
return the required failure because the current success-list path performs a
post-list guard/cursor comparison. That is not a reason to weaken or omit the
cell: the focused RED record must report each cell's actual result, including
any passing cell, and must not claim that all four fail. In contrast, the
current list-error early return precedes that later guard, and the current
handoff does not recheck authorization; those source paths motivate the other
cells. These are source-based predictions only, not executed test outcomes.

## Fitting-terminal assertion strengthening

Keep the proposed fitting-terminal test and final DTO/cache behavior. Exercise
one actual owner and one actual shared waiter on the same initializing entry:
the list and real trace callback use explicit entry/release channels, and the
second Prepare attaches while that read is held. After releasing a valid,
fitting terminal snapshot, assert:

* both Prepare calls return their valid final DTOs and nonnil leases;
* the owner reports `ReadsAttempted == 1`; the shared waiter reports
  `ReadsAttempted == 0`;
* the real worker's `workerDone` closes, and the trace callback count remains
  one after that completion;
* a subsequent real `claimPollerReadStart` on that terminal entry is refused;
* the retained terminal value remains available to the final DTO and the
  existing authorized cache/shared result, with no drain triggered merely by
  accepting the fitting terminal version.

Completion and assertion ordering use the real worker channel, not elapsed-time
assumptions. The terminal claim refusal plus joined worker proves there is no
later claim/call path; the fixed trace count records the actual calls. Preserve
all existing terminal and lifecycle assertions.

## Boundaries and review status

These are test-design additions only. They add no production field, interface,
hook, enum, counter, mode, or synchronization mechanism. The actual RED result
must record top-level and all four handoff-cell outcomes independently. The
matrix does not imply that every cell is currently red. This supplement grants
no source edits, test execution, compiler, formatter, scanner, build, Git
operation, GREEN, runtime approval, or lifecycle approval. The full original
assignments and reviews remain immutable.
