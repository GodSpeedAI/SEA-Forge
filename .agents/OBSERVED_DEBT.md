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

## Open: Case-authoring proof scenarios (6, 7) have no Playwright e2e coverage

- Observed: 2026-07-26
- Evidence: Task 6's e2e proof scenarios ("ambiguous commit has no duplicate
  side effect", "stale preflight repairs via re-preflight") are proven at the
  Rust conformance level (`crates/sea-forge-server/tests/conformance_case_authoring.rs`)
  and the XState machine level (`workbench/apps/desktop/src/machines/caseAuthoringMachine.test.ts`),
  but not as a Playwright journey against the mocked-IPC harness
  (`workbench/apps/desktop/e2e/`) that Tasks 5/7's readiness/horizon specs use.
- Impact: no proof that the real `CaseCreationWorkbench` UI (not just the
  machine in isolation) correctly disables/re-enables its buttons and renders
  the repair/recovery affordances across these two scenarios end to end in a
  browser.
- Next move: either extend the mocked-IPC e2e harness with `case_preflight`/
  `case_commit` fixtures that can simulate a stale precondition and a dropped
  commit response, or decide (as this task's skill review note flagged for
  every prior authoring-adjacent task) that a real-server e2e harness is
  needed before trusting a mocked one for exactly this class of timing-
  dependent scenario, and build that harness once rather than per-task.
- Scope: building or deciding the real-server-vs-mock e2e harness question is
  a cross-cutting decision affecting Tasks 5–13 equally (already flagged in
  this file's task-review notes since Task 5); resolving it inside Task 6
  alone would be scope creep onto a decision the plan defers explicitly.

## Open: No visual-fidelity pass exists for the case-creation screen

- Observed: 2026-07-26
- Evidence: `.agents/specs/frontend/ui_kits/app/` has no `case-creation` or
  `case-authoring` reference page (checked: only `index.html` covering
  Readiness and the Operate-route shells exist). `CaseCreationWorkbench.tsx`
  was built to the wireframe/API-spec semantics only, styled with the same
  global panel/button classes the rest of the shell uses, with no mockup to
  diff pixel geometry against.
- Impact: the skill's design-fidelity workflow (§ "For any surface with a
  checked-in mockup, treat visual fidelity as part of the settlement") cannot
  be executed for this screen — there is no reference to render side by side.
- Next move: either author a `case-creation` static reference in
  `.agents/specs/frontend/ui_kits/app/` (design-system-skill work) before the
  next visual pass, or explicitly accept semantic-fidelity-only for this
  screen and drop it from the visual-fidelity checklist.
- Scope: authoring a new static mockup is `.agents/specs/frontend/SKILL.md`'s
  domain (design-system skill), not this implementation task's.

## Open: Static Workbench kit hides Operate-route CSS in reduced-motion media

- Observed: 2026-07-25
- Evidence: `.agents/specs/frontend/ui_kits/app/styles.css:998-1067` opens
  `@media (prefers-reduced-motion: reduce)` before the Operate-route variants
  and does not close it until after those selectors. In a normal-motion browser,
  `.asset-row` computes to `display: block` even though the source later declares
  a grid; the production browser regression now asserts `display: grid`.
- Impact: rendered reference routes appear compressed in the default browser
  mode, and a source-only fidelity pass can copy a media-query parsing defect
  instead of the intended route layouts.
- Next move: correct the brace boundary in a dedicated frontend-spec revision,
  then recapture normal/reduced-motion reference screenshots and update the
  source-projection contract deliberately.
- Scope: this slice preserves the checked-in reference byte-for-byte and
  projects the intended route selectors through production CSS; editing
  preserved specification evidence is a separate review decision.

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

## Open: `scripts/check-agent-context.sh` is unrunnable on this branch

- Observed: 2026-07-25
- Evidence: the file is git-tracked as mode `100644` (non-executable;
  `git ls-files -s scripts/check-agent-context.sh`) and has CRLF line endings
  (`env: 'sh\r': No such file or directory` when invoked after `chmod +x`).
  `devbox run -- just check` calls it via the `context-check` recipe
  (`justfile:104`) and fails with exit 126/127 before reaching `fmt-check`,
  `lint`, or `test`. Neither issue was introduced by Task 5 (`git log -1 --
  scripts/check-agent-context.sh` shows the last change predates this work).
- Impact: `devbox run -- just check` — the composite gate the skill and
  `AGENTS.md` name as required before declaring any slice done — cannot run
  end-to-end on this branch. Task 5 worked around it by running `fmt-check`,
  `lint`, and `test` individually (all green); every other task on this branch
  since the file broke has presumably done the same or silently skipped
  `context-check`.
- Next move: fix the file's line endings (convert to LF) and restore the
  executable bit (`chmod +x` + commit), then confirm `just check` runs clean
  end-to-end again.
- Scope: unrelated to `readiness.get`/`ReadinessConsole`; fixing it would
  touch a file no reviewer would expect in this slice's diff.

## Open: Readiness Playwright journey never talks to a real `sea-forge-server`

- Observed: 2026-07-25
- Evidence: `workbench/apps/desktop/e2e/readiness.spec.ts` +
  `e2e/tauriMock.ts` mock `window.__TAURI_INTERNALS__` entirely (no native
  Tauri shell, no Unix-socket connection); `sfwp_query` resolves to a
  hardcoded `degradedReadinessView()` fixture. The plan's Task 5 Definition of
  Done reads "the journey passes against a real server on a temp root" — that
  requirement is met at the Rust level (`conformance_sfwp.rs`'s
  `readiness_get_*` tests boot a real server via `boot()`/`boot_with_endpoint()`
  on a temp root) but not by the Playwright journey itself.
- Impact: the e2e layer proves the React/query/component wiring is correct
  against a known-good fixture, but does not prove the full stack (real
  server → real Tauri host → real socket → renderer) together in one test.
  A regression in the Tauri bridge's actual wire serialization (as opposed to
  the mocked shape) would not be caught by this suite.
- Next move: a true end-to-end run needs either a packaged Tauri binary (OS
  windowing not available in this environment) or a browser-mode build of the
  desktop app pointed at a real `sea-forge-server` process via
  `SEA_FORGE_SOCKET`, with the mock swapped for a real IPC bridge. Worth
  revisiting once the Workbench has a CI environment that can launch a
  browser + a real server together.
- Scope: accepted as a disclosed, pragmatic scope reduction for Task 5 in this
  environment; flagged rather than silently presented as full e2e coverage.

## Open: Readiness event-invalidation is coarse (any `sfwp://event` invalidates every readiness view)

- Observed: 2026-07-25
- Evidence: `workbench/apps/desktop/src/hooks/useReadiness.ts` invalidates
  `READINESS_QUERY_KEY` on receipt of any `sfwp://event` frame — there is no
  `self_model.changed`/`endpoint.updated`-style event kind anywhere in the
  server (verified by grep across `crates/sea-forge-server/src/sfwp/events.rs`
  and callers) to narrow against.
- Impact: harmless correctness-wise (over-invalidation just causes an extra
  refetch, never a stale render), but every case/run/approval event on a busy
  cell will also refetch readiness even when readiness genuinely didn't
  change, which won't scale well once event volume grows.
- Next move: once a self-model/endpoint-config change gets its own event
  `kind`, narrow the invalidation predicate in `useReadiness.ts` to match it.
- Scope: not fixable within Task 5 without inventing a server event taxonomy
  that doesn't exist yet — logged rather than silently narrowed by guessing.

## Open: `ReadinessView.recent_invalidations` has no real source and `overall: "stale"` is never derived

- Observed: 2026-07-25
- Evidence: `crates/sea-forge-server/src/sfwp/readiness.rs` ships
  `recent_invalidations` as an always-empty `Vec` and never constructs
  `OverallReadiness::Stale` — both are documented inline in the file's doc
  comments as intentional gaps (no cache/invalidation-record layer exists to
  source either honestly).
- Impact: the frontend correctly never implies these are populated (renders
  "No readiness invalidations recorded yet." rather than fabricating
  entries), but the spec's `ReadinessView` contract (which the frontend and
  any other future SFWP client will read) currently always has these two
  fields effectively dead, which a future consumer might not realize without
  reading this file.
- Next move: build a real invalidation-record source (likely tied to whatever
  emits the `self_model.changed`/`endpoint.updated` events from the item
  above) and a cache/freshness layer before deriving `stale` honestly.
- Scope: explicitly out of scope for Task 5 per the plan's own "ship the
  subset with explicit unknown sections" redesign-trigger guidance.
