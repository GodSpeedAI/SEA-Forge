# Observed Debt

Record concrete debt, gaps, risks, or defects found while working but outside
the current task. Each entry needs evidence, impact, and a next move. Remove an
entry when resolved; do not use this file as a backlog of ideas.

<!-- Entry format:
## Open: Short problem statement
- Observed: YYYY-MM-DD
- Evidence: file, line, command, or failing proof
- Impact: concrete cost or risk
- Next move: smallest credible resolution
- Scope: why it was not fixed in the discovering task
-->

## Open: `spec-implementation-audit.md` Thoth-governance finding is stale

- Observed: 2026-07-24
- Evidence: `.agents/reports/2026-07-22-spec-implementation-audit.md` §3
  finding 2 states Thoth `ask` "bypasses governance and leaves no required
  chain." Direct inspection during SFWP grounding
  (`.agents/reports/2026-07-24-sfwp-grounding.md`, thoth family) found
  `sea_forge_thoth::service::ask` (`crates/sea-forge-thoth/src/service.rs:258-350`)
  already commits `self_disclosure_question`, `self_disclosure_plan`, decision,
  and answer to the `thoth-asks` ledger stream before returning.
- Impact: a future reader trusting the 2026-07-22 audit verbatim would
  over-distrust an already-governed code path, or re-implement a fix that has
  already landed.
- Next move: re-verify the finding against `service.rs:258-350` and, if
  confirmed resolved, update or annotate the 2026-07-22 audit report.
- Scope: correcting a prior report is outside Task 1's grounding scope for the
  Workbench plan; filed here rather than edited in place.

## Open: `cell.migrate` naming collision with `sea_forge_cell::bundle` federation exchange

- Observed: 2026-07-24
- Evidence: `.agents/specs/frontend/sea-forge-workbench-api-spec-v0.1.md`
  Appendix A defines `cell.migrate` as "append a verified migration into a new
  or existing cell without rewriting prior history." The only existing
  cell-to-cell substrate is `sea_forge_cell::bundle::{export,import}`
  (`crates/sea-forge-cell/src/bundle.rs:29,135`), explicitly documented
  (`bundle.rs:1`) as *federation* bundle export/import, whose import path
  never merges into local `capabilities.jsonl`.
- Impact: if a future implementer aliases or merges `cell.migrate` onto
  `bundle::export/import` for convenience, migration and federation-exchange
  semantics blur, risking silent evidence/history handling differences.
- Next move: when Task 1's plan successors implement the `cell` family, build
  `cell.migrate` as new, distinct surface (see grounding report decision:
  `add`) and do not alias it onto the bundle federation path.
- Scope: this is a caution for later implementation tasks, not a defect to
  fix during Task 1's grounding pass itself.

## Open: Historical frontend planning documents fail repository-wide diff whitespace checks

- Observed: 2026-07-24
- Evidence: `git diff --check 02ed18d..HEAD` reports trailing whitespace in
  `.agents/plans/DEV_PLAN_TEMPLATE.md` and several
  `.agents/specs/frontend/*.md` files; `git diff --check` for Task 19's own
  changes is clean.
- Impact: broad historical-range reviews produce noisy whitespace failures and
  can hide a new formatting defect in the same output.
- Next move: normalize only those documentation files in a dedicated
  formatting commit after confirming intentional Markdown hard breaks.
- Scope: the files are pre-existing committed frontend/planning work unrelated
  to Task 19's conformance claims; changing them would expand this closeout.
