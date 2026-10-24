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

## 3. The Standalone Desktop Workspace Boundary (Historical)

> Status (2026-10-13): the Tauri `src-tauri` workspace was removed with the desktop shell. This section is kept as history; the current Workbench is a Bun/React web renderer that reaches the kernel through SFWP. See [ADR-004](../decisions/ADR-004-workbench-stack.md).

The original boundary decision was still instructive. A Tauri host depends on `tokio`, `webkit2gtk`, and native windowing libraries. If it had been an ordinary member of the root Cargo workspace, `cargo check --workspace` would have adopted those dependencies into the kernel's dependency graph, breaking the `no-async-kernel` boundary and forcing headless CI hosts to install GTK and WebKit merely to build the kernel.

The Tauri manifest therefore declared an empty `[workspace]` table, making `src-tauri` its own workspace root with its own lockfile. Cargo does not traverse upward past an existing `[workspace]` table. That host is gone; the boundary rule it protected is not.

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
The script inspects `cargo tree` for the 22 synchronous crates (all workspace members except `sea-forge-agent` and `sea-forge-server`):
* `sea-forge-core`, `sea-forge-domain`, `sea-forge-planner`, `sea-forge-authority`
* `sea-forge-sandbox`, `sea-forge-runtime`, `sea-forge-trace`, `sea-forge-evidence`
* `sea-forge-settlement`, `sea-forge-capability`, `sea-forge-ledger`, `sea-forge-domainforge`
* `sea-forge-spec-pipeline`, `sea-forge-cell`, `sea-forge-artifact-ip`, `sea-forge-self-model`
* `sea-forge-thoth`, `sea-forge-case-runner`, `sea-forge-extension`, `sea-forge-cli`
* `sea-forge-changeset`, `sea-forge-realitytrace`

If any of these 22 crates transitively references `tokio`, `async-std`, `futures`, `reqwest`, or `hyper`, the gate fails immediately and CI turns red. The two edge crates are where async is allowed.

---

## 6. Source Evidence

* [`justfile:111-146`](../../justfile#L111-L146) — Canonical `no-async-kernel` verification recipe.
* [`docs/decisions/ADR-004-workbench-stack.md`](../decisions/ADR-004-workbench-stack.md) — Architectural decision isolating the removed Tauri workspace.
* [`workbench/AGENTS.md`](../../workbench/AGENTS.md) — Current Workbench renderer/host boundary.
* [`crates/sea-forge-server/src/case_dispatch.rs`](../../crates/sea-forge-server/src/case_dispatch.rs) — `spawn_blocking` integration seam.
