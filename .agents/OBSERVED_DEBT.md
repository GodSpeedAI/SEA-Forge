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

## Resolved (2026-07-26, Task 7): Host event-catch-up page cap is a separately hardcoded guess at the server's actual cap

**Fix.** `GET_RANGE_PAGE_CAP` was deleted rather than reconciled. The catch-up
drain in `workbench/apps/desktop/src-tauri/src/events.rs` now terminates on an
**empty** page instead of a short one, so the host encodes no assumption about
the server's page size at all — the duplication that could drift is gone rather
than documented. Cost is one extra round trip per reconnect.

A non-empty page that fails to advance the cursor would re-read forever, so that
case now returns a named error instead of spinning; a hung reconnect is harder
to diagnose than a reported one.

Proven by `catch_up_drains_until_a_page_is_empty_not_merely_short`
(`workbench/apps/desktop/src-tauri/tests/bridge.rs`), which scripts two
three-event pages before the empty one. Verified to have teeth: restoring the
short-page rule fails it with 3 of 6 events silently dropped.

Original entry follows.

## Open (superseded by the above): Host event-catch-up page cap is a separately hardcoded guess at the server's actual cap

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

## Resolved (2026-07-26, Task 7): `scripts/check-agent-context.sh` is unrunnable on this branch

**Fix.** Converted CRLF → LF and restored the executable bit (git mode is now
`100755`). `./scripts/check-agent-context.sh` executes and emits its real
diagnostic rather than `env: 'sh\r': No such file or directory`, so
`just context-check` is a live gate again instead of a step every task worked
around.

Original entry follows.

## Open (superseded by the above): `scripts/check-agent-context.sh` is unrunnable on this branch

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

## Resolved (2026-07-26, Task 7): Readiness event-invalidation is coarse (any `sfwp://event` invalidates every readiness view)

**Fix.** `workbench/apps/desktop/src/hooks/eventKinds.ts` names the three kinds
the server actually emits (`case.submitted`, `approval.approved`,
`approval.rejected`) and classifies them per surface. `useReadiness` now skips
frames known not to affect readiness; `useCases` and `useApprovals` use the same
taxonomy.

The narrowing is deliberately **asymmetric**: a kind is skipped only when known
irrelevant, and an *unrecognized* kind still invalidates. A future server event
therefore starts out over-invalidating (an extra read) rather than being
silently ignored (a stale render with no signal). The original entry's "once a
self-model event kind exists, narrow positively" plan was rejected for that
reason — an inclusion list fails closed on kinds it has never heard of.

The listener also reads `event?.payload` defensively: a throwing listener tears
down the whole subscription, so a malformed frame degrades to "invalidate
anyway" rather than to a dead stream.

Original entry follows.

## Open (superseded by the above): Readiness event-invalidation is coarse (any `sfwp://event` invalidates every readiness view)

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

## Open: `kill_9_leaves_a_valid_jsonl_prefix_without_capability_corruption` fails on a loaded machine

- Observed: 2026-07-26
- Evidence: fails at `crates/sea-forge-cli/tests/lifecycle.rs:1147` with
  "command should start before kill timeout" on a baseline
  `cargo test --workspace --no-fail-fast` (517 passed, 1 failed) taken
  *before* any edit in the transport-hardening change, and reproducibly on a
  loaded WSL2 host. The assertion is a wall-clock startup deadline, not a
  governance property.
- Impact: the crash-durability proof this test exists to give (a `kill -9`
  leaves a valid JSONL prefix and uncorrupted capability memory) is not
  actually being exercised when the deadline trips first, so a real
  regression in that path could hide behind an assumed-flaky failure.
- Next move: replace the fixed startup deadline with a readiness signal from
  the child process (poll for the run directory or a first JSONL line) so the
  test waits on the condition it means rather than on elapsed time.
- Scope: pre-existing and in `sea-forge-cli`; out of scope for a change
  confined to `sea-forge-server` transport and `sea-forge-cell` bundle reads.
- **Re-confirmed 2026-07-26 (Task 7), with a differential measurement.** It
  failed again during a `cargo test --workspace` run on a loaded host. Because
  Task 7 touched `sea-forge-cli/src/approvals.rs`, "assumed flaky" was not good
  enough — the entry's own Impact paragraph warns that a real regression could
  hide behind that assumption. Measured instead: a clean `HEAD` worktree
  **passed** while the working tree **failed**, which initially read as a
  regression. Re-running both on an idle host resolved it: the working tree
  then passed **8/8 consecutive runs**, and the first "isolated" failure had in
  fact overlapped the tail of the workspace run. So the failure tracks host
  load, not the diff. Recording the method because the cheap conclusion
  ("known flake") and the correct one differed by one measurement, and the
  differential worktree check is what separated them.

## Open: accept-loop resilience has no executed test

- Observed: 2026-07-26
- Evidence: `crates/sea-forge-server/src/lib.rs` `run()` now logs and continues
  (with a 100ms backoff) instead of propagating a failed `accept()`, but
  `crates/sea-forge-server/tests/conformance_transport_hardening.rs` only
  covers the weaker testable property — that connection churn does not
  terminate the server. Neither `EMFILE`/`ENFILE` nor `ECONNABORTED` is
  portably reproducible from a test, so the resilience branch itself is
  verified by inspection only.
- Impact: a future edit could turn the `continue` back into a `?` and every
  test would still pass; the daemon would then exit on the first transient
  accept error under descriptor pressure.
- Next move: inject the listener behind a trait, or drive the process to its
  descriptor limit in an isolated integration test, if the branch is ever
  judged worth a real test.
- Scope: recorded rather than covered by a test that would not actually
  exercise the branch it claims to.

## Partly resolved (2026-07-26, Task 7): three workbench surfaces still render copied mockup data

**Resolved for `cases` and `inbox`.** `CasesPage` and `InboxPage` now resolve to
`CaseHorizonPage` and `ApprovalInboxPage`, which read `case.list` /
`case.get_overview` / `case.get_horizon` and `approval.list` — all five methods
implemented this pass in `crates/sea-forge-server/src/sfwp/case_views.rs` and
`sfwp/approvals.rs`, with 13 conformance tests. The horizon renders execution
and settlement as separate pills sourced from disjoint server-side enums, so a
completed command can no longer read as accepted work.

**Still open for `thoth`, `assets`, and `domain models`** — see the original
entry below; `thoth.ask` still has no response contract, and `asset.list` /
`domain_model.list` have no SFWP method at all.

- Observed: 2026-07-26
- Evidence: `workbench/apps/desktop/src/pages/SurfacesPages.tsx` — `ThothPage`,
  `AssetsPage`, and `ModelsPage` hardcode every row they display (asset names,
  validation verdicts). Their evidence buttons emit `"display": "copied
  specification projection"` rather than a real record.
- Impact: bounded but real. Each carries a "Copied specification · not live"
  freshness badge and an `unknown` header pill, so the page does disclose that
  it is not live — but the *inner* pills (`Available`, `Validated`, `Ready`)
  are fabricated `ready` states with no such disclosure, and read as governed
  standing.
- Next move: these cannot be wired from the renderer. `thoth` needs a response
  contract for the `ask` verb (the `Ask` request variant exists at
  `crates/sea-forge-server/src/lib.rs`, but its response is untyped
  `serde_json::Value`); `assets` and `domain models` have no SFWP method at
  all. Each needs a server-side schema in
  `workbench/packages/contracts/schema/` first.
- Scope: the six surfaces that claimed a `ready`/"Active" pill over *no* data
  and *no* disclosure were replaced with `UnbackedSurface` in an earlier pass;
  `cases`/`inbox` were made real in Task 7. These three were left because
  inventing their response shapes is the failure mode the change set exists to
  remove.

## Open: `events.get_range` has no server-side tail read

- Observed: 2026-07-26
- Evidence: `crates/sea-forge-server/src/sfwp/events.rs` `get_range` applies
  `.take(cap)` after filtering, so an unpaged read of a long ledger returns the
  *earliest* 500 frames, never the most recent. The operations console
  therefore pages forward from its last cursor
  (`workbench/apps/desktop/src/hooks/useOperationsStream.ts`, `drainFrom`),
  capped at 20 pages.
- Impact: first load on a ledger with more than 10,000 events stops short of
  the tail, and the console shows a window that is behind the true head until
  a later read catches up.
- Next move: a `to_cursor`-anchored reverse read, or a `tail: bool` parameter
  on `events.get_range`, would make the first load O(1) instead of O(ledger).
- Scope: the renderer works within the method as it exists; changing the
  server's read semantics is out of scope for a frontend pass.

## Workbench surfaces outrun the kernel's method catalog

**Observed.** The workbench ships routes for eleven journeys the UX epic
defines, but `sfwp::IMPLEMENTED_METHODS` lists eleven methods total, covering
four of those journeys (negotiation, readiness, case authoring, operations).
The remaining surfaces have no method to read.

**Evidence.** `crates/sea-forge-server/src/sfwp/mod.rs:61` is the single source
of truth for `system.hello`'s `implemented_methods`. The target catalog in
`.agents/specs/frontend/sea-forge-workbench-api-spec-v0.1.md` (Appendix A)
names ~90 methods.

**Impact.** Bounded, not silent. Every surface now resolves its standing from
`system.hello` at runtime (`workbench/apps/desktop/src/hooks/useServerContract.ts`),
so an unbacked surface reports `unsupported` with the method it needs, and
`/admin` lists the whole negotiated catalog. No surface asserts availability on
the renderer's authority. The cost is reach, not honesty.

**Next move.** Implement methods server-side in journey order; the surfaces
change standing with no frontend edit.

**Update (2026-07-26, Task 7).** The catalog is now **sixteen** methods:
`case.list`, `case.get_overview`, `case.get_horizon`, `approval.list`, and
`approval.decide` were added, taking coverage to six journeys (negotiation,
readiness, case authoring, operations, case navigation, approvals). Cases and
Inbox graduated from specimen to real reads, which is the mechanism working as
designed — the surfaces changed standing because the kernel grew the verbs, and
`SURFACED_METHODS` was the only frontend edit needed.

Three specimen layouts remain (Thoth, Assets, Models) behind the
`data-specimen` watermark with every pill forced to `unknown`. They need
`thoth.ask` to gain a typed response contract, and `asset.list` /
`domain_model.list` to exist at all.

**Scope.** Frontend is complete against the kernel as it stands; the gap is
server-side method coverage.

## Resolved (2026-07-26, Task 7): `approval.decide` is reachable but not discoverable

**Observed.** `SfwpCommand::Approve` existed in the Tauri bridge, so an approval
could be *acted on*, but no method listed pending approvals — an approver had no
way to obtain the `case_id`/`approval_id` pair the command requires. The
capability was real and unreachable.

**Fix.** `approval.list` (`crates/sea-forge-server/src/sfwp/approvals.rs`) and an
`approval.decide` envelope over the existing `decide` path, both advertised
through `system.hello`/`system.describe`. `ApprovalInboxPage` renders the queue.

Notes on what the fix had to get right:

- **One owner for the fold.** The "latest record per `approval_id` wins" rule
  moved from `crates/sea-forge-cli/src/approvals.rs` into
  `sea_forge_core::approvals`; the CLI module is now a re-export. Two
  implementations of that fold could disagree about whether an approval is still
  open, and the one saying "open" would offer an operator a decision already
  made.
- **`approval.decide` forks nothing.** It routes to the same `decide` helper
  `Approve`/`Reject` use, keeping the precondition check and correlated-outcome
  recording on one path. An unrecognized verdict is refused
  (`error_class: invalid_decision`), never defaulted.
- **Expiry is reported, not hidden.** An expired approval stays listed with
  `expired: true` so a parked item remains explainable (invariant 7).
- **No optimistic removal.** A row leaves the inbox only when a re-read no longer
  returns it, so a refused decision cannot render as a successful one.

Proven by seven tests in `crates/sea-forge-server/tests/conformance_approvals.rs`
plus seven in `workbench/apps/desktop/src/pages/ApprovalInboxPage.test.tsx`.

## Run ids were displayed everywhere and resolvable nowhere

**Observed while implementing the run record (epic 12.1/12.3/12.4).**

`CaseOverview.run_ids`, `HorizonItem.run_ids`, `RunSettlement.run_id`, and every
`EventFrame.run_id` already carried run identifiers into the UI, but no SFWP
method could turn one into a record. The horizon rendered `Episodes: 2` — a
count, because a link had nowhere to point. That left epic invariant 4 (*every
important status resolves to committed source records*) unsatisfiable by
construction rather than merely unimplemented: no amount of frontend work could
have closed it.

**Fix.** `run.list`/`run.get` (`crates/sea-forge-server/src/sfwp/run_views.rs`),
a `/runs/$runId` route, and an evidence index backed by `run.list`.

Notes on what the fix had to get right:

- **The criteria pairing joins, it does not judge.** Re-evaluating settlement
  criteria in the view would have created a second settlement engine competing
  with `sea_forge_settlement::settle`. Instead the view reads the basis tokens
  that engine already writes (`exit_zero`, `required_artifact_missing:<path>`, …)
  and joins them to the declared criteria. A criterion whose deciding token is
  absent reports `unavailable`, never an assumed pass.
- **Evaluator scores stay `recorded`.** Per §10.6 a score is evidence, never
  standing, so it gets its own standing value rather than borrowing pass/fail.
- **Termination and settlement never merge.** `RunRecord.termination` is read
  from the trace, `RunRecord.settlement` from `settlement.json`, and the page
  renders them in two equal columns so neither reads as the headline fact.
- **Absence is inventoried.** `RunRecord.records` lists every canonical run file
  with whether it was present, so a missing `authority.json` reads as a missing
  record rather than as an empty panel.

Proven by nine tests in `crates/sea-forge-server/tests/conformance_run_views.rs`,
seven in `workbench/apps/desktop/src/pages/RunRecordPage.test.tsx`, one in
`CaseHorizonPage.test.tsx` (episodes link, not count), and one host-boundary
serialization test in `src-tauri/src/bridge.rs`.

### Still open after this change

- **`SCHEMA_TYPES` had drifted from the generator.** `sfwp::mod::SCHEMA_TYPES`
  claims to share its list with `gen_sfwp_schema.rs` "so they never drift", but
  the Task 7 case/approval types were only ever added to the generator, so
  `system.get_schema` under-reported by eight types. Both lists are now complete,
  but nothing *enforces* the claim — a test asserting `SCHEMA_TYPES` equals the
  generator's emitted names would.
- **The event taxonomy has no run-specific kinds.** `useRuns` reuses
  `affectsCases` because `KNOWN_EVENT_KINDS` still holds only three kinds, none
  run-scoped. Correct today (over-invalidating, never stale), but a
  `run.settled` / `run.started` kind would let run views narrow properly.
- **The operations event stream still does not link its run ids.** Each frame is
  a `<button>` opening an evidence drawer, so an anchor cannot nest inside it
  without restructuring a surface covered by a mockup-fidelity test. Deferred
  deliberately: the run record is already reachable from the horizon and the
  evidence index.
