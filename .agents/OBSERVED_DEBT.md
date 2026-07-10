# Observed Debt

Record concrete debt, gaps, risks, or defects found while working but outside
the current task. Each entry needs evidence, impact, and a next move. Remove an
entry when resolved; do not use this file as a backlog of ideas.

## Open: Minimum runtime diagnostics are not implemented

- Observed: 2026-07-10
- Evidence: `crates/sea-forge-core/src/lib.rs` is still a foundation skeleton;
  `.agents/specs/spec-minimum.md` §12.3 requires contextual diagnostics and
  governed lifecycle trace events.
- Impact: agents can diagnose the development foundation and CLI startup, but
  cannot inspect real authority, execution, evidence, or settlement failures yet.
- Next move: implement and instrument each minimum-kernel lifecycle module under
  its spec task. Require `run_id`, component, and error class on diagnostics and
  verify `trace.jsonl` event order in conformance tests.
- Scope: planned minimum-kernel acceptance work; do not create fake telemetry
  before the underlying lifecycle exists.

<!-- Entry format:
## Open: Short problem statement
- Observed: YYYY-MM-DD
- Evidence: file, line, command, or failing proof
- Impact: concrete cost or risk
- Next move: smallest credible resolution
- Scope: why it was not fixed in the discovering task
-->
