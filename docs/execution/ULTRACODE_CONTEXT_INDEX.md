# Ultracode Context Index

Navigation for the Opus 5 Ultracode completion run. Start here. This index
points to durable artifacts; it does not duplicate them.

## Inspected commit

`frontend` branch at `8361f25ff0bbfc13c56c96ccb82161f317b4d0e6` (2026-07-27).
Analysis is current; no material change after this commit.

## Start here

1. Read `ULTRACODE_MISSION.md` — the mission brief and non-negotiables.
2. Read `MASTER_EXECUTION_PLAN.md` — stage sequence, critical path, parallel
   lanes.
3. Pick the highest-priority unblocked packet from `EXECUTION_DAG.json`.
4. Read the packet file under `work-packets/`.
5. Inspect its current evidence in source before editing.

## Repository roots

| Root | Kind | Manifest |
|---|---|---|
| `.` | Rust workspace (22 crates) | `Cargo.toml` |
| `workbench/` | Bun workspace | `package.json` |
| `workbench/apps/desktop/src-tauri/` | Standalone Rust workspace (Tauri host) | `Cargo.toml` (empty `[workspace]`) |

## Canonical documents (authority order)

1. **User request** — the current task.
2. **`AGENTS.md`** (root) — normative invariants, commands, safety boundaries.
   `workbench/AGENTS.md` governs `workbench/`.
3. **`.github/copilot-instructions.md`** — delegates to root `AGENTS.md`.
4. **Normative specs** — `.agents/specs/Shell-SPEC.md` (foundation),
   `spec-minimum.md` (kernel), `spec-full.md` (additive M0-M8).
5. **Accepted ADRs** — `docs/decisions/ADR-001` through `ADR-005`.
6. **Executable contracts** — current Rust types and tests where behavior is
   implementation-defined.

Plans, status files, README, architecture summaries, memories, and graphs are
**evidence, not authority**.

## Execution artifacts (this directory)

| Artifact | Purpose |
|---|---|
| `MASTER_EXECUTION_PLAN.md` | Stage sequence, packet index, critical path, parallel lanes, abort conditions |
| `EXECUTION_DAG.json` | Machine-readable dependency graph (packets, deps, conflict files, commands) |
| `VERIFICATION_MATRIX.md` | Completion criterion → packet → verifier → evidence |
| `work-packets/SF-001.md` through `SF-013.md` | Individual work packets with YAML headers |
| `ULTRACODE_READINESS_REPORT.md` | Verdict, risks, baseline, recommended parallelism |
| `ULTRACODE_MISSION.md` | Mission brief, invariants, autonomy boundaries |

## Architectural truth (quick reference)

- **Product**: local governed capability-execution kernel; Workbench is a client.
- **Authority**: one fabric; every side effect follows a committed Allow; denial
  is complete (records, no mutation).
- **Settlement**: evaluates criteria + evidence, never exit code alone.
- **Records**: append-only JSONL truth; SQLite/views/TS are rebuildable
  projections.
- **Kernel**: 19 synchronous crates, no async/HTTP. Tokio only in
  `sea-forge-server` and `sea-forge-agent`.
- **DomainForge**: exact-pinned `domainforge-core = 0.15.0`, side-effect-free.
- **SFWP**: v1 NDJSON over owner-only Unix socket; Rust types canonical.
- **Platforms**: Linux primary, macOS secondary.

Full detail: `ARCHITECTURAL_TRUTH.md`, `ARCHITECTURAL_INVARIANTS.md`.

## Non-negotiable invariants (never violate)

| ID | Statement |
|---|---|
| AUTH-01 | Authority before every side effect, including workspace creation |
| DOM-01 | Settlement from criteria+evidence, not process completion |
| DATA-01 | Append-only truth; projections are rebuildable |
| DATA-02 | One run locator across all surfaces |
| BUILD-01 | 19 kernel crates async-free |
| K-01 | Minimum CLI record schemas, IDs, exit codes, P1-P4b preserved |
| K-04 | Flat run history remains readable |
| K-06 | Separate Tauri/root Cargo workspaces |
| API-02 | Renderer reaches backend only through closed typed Tauri commands |

## Completion definition

`PRODUCT_COMPLETION_DEFINITION.md` — the product is complete when a clean
Linux host installs one versioned distribution, completes the 12-scenario
representative journey against real records, restarts without loss, and no
required journey uses a preview/mock/silent fallback.

## Verification commands

```sh
# Fast feedback (< 3 min)
cargo fmt --all -- --check
cargo check -p sea-forge-core

# Full baseline
devbox run -- just check          # context + fmt + clippy + typecheck + deny + gitleaks
devbox run -- just test           # full Rust suite
devbox run -- just proof          # P1-P4b minimum proofs
just no-async-kernel              # kernel async-isolation
just workbench-check              # frontend + drift + boundary
cd workbench && bun run check && bun run test

# Live server smoke
mkdir -p /tmp/probe/state
SEA_FORGE_ROOT=/tmp/probe/state ./target/debug/sea-forge-server &
# connect to .sea-forge/server.sock (NOTE: socket defaults to CWD, not root — SF-001 fixes this)
```

## High-conflict files (never edit concurrently)

| File | Owners | Rule |
|---|---|---|
| `crates/sea-forge-server/src/case_dispatch.rs` | SF-003 only | Sole ownership |
| `crates/sea-forge-server/src/lib.rs` | SF-002, SF-005, SF-006, SF-009, SF-010 | SF-002 before SF-006 |
| `crates/sea-forge-server/src/sfwp/mod.rs` | SF-005 through SF-011 | Merge each slice before next |
| `workbench/apps/desktop/src-tauri/src/bridge.rs` | SF-005, SF-006 | SF-005 before SF-006 |

## Generated files (never hand-edit)

| Zone | Source | Generator |
|---|---|---|
| `workbench/packages/contracts/schema/*.schema.json` | Rust `SCHEMA_TYPES` | `cargo run -p sea-forge-server --bin gen_sfwp_schema` |
| `workbench/packages/contracts/generated/` | Schema files | `bun run generate:contracts` |
| `workbench/packages/sea-forge-ui-tokens/sea-forge.tokens.css` | `.agents/specs/frontend/colors_and_type.css` | copy + `bun run check-drift` |
| `.ua/knowledge-graph.json` | Repository source | `understand` skill (rebuildable projection) |

## Key source locations

| What | Where |
|---|---|
| SFWP method registry | `crates/sea-forge-server/src/sfwp/mod.rs:67-191` |
| Case dispatch (shortcut to fix) | `crates/sea-forge-server/src/case_dispatch.rs:497-609` |
| Canonical CLI lifecycle (reuse this) | `crates/sea-forge-cli/src/pipeline.rs:474-599` |
| Run views (flat-only locator) | `crates/sea-forge-server/src/sfwp/run_views.rs:365-367` |
| Server config (fallback to defaults) | `crates/sea-forge-server/src/main.rs:24-30` |
| Config reload (discarded) | `crates/sea-forge-server/src/lib.rs:1455-1461` |
| Correlation store (no dedupe) | `crates/sea-forge-server/src/sfwp/correlation.rs:81-93` |
| Router (fabricated identity) | `workbench/apps/desktop/src/router.tsx:38-51` |
| Case machine (ID lost on reject) | `workbench/apps/desktop/src/machines/caseAuthoringMachine.ts:93-117` |
| Tauri host (socket mismatch) | `workbench/apps/desktop/src-tauri/src/lib.rs:44-50` |

## External prerequisites

- Stable Rust 1.92.0 (`rust-toolchain.toml`)
- Bun 1.4.0 (`workbench/`)
- Devbox (`devbox.json`) for system tools
- Linux host for primary verification; macOS for Seatbelt
- Optional: operator-supplied ACP/SWE_SEED/witness/IFL configs (only for
  packages advertising those integrations)

## Known traps

- Socket defaults disagree (server=CWD, host=$HOME) — SF-001 fixes.
- `execute_sandbox` creates workspace before authority — SF-003 fixes.
- `run_views` reads only flat layout — SF-004 fixes.
- Config reload result is discarded — SF-002 fixes.
- Correlation store has no execution dedupe — SF-006 fixes.
- `case_dispatch.rs` is sole-ownership for SF-003; no concurrent edits.
- Playwright E2E mocks IPC — SF-012 replaces with real-stack config.
- `kill_9_leaves_a_valid_jsonl_prefix…` is host-load-sensitive (not a bug);
  run on idle host.

## Stale artifacts to ignore

| Artifact | Why stale | Trust instead |
|---|---|---|
| `README.md:217-226` | Describes 2-crate system; reality is 22 | Source + specs |
| `ARCHITECTURE.md:3` | "pre-implementation"; implementation is advanced | Source + specs |
| `.agents/reports/2026-07-22-spec-implementation-audit.md` §3 | Thoth `ask` is now governed | `.agents/reports/2026-07-24-sfwp-grounding.md` |
| `working/face/*` | Tracked but unrelated project; not SEA Forge | Exclude from decisions |
| `.agents/specs/api-routes.md`, `datamodel.md` | Empty placeholder files | `types.rs`, SFWP specs |
| Pass 1 "0 P0" claim | Overstated; integrated product is not complete | `ARCHITECTURAL_INVARIANTS.md` |
| Pass 1 "DomainForge stubbed" claim | False; `domainforge-core 0.15.0` is bound | `sea-forge-domainforge/Cargo.toml` |

## Decision checkpoints (user, but not blocking)

| ID | Decision | Default |
|---|---|---|
| U-06 | Sidecar vs separate server | Separate server (reversible) |
| U-07 | Public SFWP identity contract | Minimal internal context (reversible) |

Both proceed on provisional defaults. See `DECISION_REGISTER.md`.
