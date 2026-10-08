# Scope correction to the independent fixture review erratum

Date: 2026-10-08

This note supplements the immutable review and clarification records
`run-observation-watcher-terminal-fixture-repair-independent-review-oct08.md`
(SHA-256 `a830e3130a3c6c2f59273f788b2dc3638c533e55f4790f33353808bbfa0c1701`)
and `run-observation-watcher-terminal-fixture-repair-independent-review-erratum-oct08.md`.

The required missing checks are scoped to the **new pre-port group-6
subcase**: after failed-initiator cleanup, assert `manager.pollers[key] ==
entry` and `entry.current.Key == key`, along with the already-required
survivor cohort, exact refs in both directions, and not-draining status. The
held-read sibling already asserts exact registry mapping and retained-current
availability. The prior erratum's suggestion to add `entry.current.Key ==
key` to that sibling is not part of this bounded repair assignment and should
not be treated as a separate defect. The source-readiness verdict remains
REJECT until the two pre-port checks are added.

No source or test was changed and no test, compiler, formatter, scanner, build,
Graft build, or Git operation was run.
