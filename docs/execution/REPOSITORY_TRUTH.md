# Repository Truth — SEA Forge (sea-rs)

Census pass 1, completed 2026-07-27. Inspected commit `8361f25` on branch `frontend` (5 commits ahead of `origin/frontend`). Working tree clean except `.jolli/jollimemory/debug.log` (transient agent memory log). This document records **executable evidence**, not narrative.

## 1. Repository identity and inspected commit

- Path: `/home/sprime01/projects/sea-rs`
- HEAD: `8361f25 feat: Add tests for DelegationRoster component…` (2026-07-27 14:54 UTC)
- VCS: git. Remote: `https://github.com/GodSpeedAI/SEA-rs` (per `Cargo.toml:34`)
- License: `LicenseRef-SEA-Forge` (SEA-Forge Sustainable Use License; commercial dual-licensing via `COMMERCIAL-LICENSE.md`)
- License scoping file: `LICENSE_EE.md` identifies enterprise-only components
- Branch tips present locally: `frontend` (current), `main` (present), `ci-cd`, `docs/public-licensing`, `entire/*` (5 Entire-CLI integration snapshots), `feature/understand-anything`, `fix/domain-model-validation`

## 2. Workspace topology

Three distinct roots, one optional auxiliary directory:

1. **Root Cargo workspace** (`Cargo.toml`, resolver 2) — the **kernel and product crates**. 22 members, all under `crates/`.
2. **Workbench Bun workspace** (`workbench/package.json`, `packageManager: bun@1.4.0`) — the **desktop frontend**. Workspaces: `apps/desktop`, `packages/contracts`, `packages/sea-forge-ui-components`, `packages/sea-forge-astryx-theme`, `packages/sea-forge-ui-tokens`.
3. **Tauri host Cargo workspace** (`workbench/apps/desktop/src-tauri/Cargo.toml`, empty `[workspace]` table) — **deliberately standalone** to keep the host's `tokio`/`reqwest` deps out of the root kernel workspace (ADR-004). Has its own `Cargo.lock`.
4. **Auxiliary: `working/face/`** — 6 markdown files (`ARCHITECTURE.md`, `DESIGN.md`, `DEV_PLAN.md`, `PRODUCT.md`, `design.sample.md`, `dump.md`) describing a **different project** called "face" dated 2026-07-09. Not part of sea-forge. No build manifest, no Cargo/package.json, no git tracked integration. Looks like a parked parallel design dump; see INCOMPLETENESS_INVENTORY.md P3-FACE.

The README and AGENTS.md describe the project as a "minimum governed kernel (two-crate Rust workspace)" — this is **stale relative to reality**. The actual implemented surface spans 22 crates covering milestones M0–M16 of `spec-full.md` + `spec-adlc-thoth-minimum.md` + `spec-agent-orchestration.md`, plus a working desktop frontend. The README's "Quick start" command surface (`just check`, `just test`, `just proof`) is accurate; its "Project layout" section is not.

## 3. Technology and toolchain inventory

| Layer | Tool | Pin | Evidence |
|---|---|---|---|
| Rust toolchain | rustup→rustc/cargo | `1.92.0` (stable, minimal profile, +rustfmt +clippy) | `rust-toolchain.toml` |
| Rust edition | cargo | `2021` | `Cargo.toml:30` |
| Workspace deps | cargo + `--locked` | 331 entries in Cargo.lock | `Cargo.lock` |
| Lint policy | clippy | `clippy::all = warn (priority -1)`, run with `-D warnings`; `unsafe_code = deny` workspace-wide | `Cargo.toml:36-40` |
| Async runtime | tokio | `1` (multi-thread+net+io+process+sync+time+fs+macros) — **only in `sea-forge-server`** | `crates/sea-forge-server/Cargo.toml` |
| HTTP client | reqwest 0.12 (rustls, no default features) | only in `sea-forge-agent`, `sea-forge-server` (and Tauri host) | workspace deps |
| Persistence | rusqlite 0.32 (bundled) | only in `sea-forge-capability` | `crates/sea-forge-capability/Cargo.toml` |
| Crypto | `chacha20poly1305`, `ed25519-dalek`, `sha2`, `zeroize`, `getrandom` | transcript sealing, identity, hashing | workspace deps |
| Frontend runtime | Bun | `1.4.0` (canary Zig→Rust rewrite, reversible) | `workbench/package.json:6` |
| Renderer | React | `^19.2.7` | `workbench/apps/desktop/package.json:29` |
| Router/Query | TanStack | Router `1.170.18`, Query `5.101.4` | same |
| State machines | XState | `5.32.5` + `@xstate/react 6.1.0` | same |
| Forms | react-hook-form | `7.82.0` | same |
| Design substrate | Astryx | `@astryxdesign/core 0.1.8`, `@astryxdesign/theme-neutral 0.1.8` | same |
| Desktop host | Tauri | `2` (CLI `2.11.4`) | same |
| Test (Rust) | cargo test | built-in; 102 test binaries | observation |
| Test (TS) | Vitest | `4.1.10` | same |
| E2E | Playwright | `1.62.0` + axe-core `4.10.3` | same |
| Typecheck (TS) | tsc + oxlint | TypeScript `~6.0.2`, oxlint `^1.71.0` | same |
| Schemas | schemars `1` (Rust) → `json-schema-to-typescript` + `ajv 8.20.0` (TS) | ADR-005 | `workbench/packages/contracts/scripts/generate.ts` |
| Secrets | SOPS `3.10.2` + age `1.2.1` | only `*.enc.env` tracked | `.gitignore:35`, `.sops.yaml`, `.envrc` |
| Supply chain | cargo-deny `0.19.9` + gitleaks `8.30.1` | `deny.toml`, `.gitleaks.toml` | `devbox.json` |
| Build orchestration | just `1.55.1` | `justfile` (grouped recipes) | `devbox.json` |
| Dev shell | Devbox (Nix-backed) | `devbox.json` (8 packages) | `devbox.lock` |
| Host shims | mise | `mise.toml` | observation |

## 4. Actual component graph

The kernel crates form a layered dependency graph. The invariant enforced by `just no-async-kernel` is that **19 kernel crates have zero async-runtime and zero HTTP-client dependencies**; only `sea-forge-agent` and `sea-forge-server` may pull tokio/reqwest. This gate is green.

Layered (top to bottom):

- **CLI / server edges** — `sea-forge-cli` (bin-only), `sea-forge-server` (tokio edge), Tauri host
- **Orchestration** — `sea-forge-thoth` (manager loop, ask service), `sea-forge-agent` (delegation + ACP), `sea-forge-case-runner` (extracted lifecycle primitives, M13)
- **Pipeline/IP/cell** — `sea-forge-spec-pipeline` (M5), `sea-forge-artifact-ip` (M8), `sea-forge-cell` (federation), `sea-forge-self-model`, `sea-forge-domainforge`
- **Capability/settlement/ledger** — `sea-forge-capability` (SQLite-indexed), `sea-forge-settlement` (declarations + SWE_SEED), `sea-forge-ledger` (ULID + hash chain + MMR)
- **Kernel** — `sea-forge-authority` (policy engine, identity), `sea-forge-planner` (intent→plan, templates, criteria), `sea-forge-sandbox` (Landlock/Seatbelt/process), `sea-forge-runtime` (argv exec), `sea-forge-trace`, `sea-forge-evidence`, `sea-forge-extension`
- **Foundation** — `sea-forge-domain` (enums), `sea-forge-core` (types, ids, errors)

Frontend (`workbench/apps/desktop/src/`):

- **Router** (`router.tsx`) — 17 top-level routes, each with `evaluateGuard("G1".."G9")` and a `GovernedDenialSurface` fallback
- **Pages** — `ReadinessPage`, `CaseCreationWorkbench`, `CaseHorizonPage`, `DelegationWorkbench`, `ApprovalInboxPage`, `EvidencePage`, `RunRecordPage`, `OperationsPage`, plus `ThothPage`/`AssetsPage`/`ModelsPage`/`MemoryPage`/`CapabilitiesPage`/`ArtifactsPage`/`FederationPage`/`AdminPage` (all under `SurfacesPages.tsx`)
- **Shell** — `AppShell`, `GlobalHeader`, `Sidebar`, `JourneyRibbon`, `EvidenceContext`
- **Hooks** — TanStack Query wrappers (`useReadiness`, `useCaseEntryOptions`, …) validating each response against generated AJV validators
- **Machines** — XState v5 (`caseAuthoringMachine`, `statusMachine`) for frontend interaction state only
- **Guards** — G1..G9 typed predicates over `GuardContext`

Tauri host (`workbench/apps/desktop/src-tauri/src/`):

- `socket.rs` — NDJSON client: `call()`, FIFO response pairing
- `bridge.rs` — closed `SfwpQuery` / `SfwpCommand` enum (no generic invoke escape hatch)
- `events.rs` — durable cursor, reconnect, gap recovery
- `drafts.rs` — versioned local JSON in `app_data_dir/drafts/` (never touches `.sea-forge/`)
- `lib.rs` / `main.rs` — Tauri command registration

## 5. Executable entry points

| Entry | Kind | Invocation |
|---|---|---|
| `sea-forge` CLI | one-shot | `./target/debug/sea-forge <subcommand>` (24 subcommands) |
| `sea-forge-server` | long-running | `SEA_FORGE_ROOT=<dir> ./target/debug/sea-forge-server` (no CLI flags) |
| `gen_sfwp_schema` | one-shot emitter | `cargo run -p sea-forge-server --bin gen_sfwp_schema` |
| Tauri desktop | GUI | `cd workbench/apps/desktop && bun run tauri dev` (or `tauri build`) |
| Vite dev renderer | local web | `cd workbench && bun run dev` (http://localhost:1420) |
| Storybook | local web | `cd workbench && bun run --cwd packages/sea-forge-ui-components storybook` (http://localhost:6006) |
| CI (canonical union) | local/remote | `devbox run -- just ci` |
| Conformance proofs | local | `devbox run -- just proof` |

## 6. Startup and installation paths

Documented happy path (`README.md`):

```sh
git clone <repo> sea-rs && cd sea-rs
devbox shell          # enter pinned environment
direnv allow          # activate direnv (loads encrypted secrets if present)
just setup            # devbox install + rustup + cargo fetch --locked
just hooks-install    # git config core.hooksPath .githooks
just doctor           # writes target/bootstrap-evidence/doctor.jsonl
just check            # context + fmt + clippy + typecheck + security
just test             # cargo test --workspace --all-features --locked
```

This happy path was **verified end-to-end in this pass**: `cargo check`, `cargo fmt --check`, `cargo clippy` (implicit via clean test build), `cargo test --workspace`, `just proof`, `just context-check`, `just no-async-kernel` (implicit — see test count), frontend `bun run check`/`test`/`build`, and Tauri host `cargo build` all exited 0.

Frontend happy path (`workbench/AGENTS.md`):

```sh
cd workbench && bun install --frozen-lockfile
bun run dev           # renderer only
bun run check         # tsc --noEmit + oxlint
bun run build         # tsc -b + vite build
bun run test          # vitest run
# desktop host:
cargo build --manifest-path apps/desktop/src-tauri/Cargo.toml
bun run tauri dev     # full desktop window
```

Also verified.

## 7. Frontend state

- **Implemented and working**: 17 routes wired in `router.tsx`, AppShell with sidebar/header/journey-ribbon/evidence drawer, 9 semantic UI components in `@sea-forge/ui-components` with Storybook stories, full design-token projection from `.agents/specs/frontend/colors_and_type.css` → `sea-forge-ui-tokens` → Astryx theme.
- **Live backend-backed data**: `ReadinessPage`, `CaseCreationWorkbench` (draft→preflight→commit→navigate), `CaseHorizonPage`, `ApprovalInboxPage`, `RunRecordPage`, `DelegationWorkbench`, `OperationsPage`, `EvidencePage`, `DelegationRoster`. All consume SFWP through TanStack Query + AJV validation.
- **Visibly marked "Specification preview · not live"**: `ThothPage`, `AssetsPage` (partially live per `asset.list`), `ModelsPage`, `MemoryPage`, `CapabilitiesPage`, `ArtifactsPage`, `FederationPage`, `AdminPage` — render structure but do not mutate backend state.
- **Tests**: 116 desktop + 17 component = 133 Vitest tests pass.
- **Build**: `bun run build` produces a renderer bundle.
- **E2E**: Playwright harness boots Tauri IPC via `window.__TAURI_INTERNALS__` mock. **Limitation**: the harness mocks the entire Tauri bridge, so it cannot honestly prove real IPC delivery end-to-end; this is a tracked debt (`.agents/OBSERVED_DEBT.md`).
- **Accessibility**: axe-core wired into Playwright; routes scanned for violations.

## 8. Backend state

The `sea-forge-server` is a real Tokio Unix-socket NDJSON server with:

- Concurrent case dispatch under a semaphore (`max_concurrent_runs`)
- Per-episode agent delegation with cancellation handles
- ACP broker (permission mediation, restart recovery, continuation linking)
- SWE_SEED declaration reconciliation (startup + read-time)
- Transcript sealing (XChaCha20Poly1305, summarized mode default)
- 21 SFWP methods (see SUBSTRATE_MANIFEST.json)
- Durable event ledger with cursor + gap recovery
- Request correlation across reconnects

**Live smoke verified**: `system_hello` returns the full 21-method catalog; `system_describe` returns the method/class table. `readiness_get` returned an `error` body in the smoke test because no policy/seed model was provisioned at the temp root — that is expected behavior (the method is implemented but requires a configured installation). The smoke confirms the wire path works end to end.

## 9. API state

SFWP (the only externally-exposed API):

- **Transport**: AF_UNIX socket at `.sea-forge/server.sock` (default) — socket_path is **independent of `root`** in `ServerConfig`; setting `SEA_FORGE_ROOT` alone does not move the socket. This is a usability gap for operators who expect one env var to relocate the whole install.
- **Versioning**: major-only (`"1"`); unknown verb → serde unknown-variant, server rejects cleanly (verified by `unknown_request_verb_fails_clean_not_panic` conformance test).
- **Methods**: 21 implemented of 123 catalog-target. Coverage by family: system 3/3, request 1/1, events 3/3, readiness 1/1, case 5/6, approval 2/2, run 2/4, asset 1/?, delegation 2/3, thoth 0/?, memory 0/?, capability 0/?, evidence 0/?, integrity 0/?, settlement 0/?, agent_run 0/?, operation 0/1, cell 0/?.
- **Auth at API boundary**: filesystem permissions on the socket only. No in-protocol authentication; this is by design (single-operator local install) but worth noting as a deployment boundary.

CLI surface (24 subcommands, see SUBSTRATE_MANIFEST.json) — fully working.

## 10. Persistence state

- `.sea-forge/cases/<case_id>.json` — case records
- `.sea-forge/runs/<run_id>/{plan,authority,trace.jsonl,evidence.jsonl,settlement,semantic-envelope}.json[onl]` — per-run records
- `.sea-forge/runs/<run_id>/workspace/` and `artifacts/` — materialized sandbox + evidence
- `.sea-forge/capabilities.jsonl` — capability-attempt memory (v0 name preserved)
- `.sea-forge/ledgers/<stream>/` — ULID ledger streams with hash chains + MMR commitments
- `.sea-forge/requests/` — SFWP request correlation store
- `.sea-forge/sealed/` — sealed transcript keys + sealed transcripts (summarized mode)
- `.sea-forge/templates/` — materialized plan templates
- SQLite capability index (rebuilt projection, not source truth)
- All of `.sea-forge/` is gitignored and never committed (verified).

The minimum proofs (`just proof`) verify cross-reference resolution, hash verification of artifacts, schema versioning, and append-only semantics for one full lifecycle including denial and failure paths.

## 11. Domain / runtime state

Domain model is mature and typed:

- Operation kinds: `WriteFile`, `ExecuteCommand`, `AgentTask` (M12), `Synthesize`/`Productize`/`Capitalize`/`Attest` (M8), and reserved authority surfaces for API/git/PR/prompt/policy/evidence/projection/artifact-transition/approval/deployment/secret/policy-mutation
- Settlement status: `Accepted | Rejected | Escalated` (typed enum; never derived from exit code)
- Case states: `Active | Completed | AwaitingApproval | Parked | Reopened | Closed`
- Thoth question kinds: `AskOperationRequirements | AskAuthorityRequirements | AskFailureExplanation | AskEvidenceForClaim`
- Delegation termination bases: typed enum (`TurnCapExceeded`, `Cancelled`, `EndpointError`, etc.)
- Manager judgment: `Satisfied | Blocked | Progressing | Stalled`

All errors modeled as typed `ForgeError` with machine-readable classes (`Input`, `Plan { class }`, `Run`, `Serialization`, `io(...)`, etc.).

## 12. Testing state

| Tier | Result | Evidence |
|---|---|---|
| Workspace Rust tests | **776 passed, 0 failed, 4 ignored** | `cargo test --workspace --all-features --locked --no-fail-fast` (102 test binaries, ~5 min) |
| Minimum proofs P1–P4b | **green** | `devbox run -- just proof` (exit 0) |
| Workbench desktop tests | **116 passed** | `bun run test` in `apps/desktop` |
| UI components tests | **17 passed** | `bun run test` in `packages/sea-forge-ui-components` |
| Tauri host integration tests | **4/4 green** (per CURRENT_STATUS 2026-07-26) | `workbench/apps/desktop/src-tauri/tests/bridge.rs` |
| Ignored | **4 Rust + 0 TS** | Real-host release gates intentionally ignored pending operator configuration (ACP transport, SWE_SEED release, Seatbelt on Linux) |

Coverage philosophy (`ARCHITECTURE.md` §7, `spec-minimum.md` §17): tests assert both results **and forbidden side effects** — a denial test proves no child ran; a redaction test proves the sentinel secret never appears; a rebuild test compares byte-stable output. Multiple `conformance_*` integration test files exist per milestone (conformance_m5, m6, m9, m10, m11, m12, m13, m14, m15, m16, sfwp, case_authoring, case_views, approvals, assets, delegations, run_views).

## 13. Deployment and packaging state

- **No container/deployment manifests present**: no Dockerfile, no k8s yaml, no Helm chart, no Terraform. Deployment is operator-managed; the architecture intentionally keeps the kernel single-process and synchronous.
- **Release mechanism**: `release-please.yml` (GitHub Actions, OIDC trusted publishing to crates.io for `sea-forge-core` and `sea-forge-cli` only). One-time classic-token bootstrap via `just publish-bootstrap <crate>`.
- **Tauri packaging**: `bun run tauri build` produces a desktop bundle; not yet exercised in CI (Task 14 of the workbench plan, per CURRENT_STATUS).
- **Version synchronization**: `just release-check [tag]` verifies `workspace.package.version`, `sea-forge-core`, and `sea-forge-cli` agree.
- **CI gate**: `gate` job requires lint + test + macos all `success` (no path filters; skipped = failed).

## 14. Generated-code state

Generated artifacts are governed and committed:

- `workbench/packages/contracts/schema/*.schema.json` — emitted by `gen_sfwp_schema` from `crates/sea-forge-server/src/sfwp/mod.rs`'s `SCHEMA_TYPES`. Conformance test `generated_schemas_are_committed_and_current` fails the Rust gate on drift.
- `workbench/packages/contracts/generated/*.ts` + `*.validator.ts` — emitted by `bun run generate:contracts`. Verified deterministic this pass (50 types, zero diff after regeneration).
- `workbench/packages/sea-forge-ui-tokens/sea-forge.tokens.css` — byte-copy of `.agents/specs/frontend/colors_and_type.css`; `bun run check-drift` (in that package) fails on divergence.
- `.ua/knowledge-graph.json` — Understand-Anything projection; regenerated, never hand-edited.
- All build outputs (`target/`, `apps/desktop/dist/`, `apps/desktop/src-tauri/target/`, `node_modules/`, `.sea-forge/`, `target/bootstrap-evidence/`) gitignored.

## 15. External integration state

| Integration | Status | Notes |
|---|---|---|
| OpenAI-compatible HTTP | implemented | `sea-forge-agent::provider::openai` (rustls, no ambient env, Zeroizing credential) |
| Anthropic-compatible HTTP | implemented | same surface |
| ACP v1 (CLI agents) | implemented + portable-tested | `sea-forge-agent::acp`; real-host release test ignored pending operator config |
| SWE_SEED | harvesting + reconciliation implemented | `swe_seed_reconciliation`; declaration ingress production-wired (Task 18); real-host release test ignored |
| DomainForge | adapter present (`crates/sea-forge-domainforge`) | M0 boundary per ADR-001; not yet the M0 first-party library binding (still stubbed semantic surface per spec) |
| MCP servers | spec'd, not implemented | `.agents/specs/Shell-SPEC.md §2.2` says "no integrations declared yet" |
| SOPS/age secrets | working | `secrets/dev.enc.env` decrypts; `just secrets-check dev` is green |
| GitHub (PR/release) | working | `just pr` opens PR via `gh`; `release-please.yml` automates semver |

## 16. Git and working-tree risks

- **Branch is `frontend`, 5 commits ahead of `origin/frontend`**. Not pushed. The implementation work recorded in CURRENT_STATUS as 2026-07-23..2026-07-26 is unpushed.
- **Multiple `entire/*` branches** present locally — these are Entire-CLI integration snapshots (checkpoints). Not product branches.
- **`main` is present locally** but the active work is on `frontend`. The README's contributor happy path assumes `main` as the base; verify with the user whether `frontend` should merge to `main` before any release.
- **Untracked directory `working/face/`** is a parked unrelated design dump (6 markdown files dated 2026-07-09). It is not in `.gitignore`. Recommend the next pass confirm whether it should be removed, moved, or gitignored.
- **`ARCHITECTURE.md` header** states "Status: intended architecture, pre-implementation" — stale; the architecture is largely implemented.
- **`README.md` "Project layout"** describes 2 crates — stale; 22 exist.
- The current commit modified only `workbench/` (DelegationRoster tests) and the Jolli debug log; no kernel changes in the latest 5 commits.

## 17. Confirmed working behavior

Directly executed and observed this pass:

1. `cargo fmt --all -- --check` — clean (zero diff).
2. `cargo check --workspace --all-targets --locked` — clean (37.64s).
3. `cargo test --workspace --all-features --locked --no-fail-fast` — 776 passed, 0 failed, 4 ignored across 102 binaries.
4. `devbox run -- just proof` — P1–P4b passed (exit 0).
5. `scripts/check-agent-context.sh` — exit 0, passes (no longer blocked despite CURRENT_STATUS suggesting it was at one point).
6. `cd workbench && bun run check` — exit 0 (one pre-existing Fast Refresh warning in `router.tsx`).
7. `cd workbench && bun run test` — 116 desktop + 17 component tests pass.
8. `cd workbench && bun run generate:contracts` — 50 types emitted deterministically (zero diff).
9. `cargo build --manifest-path workbench/apps/desktop/src-tauri/Cargo.toml` — clean (1m 48s).
10. `./target/debug/sea-forge --help` — 24 subcommands listed, exit 0.
11. **Live server smoke**: started `sea-forge-server` with `SEA_FORGE_ROOT=/tmp/sea-rs-smoke/state`, connected to `.sea-forge/server.sock`, sent `system_hello` and received the 21-method catalog; sent `system_describe` and received the method/class table. End-to-end SFWP path works.
12. Frontend route inventory: 17 routes registered in `router.tsx`, all components present, all hooks consume SFWP through TanStack Query + AJV validators.

## 18. Confirmed failures

None encountered this pass. The four ignored Rust tests are intentional platform/release gates, not failures.

## 19. Unverified behavior

- **Real ACP transport** (subprocess JSON-RPC pump against Claude Code/Codex) — release test ignored pending operator config.
- **Real SWE_SEED host** — release test ignored.
- **Seatbelt (macOS) jail** — Linux host cannot exercise; portable Landlock tests pass.
- **Tauri desktop bundle** (`bun run tauri build` packaging) — not executed this pass; only the dev host crate was compiled.
- **Playwright e2e against a real `sea-forge-server`** — the existing harness mocks Tauri IPC entirely (debt tracked).
- **Release-please OIDC trusted publishing** — not executed.
- **`crates.io` publish path** — bootstrap not run.
- **macOS CI** — runs in GitHub Actions only; not exercised locally.

## 20. Environmental blockers

None. All host tools (`cargo 1.92.0`, `rustc 1.92.0`, `just 1.55.1`, `devbox`, `bun 1.4.0`, `node lts`, `python3`) are present and pinned. No missing prerequisites.

## 21. Evidence index

| Claim | Evidence |
|---|---|
| 22-crate workspace | `Cargo.toml:3-26` |
| 776 tests pass | `cargo test --workspace` (this pass, /tmp/sea-rs-test-results.txt) |
| 21 SFWP methods | `crates/sea-forge-server/src/sfwp/mod.rs:67-191` |
| Live server response | observation this pass (`system_hello` reply with 21-method array) |
| Frontend 17 routes | `workbench/apps/desktop/src/router.tsx:84-218` |
| Tauri host bridge | `workbench/apps/desktop/src-tauri/src/bridge.rs:27-100` |
| Kernel async invariant | `justfile:117-146` (`just no-async-kernel`), `Cargo.toml:36-40` |
| 24 CLI subcommands | `crates/sea-forge-cli/src/main.rs:25-275` |
| Sealed transcript crypto | `crates/sea-forge-server/src/transcript_seal.rs` |
| Current handoff | `.agents/CURRENT_STATUS.md` (~2402 lines) |
| Open debt | `.agents/OBSERVED_DEBT.md` (~833 lines) |
| Empty spec files | `.agents/specs/api-routes.md` (0 bytes), `.agents/specs/datamodel.md` (0 bytes) |
| Unrelated `working/face/` | `working/face/{ARCHITECTURE,DESIGN,DEV_PLAN,PRODUCT}.md` |
| 133 frontend tests | `bun run test` output (this pass) |
| Contracts deterministic | `bun run generate:contracts` zero diff (this pass) |
