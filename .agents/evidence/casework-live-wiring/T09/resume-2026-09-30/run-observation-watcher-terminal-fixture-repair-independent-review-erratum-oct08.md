# Independent review clarification: pre-port survivor ownership

Date: 2026-10-08

This additive clarification supplements, and does not overwrite,
`run-observation-watcher-terminal-fixture-repair-independent-review-oct08.md`
(SHA-256 `a830e3130a3c6c2f59273f788b2dc3638c533e55f4790f33353808bbfa0c1701`).
It records the final scope of the existing bounded rejection after the root
confirmed the precise ownership assertions required by cleanup clarification
`run-observation-watcher-terminal-tdd-cleanup-root-clarification-oct08.md`
(SHA-256 `7fe1dfde618e2740ab16911be13ae5fa6a2362cc82d99f0c7d94391ba2892d9d`)
and repair assignment
`run-observation-watcher-terminal-fixture-repair-assignment-oct08.md`
(SHA-256 `61d5bf6d435b73696b8b9adba7718b70b51a7f650a5a8ac40c013a301b65da6f`).

The pre-port group-6 assertion at current fixture lines 787–797 must verify
both `manager.pollers[key] == entry` and `entry.current.Key == key`, in addition
to the survivor cohort, both exact lease/entry membership directions, and
`!survivor.lease.draining`. It currently checks the survivor cohort and lease
refs, then only that `entry.current` is non-nil with `retainedCurrent`
availability. It does not establish that the manager still registers this
exact entry or that the retained value belongs to the expected key. The
captured `entry` and forward reference could be stale, and retained availability
alone is not the required exact valid retained value. The held-read sibling
already checks the manager registry mapping, but its `validEntry` condition
also needs `entry.current.Key == key` if that exact retained-key invariant is
to be claimed for both group-6 cases.

The verdict remains **REJECT source readiness pending these read-only
assertions in the new pre-port subcase** (and matching retained-key check in
the held-read subcase). This is one fixture-only repair; no production code,
behavior assertion, or execution gate is authorized by this clarification.
No compiler, test, formatter, scanner, build, Graft build, or Git command was
run. Formatting status is unverified and not claimed.
