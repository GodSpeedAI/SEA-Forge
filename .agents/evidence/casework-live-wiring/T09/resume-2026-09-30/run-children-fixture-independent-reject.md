# Independent run-child fixture source review: REJECT

Review date: 2026-10-01

## Reviewed source identity and scope

Reviewed only the original fixture contract, its root clarification, and the frozen test file. No production files were edited and no compiler or tests were run.

- `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-children-test-first-assignment.md`
- `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-children-root-fixture-clarifications.md`
- `apps/godspeed-casework-go/internal/projection/run_children_test.go`
- Frozen test SHA-256: `68511dc5618c9609e27ae501794265d8fd4d5b18d62466e2465dd51b3507269d`
- Frozen test length: 278 lines

Read-only identity command: `sha256sum apps/godspeed-casework-go/internal/projection/run_children_test.go && wc -l apps/godspeed-casework-go/internal/projection/run_children_test.go`.

The intermediate pre-review hash `034d9dff69ef7b58618dd77fc64f19a37faf2a4a1b6deba0db9660b8a0f31300` was superseded before this review by the clarification-driven focus assertion edit. No verdict or compiler run was issued against that intermediate file.

## Blocking finding

The primary matrix does not prove exactly one child for every valid summary. At lines 40–49, each emitted `execution_trace` is checked only for membership in `want`. At lines 83–85, the test checks only that the aggregate child count is 24. It does not record IDs already seen or assert that every key in `want` appeared. Therefore 23 distinct expected IDs plus a second child with a duplicate ID can satisfy both checks while one valid summary has no child.

The original assignment requires “one child per summary” and an exact run ID for each child. The matrix must track emitted IDs, fail duplicate IDs, and assert every expected ID was seen exactly once. Aggregate count plus membership is insufficient. This is a core contract omission, so the fixture is rejected before compilation; no expected-RED evidence is claimed.

## Other reviewed requirements

The frozen file otherwise includes the six-by-four execution/settlement matrix, actual horizon parent linkage, empty child actions, observation-state key exclusions, >8-summary coverage, preserved parent actions and clarified focus checks, invalid row/collision cases with valid sibling preservation, and retained Store facts/snapshot checks. These observations do not cure the missing exact one-to-one matrix assertion.

Disposition: source REJECT. Return the fixture to a fresh builder for the bounded test-only correction. Keep production/API files untouched. The compiler token was not used and remains with root.
