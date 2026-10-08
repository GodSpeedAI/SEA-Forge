# Run-child runtime test phases

Root supplement to ORIGINAL run-children-runtime-assignment.md. Runtime write release
requires root verification of the repaired pure-child fixture's independent expected RED.
The following phaseA is test-first preparation, not implementation approval.

PhaseA owner t09_ask_fixture_independent_critic adds ONLY semantic RunListResult declaration
in ports.go and factual UnreadableRunIDs []string in CaseFacts (builder.go); no interface
method, constructor, clone, scoped adapter/source or child implementation yet. These two
declarations let new behavior tests compile against the existing implementation. Existing
run_children_test.go remains frozen4ddb2445... and every existing test/golden is preserved.

Own new tests ONLY internal/adapters/sfwp/scoped_runs_test.go,
internal/projection/scoped_live_test.go and internal/projection/unreadable_runs_test.go.
Use a test-local scoped interface type assertion against Authority to exercise the absent
method without referencing an undefined constructor; after implementation, the same tests
exercise the real adapter and assert actual outgoing case_id, verb and mapping. Legacy
NewRunList encoding stays unscoped. Never invent a production stub or loosen assertions.

Explicit LiveSource fake declares the future scoped method with ports.RunListResult and
counts both scoped and legacy calls; legacy failure is deliberate, not a nil-interface
panic. Tests assert exactly1 scoped and0 legacy, preserved unreadable IDs, requested view
refs/readable case/actual parent validation and malformed/duplicate/overlap failures.
Store tests seed unreadable IDs into actual captured CaseFacts and prove both append and
returned-read cloning. All adapter refusal/missing/null/foreign/duplicate/overlap/time/enum
cases in original assignment remain required, even if baseline first fails missing method.

Freeze declarations/tests/hash/coverage/deviations, no compile token for builder. Independent
critic reviews original assignment, fixture clarifications, this supplement and actual files,
then runs compiling assertion RED under sole token with actual-host preflight. Distinguish
which baseline assertions executed versus guards blocked by absent scoped method. Missing
method must be an assertion failure, not a compile failure. After approval/root byte checks,
PhaseB implements the original owned runtime files with every fixture frozen. Critic then
runs focused GREEN, canonical/full Go and actual-kernel scoped-list proof. Any REJECT requires
a fresh builder. Full T09 remains pending.
