# SEA Forge Crates Agent Guide

Durable operating instructions for the 22 Rust workspace crates under `crates/`. This directory contains the synchronous governed kernel, the server and agent edge ingress, and the CLI. Follow root `AGENTS.md` for repository-wide invariants and retrieval tools.

## 1. Synchronous Kernel vs. Async Edge Boundary

* **19 kernel crates MUST remain strictly synchronous**: `sea-forge-core`, `sea-forge-domain`, `sea-forge-authority`, `sea-forge-planner`, `sea-forge-sandbox`, `sea-forge-runtime`, `sea-forge-trace`, `sea-forge-evidence`, `sea-forge-settlement`, `sea-forge-capability`, `sea-forge-extension`, `sea-forge-ledger`, `sea-forge-domainforge`, `sea-forge-spec-pipeline`, `sea-forge-cell`, `sea-forge-artifact-ip`, `sea-forge-self-model`, `sea-forge-thoth`, `sea-forge-case-runner`.
* **Forbidden in kernel**: `tokio`, `async-std`, `smol`, `embassy`, `executor`, `reqwest`, `hyper`, `ureq`, `isahc`, `surf`, `attohttpc`, `minreq`, `actix-http`, `awc`.
* **Edge exceptions**: Only `sea-forge-server` and `sea-forge-agent` may depend on async runtimes (`tokio`) or HTTP clients (`reqwest`).
* **Mechanical check**: Run `just no-async-kernel` to verify dependency boundaries.

## 2. Development & Inner Loop

Prefer package-scoped feedback before workspace-wide gates:

```sh
just check-fast                         # quick fmt and typecheck (<10s)
just crate-check <crate>                # typecheck single crate (e.g. just crate-check sea-forge-core)
just crate-test <crate> [test_filter]   # test single crate with optional filter
just check                              # full workspace typecheck (cargo check --locked)
just test                               # full workspace test suite
just proof                              # minimum kernel proofs P1–P4b
```

## 3. Architecture & Domain Invariants

* **Typed intent**: Normalize intent into typed operations; never interpolate intent into shell commands or paths.
* **Authority fabric (`sea-forge-authority`)**: Default deny. Decide all operations before executing any side effects. One authority fabric across CLI, server, and extensions; keep policy out of sandbox and runtime.
* **Sandbox (`sea-forge-sandbox`)**: Safe workspace materialization. Validate workspace paths via the canonical safe-join algorithm. Isolation class (Landlock on Linux, Seatbelt on macOS) must fail closed; never weaken isolation by fallback.
* **Runtime (`sea-forge-runtime`)**: Execute child processes via argv execution, a minimal explicit environment, and enforced timeouts.
* **Integrity ledger & trace (`sea-forge-ledger`, `sea-forge-trace`)**: Append-only JSONL truth; views and indexes are rebuildable projections.
* **Settlement (`sea-forge-settlement`)**: Evaluate outcomes from evidence and evaluator criteria, never from process exit code alone.
* **DomainForge boundary (`sea-forge-domainforge`)**: Wraps `domainforge-core` PEG parser and semantic graph. DomainForge owns `.sea` syntax and semantics; SEA Forge owns authority and governance records.

## 4. Contract Synchronization (`sea-forge-server` ↔ Workbench)

* `crates/sea-forge-server/src/sfwp/mod.rs` (`SCHEMA_TYPES`) is the canonical source of truth for SFWP JSON schemas emitted to `workbench/packages/contracts/schema/`.
* When modifying any SFWP DTO or request verb:
  1. Run `just workbench-contracts-generate` to regenerate schemas and TypeScript/AJV bindings.
  2. Verify with `just workbench-contracts-gate`.
  3. Verify with `cargo test -p sea-forge-server --test conformance_sfwp`.

## 5. Testing Discipline

* Cover allow, deny, escalate, malformed input, timeout, nonzero exit, and false success.
* Denied paths must prove absence of side effects and absence of child execution.
* Minimum kernel tests must remain offline and credential-free.
* Mutation testing (`just mutation` / `cargo mutants`) and coverage (`just coverage` / `cargo llvm-cov`) are settlement evidence gates, not routine edit-loop checks.

## 6. Rust Toolchain & Build Discipline

* Toolchain: Rust 2021, pinned stable `1.92.0`, rustfmt defaults, clippy (`-D warnings`). `unsafe_code = "deny"` is enforced workspace-wide in `Cargo.toml`.
* Single compile writer: Only one agent may compile against `target/` at a time. Never run `cargo clean` while compiling or as a routine response to failures.
* Diagnosing build storage: Use `just timings`, `just cache-stats` (`sccache`), and `du -sh target`.
* WSL memory constraints: Do not increase Cargo build jobs or nextest concurrency to mask slow builds. Diagnose memory pressure on OOM failures.
