# Incompleteness Inventory — SEA Forge

Census pass 1. Gaps classified **by user-visible or system-level consequence**, not effort. P0 blocks basic usability/correctness; P1 materially weakens the product; P2 important but not required for first usable product; P3 cleanup/optimization/optional.

Count: **0 P0**, **5 P1**, **9 P2**, **6 P3**.

---

## P0 — Blocks basic product usability or correctness

**None.** The product builds, all 909 tests pass (776 Rust + 133 TS), proofs are green, the server boots and answers SFWP requests, the CLI has 24 working subcommands, and the desktop renderer builds and runs. No correctness or basic-usability blocker was found.

---

## P1 — Materially weakens the intended product

### P1-API — SFWP method coverage at 17% (21 of 123)

- **Evidence**: `.agents/specs/frontend/sea-forge-workbench-api-method-catalog-v0.1.yaml` declares 123 target methods (status `target-unmapped`); `crates/sea-forge-server/src/sfwp/mod.rs:67-191` implements 21.
- **Impact**: Workbench surfaces for Thoth (live), Memory, Capabilities, Artifacts, Federation, and Admin currently render as "Specification preview · not live"; the production operator workflows behind those verbs are unreachable from the desktop. Backend logic exists for many of them (`sea-forge ask`, `sea-forge artifact`, `sea-forge export`, `sea-forge self-model`, `sea-forge memory`) but is not exposed over SFWP.
- **Next move**: continue the workbench plan's vertical-slice pattern (Tasks 12-14) to graduate the highest-value verbs (`thoth.ask`, `domain_model.list`, `memory.search`, `capability.list`, `artifact.list`).

### P1-E2E — No real-server Playwright e2e harness

- **Evidence**: `.agents/OBSERVED_DEBT.md` "Open: Case-authoring proof scenarios (6, 7) have no Playwright e2e coverage" and related entries since Task 5. `workbench/apps/desktop/e2e/tauriMock.ts` mocks `window.__TAURI_INTERNALS__` entirely.
- **Impact**: the mocked harness cannot honestly prove real IPC timing/delivery (e.g., stale-precondition rejection, duplicate-commit unreachable, event catch-up). These are proven at the Rust/XState level only.
- **Next move**: build a real-server e2e harness once (spins up `sea-forge-server` on a temp root, drives the actual Tauri host through real SFWP) rather than per-task mocked fixtures.

### P1-DOCS — `README.md` and `ARCHITECTURE.md` contradict observed reality

- **Evidence**: `README.md:217-226` "Project layout" lists 2 crates; `ARCHITECTURE.md:3` "Status: intended architecture, pre-implementation"; `crates/` actually has 22.
- **Impact**: new contributors and agents receive a misleading picture of the implementation surface; onboarding and routing decisions start from a false premise.
- **Next move**: refresh both files in one documentation commit. Preserve the spec-first narrative (the *spec* is still canonical); only update the "what is implemented today" sections.

### P1-WORKING — Untracked `working/face/` directory

- **Evidence**: `working/face/{ARCHITECTURE,DESIGN,DEV_PLAN,PRODUCT,design.sample,dump}.md` — 6 files, dated 2026-07-09, describing an unrelated "face" project. Not in `.gitignore`. Not a sea-forge surface.
- **Impact**: noise in repository discovery; future agents may attempt to integrate "face" concepts into sea-forge.
- **Next move**: confirm with the user whether to delete, move out of repo, or add to `.gitignore`.

### P1-PUSH — Local `frontend` branch is 5 commits ahead of `origin/frontend`

- **Evidence**: `git log --oneline origin/frontend..HEAD` shows 5 unpushed commits including the most recent DelegationRoster work and Case Creation Workbench. `main` is also present locally but stale relative to `frontend`.
- **Impact**: the latest two weeks of workbench work is not on the remote. Any CI run from the remote will not reflect the current state.
- **Next move**: confirm with the user whether to push, merge to `main`, or hold.

---

## P2 — Important but not required for the first usable integrated product

### P2-SPEC-EMPTY — Two empty placeholder spec files

- **Evidence**: `.agents/specs/api-routes.md` (0 bytes), `.agents/specs/datamodel.md` (0 bytes).
- **Impact**: future readers may assume these are authoritative sources; the live API/datamodel truth is scattered across `crates/sea-forge-core/src/types.rs`, `.agents/specs/frontend/sea-forge-workbench-api-spec-v0.1.md`, and the YAML method catalog.
- **Next move**: delete the placeholders, or fill them by extracting from the canonical sources.

### P2-SOCK — `sea-forge-server` socket path is independent of `SEA_FORGE_ROOT`

- **Evidence**: `crates/sea-forge-server/src/config.rs:24-26` `default_socket_path() = ".sea-forge/server.sock"`; the `root` and `socket_path` fields are independent. Setting only `SEA_FORGE_ROOT` moves cases/runs but leaves the socket at the CWD's `.sea-forge/`.
- **Impact**: operators expecting one env var to relocate the whole install get surprising behavior; smoke-test setup (this pass) hit this directly.
- **Next move**: either compose `socket_path` under `root` when relative, or document the two-knob setup explicitly in `--help` output (which the server lacks today).

### P2-NOCLI — `sea-forge-server` has no `--help` / no clap

- **Evidence**: `crates/sea-forge-server/src/main.rs:6-39` reads `SEA_FORGE_ROOT` env var + YAML; no CLI args parsed. `sea-forge-server --help` hangs (it starts the server).
- **Impact**: discoverability is poor; operators must read source to learn the config knob.
- **Next move**: add a `clap`-derive `Cli` even if it only prints config + socket path and exits, mirroring `sea-forge-cli`'s pattern.

### P2-ACP-REAL — Real ACP / SWE_SEED host release tests are `#[ignore]`

- **Evidence**: `crates/sea-forge-server/tests/conformance_m16.rs` has 12 portable tests + 2 `#[ignore]` real-host tests.
- **Impact**: the ACP driver and SWE_SEED reconciler are portable-coverage-green but have no end-to-end proof against a real Claude Code/Codex/SWE_SEED host.
- **Next move**: when operator-supplied config becomes available, run the release gates and flip `#[ignore]` off or convert to feature-gated tests.

### P2-MACOS — Seatbelt jail backend unverified on this host

- **Evidence**: Linux host; CI runs `macos-latest` separately. The Landlock backend is portable-tested.
- **Impact**: macOS users get a kernel that is green in CI but unverified locally.
- **Next move**: run `devbox run -- just ci` on a macOS host as the next platform verification.

### P2-CI-FILTER — CI does not gate the workbench frontend

- **Evidence**: `.github/workflows/ci.yml` runs `just context-check` + `fmt-check` + `lint` + `typecheck` + `security` + `test` + `build` + `no-async-kernel`. **None of these exercise `workbench/`**. `just workbench-check` exists but is not invoked by CI.
- **Impact**: a Tauri/React regression can land without CI catching it. The `workbench/AGENTS.md` documents the separation as deliberate (ADR-004), but with no automated remote gate the frontend relies entirely on local discipline.
- **Next move**: either add a CI job that runs `just workbench-check` on PRs touching `workbench/`, or explicitly accept the local-only discipline and document it.

### P2-FROZEN — Tauri host crate boundary has no automated gate

- **Evidence**: `.agents/OBSERVED_DEBT.md` "Open: Tauri host crate's Cargo workspace boundary has no automated gate". `workbench/apps/desktop/src-tauri/Cargo.toml` declares an empty `[workspace]` table; nothing prevents a future change from accidentally joining it to the root workspace.
- **Impact**: a regression here would silently violate the kernel async-isolation invariant (ADR-004).
- **Next move**: add a CI assertion (one-line `grep` on the Cargo.toml) that `[workspace]` remains empty.

### P2-KILL-FLAKE — `kill_9_leaves_a_valid_jsonl_prefix…` is host-load-sensitive

- **Evidence**: `.agents/OBSERVED_DEBT.md` and CURRENT_STATUS 2026-07-26: "tracks host load, not the diff"; the test once failed under load, then 8/8 consecutive passes on an idle host.
- **Impact**: occasional CI flake under contention; not a correctness bug.
- **Next move**: increase the test's robustness to scheduling jitter (longer grace, retry the read) or mark as `#[ignore]` on contention.

### P2-COLLISION — `cell.migrate` vs `bundle::export/import` naming collision

- **Evidence**: `.agents/OBSERVED_DEBT.md` "Open: `cell.migrate` naming collision with `sea_forge_cell::bundle` federation exchange".
- **Impact**: if a future implementer aliases one to the other, migration and federation-exchange semantics blur.
- **Next move**: when implementing the `cell` SFWP family, build `cell.migrate` as a distinct surface (do not alias).

---

## P3 — Cleanup, optimization, or optional refinement

### P3-FACE — `working/face/` content review

(Detail in P1-WORKING; classified P1 for noise impact, but the underlying decision is P3 cleanup.)

### P3-STALE-AUDIT — 2026-07-22 audit report contains a stale Thoth finding

- **Evidence**: `.agents/reports/2026-07-22-spec-implementation-audit.md` §3 finding 2 says Thoth `ask` "bypasses governance and leaves no required chain." `.agents/reports/2026-07-24-sfwp-grounding.md` confirms `sea_forge_thoth::service::ask` already commits the full ledger chain.
- **Impact**: a future reader trusting the older audit over-distrusts an already-governed path.
- **Next move**: annotate or supersede the 2026-07-22 audit report.

### P3-VISUAL — Case-creation screen has no static mockup

- **Evidence**: `.agents/OBSERVED_DEBT.md` "Open: No visual-fidelity pass exists for the case-creation screen"; `.agents/specs/frontend/ui_kits/app/` lacks a `case-creation` page.
- **Impact**: the skill's visual-fidelity workflow cannot be executed for that screen.
- **Next move**: author a static reference page or explicitly accept semantic-fidelity-only for this screen.

### P3-CSS — Static Workbench kit hides Operate-route CSS in reduced-motion media

- **Evidence**: `.agents/specs/frontend/ui_kits/app/styles.css:998-1067` opens `@media (prefers-reduced-motion: reduce)` before Operate-route selectors and never closes it cleanly; `.asset-row` computes to `display:block` instead of `grid` in normal-motion browsers.
- **Impact**: a source-only fidelity pass would copy the parsing defect.
- **Next move**: correct the brace boundary in a dedicated frontend-spec revision.

### P3-ROUTER-WARN — `router.tsx` Fast Refresh warning

- **Evidence**: `workbench/apps/desktop/src/router.tsx:35:10 react(only-export-components)` warning during `bun run check`.
- **Impact**: HMR for the root component may not hot-reload cleanly during dev.
- **Next move**: move the `RouterContextSearch` helper out of `router.tsx` into a separate file.

### P3-AUTH-SPEC — `.github/copilot-instructions.md` referenced but absent

- **Evidence**: `AGENTS.md:35-36` rule #3 invokes it "when present"; the file does not exist.
- **Impact**: inert rule; no behavior change.
- **Next move**: either drop the rule from `AGENTS.md` or create the file when Copilot is adopted.
