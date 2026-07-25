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

## Open: Tauri host crate's Cargo workspace boundary has no automated gate

- Observed: 2026-07-24
- Evidence: `workbench/apps/desktop/src-tauri/Cargo.toml` declares an empty
  `[workspace]` table to keep the Tauri host (and its `tokio` dependency) out
  of the root kernel Cargo workspace (`docs/decisions/ADR-004-workbench-stack.md`).
  `just no-async-kernel` only enumerates a fixed list of kernel crate names —
  it does not verify this boundary itself.
- Impact: a future edit that deletes the `[workspace]` table (e.g. during
  boilerplate cleanup) would silently pull `tokio` into scope of the root
  workspace's dependency graph, or break the build if the root workspace's
  explicit `members` list doesn't include the new path — neither failure mode
  is caught by an existing gate.
- Next move: add a `workbench-check` (or `no-async-kernel`) assertion that
  `cargo metadata --manifest-path workbench/apps/desktop/src-tauri/Cargo.toml
  --no-deps` reports a workspace root equal to that same path (i.e. it is its
  own workspace, not absorbed by the root one).
- Scope: out of scope for Task 2 (stack proof only); the fix belongs with
  whichever task next touches `justfile`'s quality gates.

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

## Open: Host event-catch-up page cap is a separately hardcoded guess at the server's actual cap

- Observed: 2026-07-24
- Evidence: `crates/sea-forge-server/src/sfwp/events.rs` defines
  `EVENTS_REPLAY_CAP = 500` as the server's actual `events.get_range` page
  size when no `limit` is requested. `workbench/apps/desktop/src-tauri/src/events.rs`
  defines its own `GET_RANGE_PAGE_CAP = 256` as the threshold its catch-up
  loop uses to decide "backlog exhausted" (`count < GET_RANGE_PAGE_CAP`), and
  never sends an explicit `limit` on its `events_get_range` calls, so it is
  really comparing against the server's true 500-cap while assuming a smaller
  number.
- Impact: currently harmless because 256 < 500 — the host's check is
  conservative, so a backlog page between 256 and 499 events just costs one
  extra harmless round-trip before the loop correctly terminates on the next
  (short) page; no event is ever skipped. But the two constants live in
  separate Cargo workspaces with no shared source, so if a future change
  raises the host's assumed cap above the server's actual cap (or the two
  drift for any other reason), the same "conservative by construction"
  argument no longer holds and termination could become premature.
- Next move: either have the host always pass an explicit `limit` matching
  (or below) whatever the server advertises via `system.describe`/`get_schema`,
  or document the coupling with a comment linking the two constants by file:line
  so a future editor of either one notices the other. A generated-contracts
  entry for `EVENTS_REPLAY_CAP` itself (rather than a magic number on each
  side) would remove the duplication entirely.
- Scope: not a defect surfaced by Task 3's own gates (all pass); a robustness
  follow-up for whichever task next revisits the host's event-reconnect loop.
