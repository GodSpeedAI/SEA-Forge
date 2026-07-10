# Current Status

Updated: 2026-07-10

## Objective

Bootstrap Rust-native CLI diagnostics and make repository context sufficient for
a new agent to diagnose the foundation and resume active work without re-discovery.

## Worktree State

`main` contains uncommitted observability and context-freshness changes. No commit
or push has been requested. Preserve all listed changes as one active workstream.

## Changed Files

- Agent context: `AGENTS.md`, `.github/copilot-instructions.md`,
  `.agents/{CURRENT_STATUS,OBSERVED_DEBT,OPEN_QUESTIONS}.md`.
- Context gate: `scripts/check-agent-context.sh`,
  `scripts/tests/check-agent-context.sh`, `justfile`.
- Observability: root `Cargo.toml`, `Cargo.lock`,
  `crates/sea-forge-cli/{Cargo.toml,src/main.rs,tests/diagnostics.rs}`.
- Documentation: `README.md`, `ARCHITECTURE.md`,
  `.agents/specs/Shell-SPEC.md`.

## Completed

- Added JSON stderr diagnostics with `tracing` and `tracing-subscriber`.
- Added stable `event`, `run_id`, `component`, and `error_class` fields.
- Added `RUST_LOG` filtering with an `info` fallback for invalid/missing filters.
- Added an integration test that parses and verifies the emitted JSON.
- Documented diagnostics versus governed `trace.jsonl` in `README.md`.
- Added a vendor-agnostic context gate with local-dirty and base-revision tests.
- Wired context validation into `just check` and documented local-memory roles.
- Added a Copilot compatibility pointer to the canonical root `AGENTS.md`.
- Added structured debt and open-question templates and recorded the real runtime
  diagnostic gap as minimum-kernel acceptance work.

## Verification

- `cargo test --workspace --all-features --locked`: 3 passed, 0 failed.
- Formatting, Clippy with warnings denied, and locked checks pass.
- `devbox run -- just check`: passes cargo-deny and gitleaks; no leaks found.
- Manual CLI run emits one JSON diagnostic to stderr and exits 64.
- `sh scripts/tests/check-agent-context.sh`: 6 scenarios passed.
- `sh scripts/check-agent-context.sh`: current handoff passes.

## Remaining

- Instrument minimum-kernel lifecycle modules as they are implemented.
- Add aggregate metrics and cross-process tracing at the full spec M3 boundary,
  when a server and monitoring consumer exist.

## Blockers

None.

## Decisions

- Keep the freshness contract in POSIX shell under `scripts/`; CI providers and
  agent tools call the same script rather than owning separate implementations.
- Use worktree and optional base-revision comparison instead of date expiry, so a
  stable project is not marked stale merely because no work occurred recently.
- Do not scaffold metrics or lifecycle telemetry before the minimum kernel exists.
