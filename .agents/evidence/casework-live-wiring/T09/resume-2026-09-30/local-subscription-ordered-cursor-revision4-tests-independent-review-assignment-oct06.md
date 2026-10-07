# Local ordered-cursor revision 4 — independent source review assignment

Date: 2026-10-06. DOCONLY review assignment.  
Builder artifact: `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorBounds.test.ts`.

Review this new TESTONLY artifact against the original local ordered-cursor assignment, revision 4 proposal, revision 4 independent review and citation erratum, the frozen `localAdapter.cursorOrder.test.ts`, and actual current local adapter/timer helpers. Root owns compiler and all runtime gates: do not run Bun, typecheck, tests, Git, or build commands. Do not edit production code, the frozen test, or any other source/test/config file.

Inspect every assertion and private white-box access. Confirm each test has a deterministic controlled-timer path, exercises the named accepted obligation, and is expected to be meaningfully RED on the current production baseline for the intended behavioral gap rather than an unrelated setup/type error. Specifically review decimal parsing and numeric leading-zero equality, unsafe/malformed/wrong-epoch/over-ceiling rejection with one error and no registration, valid same-epoch future floors, constructor seed agreement/failure, the per-subscriber 64/65 FIFO boundary with healthy subscriber continuity, reentrant ordering/self-unsubscribe/throw isolation, timer cancellation, and schedule-failure handling after event commit.

Check that the helper restores global timer functions in `finally`, that shared fixture mutation is restored even on failure, and that any inability to exercise an accepted obligation without production exports/configuration changes is reported as a concrete limitation rather than silently expanded into scope. Verify the existing frozen cursor-order file was not edited. Record source-level findings, any defects, hashes, and a clear accept/reject verdict in a NEW review artifact only if separately assigned. This assignment grants no implementation approval and makes no runtime claim.
