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

## Open: Unbounded whole-file reads remain in sea-forge-server beyond SUP-09c's audited sites

- Observed: 2026-08-15
- Evidence: while capping the audit's named view-read sites (SUP-09c), a
  `rg "fs::read|read_to_string"` sweep of the same crate found more unbounded
  whole-file reads: `sfwp/correlation.rs:278` (runtime state under a run dir),
  `sfwp/assets.rs:155`, `sfwp/case.rs:157` (template/config reads), `lib.rs:329`
  and `lib.rs:427` plus `case_dispatch.rs:49` (plan files), `delegation.rs:1384`
  (canonical transcript bytes), `transcript_seal.rs:96/103` (sealed key and
  ciphertext). The SUP-09c caps landed only in `sfwp/mod.rs`
  (`MAX_RECORD_BYTES`/`MAX_JOURNAL_BYTES` via `size_within_cap`) and its view
  consumers.
- Impact: same allocation-amplification class as SUP-09c for whichever of
  these paths read child-inflatable runtime files (notably `correlation.rs`
  and `delegation.rs`); the config/template readers are lower risk but
  unbounded all the same.
- Next move: extend the `size_within_cap` guard to the runtime-file readers
  first (`correlation.rs`, `delegation.rs`, `transcript_seal.rs`), then decide
  per-site for operator-config readers whether a cap or a documented trust
  boundary is the honest answer.
- Scope: the audit remediation plan scopes SUP-09c to its five enumerated
  sites; sweeping the rest of the crate is new surface belonging to the
  hardening sweep (Batch 9) rather than this slice.

## Open: Authoring status pill keeps "Preflight passed" in the stale state

- Observed: 2026-08-15
- Evidence: `workbench/apps/desktop/src/pages/CaseCreationWorkbench.tsx`'s
  pill label ternary tests `preflight?.ok` before `state === "rejected_as_stale"`,
  so after a successful preflight the pill reads "Preflight passed" even while
  the machine is `rejected_as_stale` (alert and "Re-run preflight" button do
  render correctly). Confirmed live in the agent-browser journey evidence
  (`.agents/reports/2026-08-15-case-authoring-agent-browser-e2e/
  journey7-rejected-as-stale.png`).
- Impact: the pill understates the stale rejection — the operator's most
  glanceable status element contradicts the alert beside it.
- Next move: order the label/variant ternaries by machine state first
  (`rejected_as_stale` → "Rejected as stale"/blocked, then `preflight?.ok`),
  and pin it in `CaseCreationWorkbench.test.tsx`.
- Scope: the e2e-debt slice this was found in covers evidence harness work;
  changing component render logic with its component test belongs to its own
  focused change.

## Open: Playwright readiness fixtures fail the current ReadinessView contract

- Observed: 2026-08-15
- Evidence: `workbench/apps/desktop/e2e/tauriMock.ts`'s
  `degradedReadinessView()` omits `next_lawful_action`, which the generated
  `ReadinessItem` schema now requires (`packages/contracts/schema/
  ReadinessItem.schema.json` required list). Validated directly:
  `validateReadinessView(readyReadinessView())` fails with
  "must have required property 'next_lawful_action'" per item, so
  `fetchReadiness` throws and the page renders "Readiness projection
  unavailable" — the ready/degraded journeys the specs assert cannot pass
  as written.
- Impact: the mocked Playwright readiness suite is red (or would be, next
  run) against the current contracts; anyone trusting it for renderer
  regression evidence gets a fail-closed page instead of the fixture view.
- Next move: add `next_lawful_action` to the fixture items and re-run
  `bunx playwright test e2e/readiness.spec.ts`; consider having the suite
  validate fixtures against the generated AJV validators so schema drift
  fails loudly at the fixture, not as a wrong-page assertion.
- Scope: discovered while building the agent-browser mock's readiness
  fixture (which validated correctly); fixing the Playwright suite is its
  own change with its own verification run.

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

## Resolved (2026-07-27, Task 9): `SCHEMA_TYPES` had drifted from the generator with nothing enforcing it

**Observed** at the end of Task 8: `sfwp::SCHEMA_TYPES` claims to share its list
with `gen_sfwp_schema.rs` "so they never drift", but the claim was only a
comment — and it had already been broken once (eight Task 7 types reached the
generator and never `system.get_schema`).

**Fix.** `generated_schemas_are_committed_and_current` in
`crates/sea-forge-server/tests/conformance_sfwp.rs` now also asserts
`SCHEMA_TYPES` equals the generator's emitted filenames. Adding a type to one
list and not the other fails the gate instead of silently under-reporting the
contracts a client can fetch.

## Resolved (2026-07-27, Task 9): the Assets surface fabricated an availability ladder the kernel refuses to let anyone assert

**Observed.** `SurfacesPages.tsx`'s `AssetsPage` hardcoded three rows with
`Available` / `Installed` / `Declared` pills. Meanwhile
`AgentEndpointConfig::validate` rejects an endpoint whose configuration asserts
its own `status` — "status is evidence-derived" — because that ladder may only
be climbed by committed probe records. The renderer was asserting exactly the
thing the kernel refuses to accept from configuration.

**Fix.** `asset.list` (`crates/sea-forge-server/src/sfwp/assets.rs`) and
`pages/AssetCatalogPage.tsx`. Endpoint standing is now folded from the probe
runs `agent_probe::probe` writes, with the *most recent* probe deciding.

## Open: `agent_list` still reports a hardcoded `declared` status and hides misconfigured endpoints

- Observed: 2026-07-27 (while implementing `asset.list`)
- Evidence: `crates/sea-forge-server/src/agent_probe.rs` — `list()` sets
  `status: "declared"` for every endpoint unconditionally, and
  `.filter_map(|endpoint| endpoint.snapshot().ok())` drops any endpoint whose
  configuration will not snapshot.
- Impact: bounded. `declared` is the honest *floor*, so nothing is overstated —
  but a demonstrated endpoint reads the same as an unprobed one, and a
  misconfigured endpoint vanishes from the listing rather than reporting why.
  The workbench no longer uses this verb (it reads `asset.list`), so the
  exposure is CLI and any direct socket client.
- Next move: `agent_list` could delegate to `sfwp::assets::collect_endpoints`,
  which already derives both. Left alone here because changing a shipped verb's
  response shape is not additive and needs its own ADR-003 pass.

## Open: nothing ties `SURFACED_METHODS` to the methods hooks actually call

- Observed: 2026-07-27
- Evidence: `run.list` and `run.get` were implemented, wired into
  `useRuns.ts`, and rendered by `RunRecordPage`/`EvidencePage` in Task 8 — but
  were never added to `SURFACED_METHODS`
  (`workbench/apps/desktop/src/hooks/useServerContract.ts`). For a full release
  cycle `/admin` reported two live, operator-reachable methods as "implemented,
  not surfaced". Added in this pass along with `asset.list`.
- Impact: the error is one-directional and quiet. A missing entry *understates*
  reach, which is the safe direction, but it makes the admin catalog wrong and
  the mistake is invisible — exactly the drift `SURFACED_METHODS` exists to
  prevent for the *other* list.
- Next move: derive the set from the `queryGoverned` call sites (each already
  passes its dotted method name as the third argument) or assert the
  correspondence in a test that greps the hooks directory.
- Scope: recorded rather than fixed structurally; the immediate wrongness is
  corrected.

## Open: endpoint standing costs a full scan of `<root>/runs`

- Observed: 2026-07-27
- Evidence: `sfwp::assets::latest_probe_per_endpoint` reads `plan.json` for
  every run directory to find the probe episodes. `run_views::list` already
  does the same, so the cost is precedented, not new.
- Impact: none today (a cell has tens of runs). On a cell with tens of
  thousands, opening the asset catalog becomes an O(ledger) read.
- Next move: a per-endpoint index written at probe time, or a `runs` index
  keyed by operation kind. Deliberately not built now — no cell is near the
  size where it matters, and an index is a second truth to keep consistent.
- Scope: the projection is short-circuited when no endpoint is configured, so
  the scan never runs on a cell with nothing to attribute.

## Open: templates are now projected by two methods with two shapes

- Observed: 2026-07-27
- Evidence: `case.entry_options` returns `TemplateOption` (template_ref,
  description, parameters — what you need to *instantiate* one); `asset.list`
  returns an `AssetRow` for the same file (identity digest, standing, blocking
  reason — what you need to *judge* one).
- Impact: none yet. The two answer different questions and neither derives the
  other, so this is not duplication so much as two projections of one source.
  It becomes debt if a third reader appears, or if the parse rules diverge —
  note that `entry_options` silently skips an unparseable template while
  `asset.list` reports it, which is already a small divergence in *policy*
  rather than in code.
- Next move: if a third reader lands, extract one template-enumeration helper
  that both project from; do not merge the response shapes.

## Open: the asset catalog abandoned its mockup layout without a fidelity pass

- Observed: 2026-07-27
- Evidence: `pages/AssetCatalogPage.tsx` renders three kind-scoped tables. The
  checked-in kit's assets screen (`asset-catalog-table`, `asset-detail`,
  `asset-boundary` regions) is one merged table plus a detail panel.
- Impact: the departure is deliberate and load-bearing — one merged table puts
  three disjoint standing vocabularies in one column and invites reading them
  as one ladder — but it means the assets route now has no visual-fidelity
  coverage and no `data-od-id` correspondence to the kit.
- Next move: either update the kit's assets screen to the three-table shape and
  add a fidelity check, or record the divergence in
  `.agents/specs/frontend/DESIGN-spec-mapping.md`.
- Scope: correctness and a11y are covered by
  `pages/AssetCatalogPage.test.tsx`; only visual fidelity is uncovered.

## Resolved (Task 10): `delegate` ignored endpoint and cell transcript retention

- Observed: 2026-07-27 · Resolved: 2026-07-27
- Evidence: `delegate_inner` built its `DelegationRequest` with
  `..Default::default()`, so `transcript_retention` was always
  `TranscriptRetentionMode::Summarized` — despite the field's own doc comment
  ("already resolved through the full precedence chain by the caller") and
  despite `case_dispatch` resolving it correctly for planned agent tasks.
- Impact (while open): an endpoint configuring `transcript_retention: full`, or
  a cell configuring it globally, was silently overridden for every delegation
  issued over the socket. The plan item recorded `summarized`, so the record was
  internally consistent and the discrepancy left no trace.
- Fix: `delegate_inner` now calls `TranscriptRetentionMode::resolve(None,
  endpoint, agent)` — the same chain `case_dispatch` uses. Covered by
  `transcript_retention_is_resolved_through_the_endpoint_then_the_cell`.

## Open: `delegation.preview` reports authority but cannot dry-run it

- Observed: 2026-07-27
- Evidence: `sfwp::delegation_preview` names the authority action kind
  (`agent_task`) and its digest, but performs no `PolicyAuthorityEngine`
  evaluation. `delegation::execute_with_permission_broker` evaluates only after
  committing intent, plan, and criteria to the ledger.
- Impact: an operator can assemble a job contract that is fully eligible and
  still be denied the moment they commit, with no way to find out first. The
  deferral is deliberate — a verdict with no ledger entry behind it is an
  unrecorded grant, and `evaluate` needs a committed case/run/item identity the
  preview does not have — but the gap is real and will be felt as soon as
  `/delegate` grows its command half.
- Next move: if this is closed, it needs a *governed* shape — a decision the
  kernel records as a preview decision, not a verdict computed and thrown away.
  Do not add a bare `would_be_allowed: bool`.

## Open: `delegate` has no precondition field, so `contract_digest` is advisory

- Observed: 2026-07-27
- Evidence: `JobContractPreview::contract_digest` identifies the exact contract
  inspected, and `sfwp::precondition` already exists (Task 6 used it for
  `case.commit`). But `Request::Delegate` has no `preconditions` field, so
  nothing can be enforced against the digest.
- Impact: an operator can inspect a contract, have the endpoint descriptor
  change underneath them, and commit a delegation that differs from what they
  approved. The digest lets a careful operator *notice*; it cannot make the
  kernel refuse.
- Next move: when `/delegate` gains its command half, add
  `preconditions: Option<Precondition>` to `Delegate` and reject on mismatch the
  way `case.commit` does — same `rejected_as_stale` body, no forked logic.

## Open: `delegation.preview` costs a full asset-catalog build per call

- Observed: 2026-07-27
- Evidence: `delegation_preview::preview` calls `assets::list(root, agent)` and
  keeps one row. `assets::list` walks `<root>/runs/*` (already logged as an
  O(runs) scan), reads every template, and loads the extension registry.
- Impact: negligible today; a cell with thousands of runs makes an interactive
  preview slow, and the page reads it on every explicit inspect.
- Next move: the fix is *not* to re-derive standing in the preview — the whole
  point is that the two agree. Narrow `assets` to expose a single-endpoint
  projection that `list` also uses, so there stays exactly one derivation.

## Open: `delegate` cannot override transcript retention per request

- Observed: 2026-07-27
- Evidence: `Operation::AgentTask` carries a `transcript_retention` override and
  `TranscriptRetentionMode::resolve` honours it, but `Request::Delegate` has no
  field for one, so the item-override rung of the precedence chain is
  unreachable over the socket. `DelegationPreviewParams` therefore omits it too.
- Impact: an operator delegating an unusually sensitive task cannot ask for
  `summarized` against an endpoint configured `full` without editing
  `server.yaml`. `ValueSource::Requested` is consequently unreachable for
  retention (it is reachable for model).
- Next move: additive field on `Delegate` plus the matching preview param; the
  resolution call already accepts the override, so this is wiring, not logic.

## Resolved (Task 11): the frontend event taxonomy had drifted from the server

- Observed: 2026-07-27 · Resolved: 2026-07-27
- Evidence: `hooks/eventKinds.ts` declared `KNOWN_EVENT_KINDS` as three kinds
  and described them as "verified against its publish sites". The server's
  `publish_event` call sites emit five: `case.submitted`, `approval.approved`,
  `approval.rejected`, `agent_run.delegated`, `agent_run.cancellation_requested`.
- Impact (while open): none visible, and that is the point. The narrowing is
  deliberately asymmetric, so the two unlisted kinds over-invalidated every
  surface instead of being ignored. The safety property worked exactly as
  designed and therefore hid the drift — no surface could go stale, so nothing
  ever surfaced the omission.
- Fix: all five listed and classified, plus `affectsDelegations`. New
  `hooks/eventKinds.test.ts` pins the roster against a transcribed list of the
  server's publish sites and asserts the asymmetry directly.
- Residual: the test's server-side list is transcribed by hand, so it catches
  drift only when someone updates one side. A generated event-kind contract
  (the same Rust → schemars → TS path the view types already use) would make
  this structural. Logged separately below.

## Resolved (Task 11): run-directory enumeration reached a third copy

- Observed: 2026-07-27 · Resolved: 2026-07-27
- Evidence: the prior entry on this predicted it — `run_views::list`,
  `assets::latest_probe_per_endpoint`, and the new `delegations::list` all
  walked `<root>/runs/*` by hand.
- Fix: `run_views::run_dirs` and `case_index` are now `pub(crate)` and used by
  all three. Deliberately *not* parameterised: each caller keeps its own
  readability policy (`run.list` reports a run with neither trace nor
  settlement as unreadable; `assets` skips silently; `delegations` filters to
  agent tasks). The shared helper decides only what to look at.

## Resolved (Task 11): a Task 10 test assertion was vacuous

- Observed: 2026-07-27 · Resolved: 2026-07-27
- Evidence: `preview_creates_no_run_case_or_ledger_entry` asserted that
  directories named `runs`, `cases`, and `ledger` stayed empty. The kernel's
  path is `ledgers`; `ledger` never exists, so that third of the assertion
  passed unconditionally and would have passed for any implementation.
- Impact (while open): the "no ledger entry" half of the claim in that test's
  own name was never actually tested. The `runs`/`cases` halves were real.
- Fix: both that test and `listing_the_roster_creates_nothing` now capture the
  whole cell tree before and after the read and compare. A name the author did
  not think of can no longer produce a silent pass.

## Open: no event-kind contract ties the frontend taxonomy to the server

- Observed: 2026-07-27
- Evidence: `KNOWN_EVENT_KINDS` and the server's `publish_event` call sites are
  two independent hand-maintained lists. The new `eventKinds.test.ts` compares
  the frontend list against a *transcription* of the server's, not against the
  server.
- Impact: adding a kind server-side still silently over-invalidates until
  someone notices — which, as the resolved entry above shows, may be a long
  time.
- Next move: emit the kind list from Rust through the existing schemars → TS
  contract pipeline (an `EventKind` enum would be the natural shape), then
  assert against the generated constant instead of a transcription.

## Open: the kernel exposes no live turn, token, or streaming state

- Observed: 2026-07-27
- Evidence: `DelegationHandle` carries `case_id` and two `AtomicBool`s and
  nothing else; the delegation loop publishes no per-turn event. `turns_used`
  and `tool_calls` first exist when `transcript-evidence.json` is written at
  termination.
- Impact: epic 9.7 asks for bounded turn, token, streaming, permission, and
  continuation state *during* the dialogue. `delegation.list` can only report
  those after the fact, so the roster shows "not observable while running"
  against the plan's cap. This is the unsettled half of 9.7 — the control half
  (enumerate, cancel one) is done.
- Next move: kernel-side first. A per-turn `agent_run.turn` event (or a
  counter on the handle) is the prerequisite; no frontend work can close this.
  Do not synthesise a count from elapsed time or from the transcript file's
  presence — a guessed turn count against a hard cap is worse than an absent one.

## Open: an in-flight delegation is never reconciled after a server restart

- Observed: 2026-07-27
- Evidence: `recover_cancelled_delegations` runs at `ServerState::new` but only
  iterates `control_request` ledger entries — i.e. delegations someone had
  asked to cancel. A delegation that was simply running when the server stopped
  has no control request, so nothing reconciles it; it keeps a `plan.json` and
  no `settlement.json` forever.
- Impact: `delegation.list` reports it as `unresolved`, which is honest but
  permanent. The cell accumulates delegations whose outcome nobody will ever
  determine, and there is no path to settle or formally abandon one.
- Next move: extend startup recovery to settle *any* unsettled delegation run
  with a typed `interrupted` termination and a basis naming the restart. That
  is a governed settlement, not a projection change — the roster should keep
  reporting whatever the records say.

## Open: `delegation.list` walks every run directory

- Observed: 2026-07-27
- Evidence: `delegations::list` calls `run_dirs(root)` and reads `plan.json`
  for every run to find the agent tasks; `assets::list` does the same for
  probes. Both are O(runs) per call, and `/delegate` now issues both.
- Impact: negligible today; a cell with thousands of runs makes an
  event-invalidated roster expensive, and this view refetches on every
  `agent_run.*` frame.
- Next move: an index keyed by operation kind, written when a run is committed.
  Do not cache the projection itself — the roster's whole value is that it
  reflects the records rather than a remembered summary.

## Open: `useRuns` invalidates on the case predicate

- Observed: 2026-07-27
- Evidence: `hooks/useRuns.ts` passes `affectsCases` to
  `useGovernedEventInvalidation` against `RUNS_QUERY_KEY`. There is no
  `affectsRuns`.
- Impact: correct but imprecise — run views refetch on approval decisions that
  cannot change them. Harmless, and it fails in the safe direction.
- Next move: add `affectsRuns` alongside the others when a run-specific event
  kind exists to narrow against. Not worth a predicate today, since the two
  `agent_run.*` kinds are already the only runs-relevant ones and both are in
  the case set.

## Open: request-less protected mutations have no idempotency (F-25.n)

- Observed: 2026-08-23 (audit finding F-25.n; verified against the current
  correlation store)
- Evidence: `sfwp::correlation` dedupes only requests carrying `request_id`;
  `dedupe_key` returns `None` for the rest, so a replayed `submit` without an
  id creates a second case.
- Impact: latent. The machinery itself is sound (atomic per-id records,
  fail-closed hash-mismatch refusals); the gap is exactly "protected verb +
  no id".
- Next move: requiring `request_id` on all protected verbs is a wire-contract
  change needing owner approval; deriving a server-side synthetic key changes
  observable semantics. Trigger: the first external multi-client SFWP
  consumer. Until then, document "send `request_id` for exactly-once
  mutations" in client guidance.

## Open: CEP-0008 inbound envelopes carrying `causation_id` cannot deserialize

- Observed: 2026-08-23 (SUP-09i remainder)
- Evidence: `cep0008.rs` struct uses `deny_unknown_fields` and omits
  `causation_id`, while the copied schema's `ALLOWED_TOP_LEVEL` includes it.
  The adapter is outbound-only today, so nothing deserializes untrusted
  CEP-0008 input yet.
- Next move: when an inbound consumer appears, add
  `#[serde(default, skip_serializing_if = "Option::is_none")]
  causation_id: Option<String>` — that keeps serialized bytes stable.

## Open: dead CEP/extension spec surface (`Compatibility`, `ExtensionInstallRecord`)

- Observed: 2026-08-23 (SUP-09i remainder)
- Evidence: declared in `sea-forge-extension`, zero constructors repo-wide;
  version-compare enforcement therefore absent by absence of callers.
- Next move: delete-vs-implement is a public-interface decision for the owner;
  revisit with the first extension-distribution feature.

## Open: true RFC 8785 JCS compliance would shift every persisted hash

- Observed: 2026-08-23 (SUP-09d remainder)
- Evidence: the consolidated `jcs-nfc-v1` profile in
  `sea_forge_core::canonical` is deliberately *not* RFC 8785 (no UTF-16 key
  ordering, no ECMAScript number canonicalization); the label is hashed into
  committed ledger records, so renaming/re-profiling breaks verification.
- Consolidation note: pre-fix copies diverged on nested-value NFC (ledger/
  self-model normalized only top-level strings; evidence/authority recursed).
  The shared primitive now recurses at all depths. All kernel-committed data
  is ASCII today, so historical hashes are byte-stable in practice; if any old
  ledger entry ever carried decomposed non-ASCII nested strings, the F-25.q
  append-time predecessor check now refuses loudly instead of silently
  diverging.
- Next move: a real JCS profile needs a new `canonicalization` label plus a
  migration story — an owner-level spec decision.

## Open: ComposedModel overlay precedence is undefined (SUP-09e remainder)

- Observed: 2026-08-23
- Evidence: `ComposedModel::dual_declared_concepts()` now discloses the 30
  dual-declared names (allowlist-pinned test), but which constituent's
  definition wins on conflict is still unspecified.
- Next move: owner decision (precedence, merge, or rejection on divergence) in
  spec-adlc-thoth terms before any release ships diverging definitions.
