# Explanation: The Synchronous Kernel Boundary

> **Deep rationale for Invariant BUILD-01: Why 19 kernel crates forbid async runtimes, and why Tokio and HTTP clients are confined strictly to outer edge adapters.**

---

## 1. The Core Invariant: BUILD-01

SEA Forge enforces a strict, mechanically verified architectural boundary across its 22 Rust crates:

> **BUILD-01:** The 19 kernel crates must remain completely synchronous and free of asynchronous runtimes (`tokio`, `async-std`, `futures`) and HTTP clients (`reqwest`, `hyper`). Asynchronous code is restricted exclusively to two approved edge adapters: `sea-forge-server` and `sea-forge-agent`.

This invariant is verified on every commit by the canonical gate:
```sh
just no-async-kernel
```

---

## 2. Why Async in Core Governance Is Dangerous

In modern Rust development, there is a strong temptation to make every crate `async` by default. In a security-critical, deterministic governance kernel, async-everywhere introduces severe architectural liabilities:

### 1. Non-Deterministic Execution & Timing Races
Asynchronous runtimes multiplex green threads across thread pools. Cooperative yielding (`.await`), task stealing, and executor scheduling introduce non-deterministic execution timing. In SEA Forge, planning, authority rule evaluation, ledger hash-chaining, and criteria settlement must be **100% deterministic**: given identical inputs and policy, the kernel must execute the identical sequence of steps in the identical causal order on every machine.

### 2. Async Cancellation Vulnerabilities
In Rust, dropping a future cancels it at its nearest `.await` suspension point. If an async function is cancelled halfway through evaluating policy or writing a ledger entry, state can easily become inconsistent:
* An `AuthorityRequest` might commit, but cancellation drops the future before the `AuthorityDecision` commits.
* An `ActionGrant` might be partially initialized in memory.
By keeping the kernel purely synchronous, operations execute atomically to completion or fail explicitly with a typed `ForgeError`.

### 3. Supply Chain Blast Radius & Compile Times
Asynchronous runtimes and HTTP stacks (`tokio`, `hyper`, `h2`, `rustls`, `mio`) pull in hundreds of transitive dependencies, complex OS socket event loops, and platform-specific C bindings. Keeping the 19 kernel crates free of async dependencies:
* Reduces compile times from minutes to seconds.
* Dramatically shrinks the attack surface audited by `cargo-deny`.
* Enables the minimum kernel to compile and pass all proofs offline with zero network connectivity.

---

## 3. The Standalone Tauri Workspace Boundary (ADR-004)

A subtle but critical architectural decision was made in **ADR-004** regarding the desktop application (`workbench/apps/desktop/src-tauri`):

### The Risk
The Tauri desktop framework depends on `tokio`, `webkit2gtk`, and complex native windowing libraries. If `src-tauri` were an ordinary member of the root Cargo workspace, `cargo check --workspace` would adopt Tauri's dependencies into the kernel's dependency graph, permanently breaking the `no-async-kernel` boundary and forcing headless CI servers to install GTK and WebKit libraries merely to build the kernel.

### The Resolution: Empty `[workspace]` Table
`workbench/apps/desktop/src-tauri/Cargo.toml` explicitly declares an **empty `[workspace]` table**:
```toml
[package]
name = "sea-forge-workbench-desktop"
version = "0.1.0"
edition = "2021"

[workspace]
# Deliberately empty: creates an independent workspace root (ADR-004)
```
Because Cargo does not traverse upwards past an existing `[workspace]` table, `src-tauri` forms an independent workspace with its own `Cargo.lock`. The root Rust workspace builds the kernel, CLI, and server without touching desktop dependencies; the desktop builds independently during Workbench gates.

---

## 4. How the Server Bridges Async to Synchronous

How does `sea-forge-server` (which must handle multiplexed Unix socket I/O using Tokio) execute synchronous kernel tasks without blocking the async event loop?

Through **`tokio::task::spawn_blocking`**:

```rust
// crates/sea-forge-server/src/case_dispatch.rs
tokio::task::spawn_blocking(move || {
    // Inside this thread pool, pure synchronous kernel code executes:
    let result = case_runner.run_stage_episode(&case_dir, ...)?;
    Ok(result)
}).await??;
```

* The server's main async threads handle socket handshakes, NDJSON parsing, and real-time event streaming.
* When a governed episode or case plan item is ready to run, the task is handed off to a dedicated OS thread pool managed by `spawn_blocking`.
* The kernel runs purely synchronously inside that thread, completely unaware that an async server surrounds it.

---

## 5. Mechanical Verification

Invariant BUILD-01 is not an informal guideline; it is enforced mechanically by `justfile:111-146`:
```sh
just no-async-kernel
```
The script inspects `cargo tree` for the 19 synchronous crates:
* `sea-forge-core`, `sea-forge-domain`, `sea-forge-planner`, `sea-forge-authority`
* `sea-forge-sandbox`, `sea-forge-runtime`, `sea-forge-trace`, `sea-forge-evidence`
* `sea-forge-settlement`, `sea-forge-capability`, `sea-forge-ledger`, `sea-forge-domainforge`
* `sea-forge-spec-pipeline`, `sea-forge-cell`, `sea-forge-artifact-ip`, `sea-forge-self-model`
* `sea-forge-thoth`, `sea-forge-case-runner`, `sea-forge-extension`

If any of these 19 crates transitively references `tokio`, `async-std`, `futures`, `reqwest`, or `hyper`, the gate fails immediately and CI turns red.

---

## 6. Source Evidence

* [`justfile:111-146`](file:///c:/Users/sprim/projects/sea-rs/justfile#L111-L146) — Canonical `no-async-kernel` verification recipe.
* [`docs/decisions/ADR-004-workbench-stack.md`](file:///c:/Users/sprim/projects/sea-rs/docs/decisions/ADR-004-workbench-stack.md) — Architectural decision isolating the Tauri workspace.
* [`workbench/apps/desktop/src-tauri/Cargo.toml`](file:///c:/Users/sprim/projects/sea-rs/workbench/apps/desktop/src-tauri/Cargo.toml) — Independent `[workspace]` declaration.
* [`crates/sea-forge-server/src/case_dispatch.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-server/src/case_dispatch.rs) — `spawn_blocking` integration seam.
