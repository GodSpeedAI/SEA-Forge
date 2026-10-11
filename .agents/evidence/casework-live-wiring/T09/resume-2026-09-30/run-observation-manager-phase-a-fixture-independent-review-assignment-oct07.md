# Phase A fixture repair independent-review assignment — 2026-10-07

The full original Phase A release is archived in
`run-observation-manager-phase-a-testfirst-assignment-oct07.md` (SHA-256
`7211787b2f4a07db3d8cddb031ee7f644f38ac5b5331f12ab0de54d5fec66565`). The
complete bounded repair instruction is archived in
`run-observation-manager-phase-a-fixture-repair-assignment-oct07.md` (SHA-256
`dc37aa4b06d995431fadaa581ce9a87e85fc62db7514534523bf1d6d81722d3e`), and
the repair result is
`run-observation-manager-phase-a-fixture-repair-result-oct07.md` (SHA-256
`68fb3350d6fcacaedefd5be026cad59a6e8f5bad6929b8a8d558c046318bfaa5`).

## Complete review instruction received

> PhaseA NEW independent SOURCEONLY repaired-fixture review. Read the complete
> repair instruction and result, original release, prior rejection, and full
> governing basis. Verify all three findings are resolved only in the fixture
> and the nine-case matrix is preserved. Recompute the exact original
> preimage from its native archive and compare it with the final fixture;
> verify all six committed primitives, existing `af2df` fixture, manager
> `f932`, and worker `f90` remain frozen. No compiler, formatter, scanner, or
> Git mutation. Root owns the sole heavy token. Produce a new immutable,
> evidence-based source verdict with the full original instruction, actual
> diff, preimage proof, final hashes, and material deviations. Do not fix source
> defects yourself. Expected-RED execution still requires a later explicit
> root grant.

## Required review checks

- Confirm the fixture-only diff is limited to the three authorized repairs:
  bind the caller returned by the fixture; remove direct `entry.cancel`
  replacement and the inconclusive `TryLock` spy; add negative read-eligibility
  assertions after Stop and detach win and before the owned launch.
- Recompute the native preimage's SHA-256 and compare its exact bytes to the
  claimed `3b0f...` identity. Confirm the final fixture hash is
  `4ff2fe1c922d6ae7e232d5a58f201298e232807437967fe22a9316dcb45aa0f3`.
- Verify all nine cases and their original assertions/order remain, and inspect
  for new hooks, direct lifecycle-field mutation, sleep-based race creation,
  fake port barriers, or broadened source scope.
- Preserve the original Phase A boundary: compile-safe test-first stubs only;
  no lifecycle algorithm, compile, focused RED, or runtime approval.
- Include exact hashes and source anchors. Do not run compile/test/format/scanner
  or mutate Git.
