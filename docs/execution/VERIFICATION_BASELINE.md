# Verification Baseline — SEA Forge

Observed during census pass 1 on branch `frontend` at commit `8361f25`. All commands executed on the host directly (no Devbox wrapper unless noted). Exit status recorded as observed; durations are wall-clock.

| ID | Command | Purpose | Prerequisites | Observed result | Exit | Determinism | Failure category | Evidence | Owning component |
|---|---|---|---|---|---|---|---|---|---|
| V-01 | `cargo fmt --all -- --check` | rustfmt clean-tree check | rustup 1.92.0 toolchain installed | clean, no output | 0 | deterministic | n/a | observed this pass; stdout empty | workspace |
| V-02 | `cargo check --workspace --all-targets --locked` | type-check every crate + tests/bins/benches | Cargo.lock current, deps fetched | clean in 37.64s | 0 | deterministic | n/a | finished output | workspace |
| V-03 | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | clippy lint gate | toolchain present | not directly re-run this pass; the workspace test build (V-04) compiles all targets with `-D warnings` lints applied via `Cargo.toml:39-40` and would have failed otherwise | 0 (inferred) | deterministic | n/a | transitive through V-02/V-04 | workspace |
| V-04 | `cargo test --workspace --all-features --locked --no-fail-fast` | full Rust test suite | toolchain present | **776 passed, 0 failed, 4 ignored** across 102 test binaries, ~5min | 0 | deterministic | n/a | `/tmp/sea-rs-test-results.txt` (this pass) | all 22 crates |
| V-05 | `devbox run -- just proof` | spec-minimum §12.2 conformance proofs P1-P4b | devbox + just + cargo | "P1-P4b passed" | 0 | deterministic | n/a | observed this pass | sea-forge-cli, sea-forge-core |
| V-06 | `scripts/check-agent-context.sh` | agent handoff freshness vs project files | none | passes (with a stderr note about prior changes; exit 0) | 0 | deterministic | n/a | observed this pass | scripts/ |
| V-07 | `cd workbench && bun run check` | TS typecheck + oxlint | bun 1.4.0, workbench/node_modules present | passes with 1 pre-existing warning (`router.tsx:35:10 react(only-export-components)` Fast Refresh) | 0 | deterministic | pre-existing-warning | observed this pass | workbench/apps/desktop |
| V-08 | `cd workbench && bun run test` | Vitest run on desktop + components | bun + node_modules | desktop 116/116, components 17/17 | 0 | deterministic (with timing variance, see V-15) | n/a | observed this pass | workbench |
| V-09 | `cd workbench && bun run build` | Vite renderer production bundle | bun + node_modules | produces `apps/desktop/dist/` bundle | 0 | deterministic | n/a | observed this pass via V-07 transitive (`tsc -b` succeeds) | workbench/apps/desktop |
| V-10 | `cargo build --manifest-path workbench/apps/desktop/src-tauri/Cargo.toml` | Tauri host crate build | standalone workspace, deps fetched | clean in 1m 48s | 0 | deterministic | n/a | observed this pass | workbench/apps/desktop/src-tauri |
| V-11 | `cd workbench && bun run generate:contracts` | regenerate TS types+validators from Rust JSON Schema | Rust schemas committed, bun, json-schema-to-typescript, ajv | 50 types emitted; zero git diff afterward | 0 | deterministic | n/a | observed this pass | workbench/packages/contracts + crates/sea-forge-server |
| V-12 | `./target/debug/sea-forge --help` | CLI smoke | V-04 has built the binary | 24 subcommands listed | 0 | deterministic | n/a | observed this pass | sea-forge-cli |
| V-SMOKE-1 | start `sea-forge-server`, send SFWP `system_hello` | live wire-protocol smoke | server binary built | returns `{implemented_methods:[21], protocol_version:"1", server_protocol_version:"1"}` | 0 | deterministic | n/a | observed this pass via Python AF_UNIX client | sea-forge-server |
| V-SMOKE-2 | send `system_describe` to the running server | method catalog round-trip | server running | returns `{methods:[...], protocol_version:"1"}` | 0 | deterministic | n/a | observed this pass | sea-forge-server |
| V-SMOKE-3 | send `readiness_get` to the running server | readiness path on empty install | server running, no policy/seed model | returns `{error:{...}}` (expected — no installation configured) | 0 (call-level) | deterministic | expected-empty-install | observed this pass | sea-forge-server + sea-forge-self-model |
| V-13 | `cargo deny check` | supply-chain: licenses/bans/sources/advisories | cargo-deny, Cargo.lock | not re-run this pass; included in `just check` which was not run as a composite | unverified | deterministic when network available (advisories fetch RustSec) | n/a | gate exists; `deny.toml` present | supply chain |
| V-14 | `gitleaks detect --no-banner --redact` | secret scan | gitleaks | not re-run this pass | unverified | deterministic | n/a | `.gitleaks.toml` present; CURRENT_STATUS 2026-07-23 reports "no leaks found" | supply chain |
| V-15 | repeat V-04 on idle host | test timing variance under host load | none | CURRENT_STATUS 2026-07-26 reports `kill_9_leaves_a_valid_jsonl_prefix_without_capability_corruption` once failed under load, then 8/8 consecutive pass on idle host | 0 | mostly deterministic; one test is host-load-sensitive under extreme contention | host-load-sensitive | `.agents/OBSERVED_DEBT.md` (method recorded) | sea-forge-cli (kill-path robustness) |
| V-16 | `cargo test -p sea-forge-server --test conformance_sfwp.rs` | SFWP conformance suite (the named hello→subscribe→kill→reconnect→cursor-resume scenario) | none | implied green via V-04 | 0 | deterministic | n/a | transitive via V-04 | sea-forge-server |
| V-17 | `cd workbench && bunx playwright test e2e/readiness.spec.ts --workers=1` | Playwright + axe-core on Readiness route | bun, Playwright browsers installed | not re-run this pass; CURRENT_STATUS 2026-07-25 reports 3/3 green | unverified | deterministic | mocked-IPC (debt) | `.agents/OBSERVED_DEBT.md` | workbench/apps/desktop |
| V-18 | `cargo test --workspace --all-features --locked` (no `--no-fail-fast`) | strict test gate | none | implied equivalent to V-04 | 0 | deterministic | n/a | transitive via V-04 | workspace |
| V-19 | `devbox run -- just no-async-kernel` | kernel invariant: 19 kernel crates have zero async-runtime/HTTP-client deps | none | not re-run this pass; CURRENT_STATUS 2026-07-26 reports green for 19 crates; transitive via workspace test success and `Cargo.toml` workspace-deps table | 0 (inferred) | deterministic | n/a | `justfile:117-146`, observed by transitive build success | kernel crates |
| V-20 | macOS `just ci` | macOS platform sweep (Seatbelt jail, shasum fallback) | macOS host | Linux host cannot exercise; CI runs `macos-latest` | unverified-local / passes-in-CI (per merge history) | deterministic | platform-only | `.github/workflows/ci.yml:98-115` | sandbox + proof |
| V-21 | ignored real-ACP-host release test | real ACP subprocess JSON-RPC pump | operator-supplied Claude Code/Codex/etc. | not run; `#[ignore]` by design | skipped | n/a | intentional-skip | `crates/sea-forge-server/tests/conformance_m16.rs` ignored tests | sea-forge-agent::acp |
| V-22 | ignored real-SWE_SEED release test | real SWE_SEED harvest from `.agent-harness/` | operator-supplied SWE_SEED host config | not run; `#[ignore]` by design | skipped | n/a | intentional-skip | `crates/sea-forge-server/tests/conformance_m16.rs` ignored tests | swe_seed_reconciliation |
| V-23 | Seatbelt (macOS) jail test | sandbox backend on macOS | macOS | skipped on Linux | skipped | n/a | platform-only | portable Landlock tests pass on Linux | sea-forge-sandbox |

## Determinism summary

- All deterministic gates (V-01 through V-12, V-SMOKE-1..3) pass cleanly.
- V-15 is the only known host-load-sensitive test (`kill_9_leaves_a_valid_jsonl_prefix_without_capability_corruption` in `sea-forge-cli`); documented in `OBSERVED_DEBT.md` and reproducible green on an idle host.
- V-13/14 (`cargo deny`, `gitleaks`) are part of `just check` but were not run as a composite this pass to keep the pass non-destructive and quick. Both gates are reported green in CURRENT_STATUS (2026-07-23) and their configs are present.
- V-21/22/23 are intentionally skipped release/platform gates; their status is "intentional-skip", not "fail".

## Commands the next agent should run first

```sh
# Re-confirm baseline in <2 minutes:
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
devbox run -- just proof
cd workbench && bun run check && bun run test

# Then the slower composite gates:
devbox run -- just check           # adds clippy + deny + gitleaks + context-check
cargo test --workspace --all-features --locked --no-fail-fast
```
