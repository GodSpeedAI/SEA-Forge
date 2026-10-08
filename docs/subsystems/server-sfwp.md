# Subsystem: Server & Wire Protocol (SFWP v1)

> **Tokio-based Unix domain socket server, bounded request admission, idempotency correlation, and SFWP v1 NDJSON protocol.**  
> Governed crate: `sea-forge-server` · Governing ADRs: `ADR-003`, `ADR-005`

---

## 1. Purpose

The **Server & Wire Protocol (SFWP)** subsystem is the asynchronous edge adapter for SEA Forge. It listens on an owner-only local Unix domain socket (`0600`), provides a multiplexed request/response and streaming event interface for the Workbench desktop client and automated integrations, and bridges async network I/O into the strictly synchronous kernel crates via bounded thread pools.

---

## 2. Responsibilities

* **SFWP v1 Protocol Dispatch:** Serves NDJSON protocol methods across three interaction classes: `Inspect` (read-only projections), `Command` (governed mutations), and `Subscribe` (streaming events).
* **Bounded Request Admission:** Protects cell execution using a two-tier semaphore: active execution is bounded by `max_concurrent_runs`, while pending protected requests wait in a strictly bounded 8-slot queue (`REQUEST_ADMISSION_WAIT_QUEUE_CAPACITY = 8`). Overflow fails immediately with `server_busy`.
* **Mutation Idempotency (`RequestCorrelationStore`):** Requires caller-supplied `request_id` for durable mutations (`case.commit`, `approval.decide`, `agent.delegate`). Repeated requests return the cached terminal response without re-executing side effects.
* **Allocation Amplification Defense:** Caps single-record reads at 4 MiB (`MAX_RECORD_BYTES`) and journals at 64 MiB (`MAX_JOURNAL_BYTES`), preventing maliciously inflated files from causing out-of-memory crashes.
* **Real-Time Event Broadcasting:** Streams live `EventFrame` records over active subscription channels, backed by the append-only events ledger with durable ULID cursors.
* **Exclusive Cell Supervision:** Manages the exclusive cell lock (`<root>/server.sock.lock`), ensuring two server instances cannot concurrently bind or write to the same cell root.

---

## 3. Non-Responsibilities

* **Kernel Execution Logic:** Does not implement planner, authority, or settlement logic; dispatches synchronous kernel pipelines via `tokio::task::spawn_blocking`.
* **Remote Network Ingress:** The server listens strictly on local Unix domain sockets; remote HTTP/TLS exposure is out of scope.

---

## 4. Position in the System

```mermaid
flowchart TB
    TAURI["Tauri Desktop Host"] -->|NDJSON over server.sock (0600)| LISTENER["tokio::net::UnixListener"]
    LISTENER --> DISPATCH["SFWP Dispatcher"]
    
    DISPATCH -->|Inspect (readiness, case.list)| PROJ["In-Memory View Projections"]
    DISPATCH -->|Subscribe (events.subscribe)| BUS["Event Broadcast Channel"]
    DISPATCH -->|Command (case.commit)| ADMIT{"Request Admission Semaphore"}
    
    ADMIT -->|Overflow (>8)| BUSY["Return error: server_busy"]
    ADMIT -->|Admitted| IDEMP{"Check RequestCorrelationStore"}
    IDEMP -->|Duplicate ID| CACHED["Return Stored Terminal Outcome"]
    IDEMP -->|New Request| BLOCKING["spawn_blocking(CaseRunner / Pipeline)"]
    BLOCKING --> KERNEL["Synchronous Governed Kernel"]
```

---

## 5. Core Abstractions

### `ServerState` (`crates/sea-forge-server/src/lib.rs:66-120`)
Shared server state across all active client connections:
* `root: PathBuf`: The immutable cell root path.
* `config: RwLock<Arc<ServerConfig>>`: Hot-reloadable configuration snapshot.
* `request_admission: Arc<Semaphore>`: Concurrency permit pool equal to `max_concurrent_runs`.
* `request_admission_waiters: Arc<Semaphore>`: 8-slot waiting room for pending permits.
* `correlation: RequestCorrelationStore`: Durable store answering `request.get_status` and preventing duplicate execution.
* `correlation_admission: Mutex<()>`: Mutex serializing check-and-bind operations per request ID.
* `events_ledger: Arc<LedgerStream>`: Single durable events ledger providing the monotonic ULID event cursor.

### SFWP Method Interaction Classes
* `InteractionClass::Inspect`: Read-only queries over cell state (`system.hello`, `system.describe`, `readiness.get`, `case.list`, `case.get_horizon`, `run.get`, `asset.list`).
* `InteractionClass::Command`: Governed mutations that alter cell truth (`case.commit`, `approval.decide`, `thoth.ask`, `events.unsubscribe`).
* `InteractionClass::Subscribe`: Streaming event connections (`events.subscribe`).

---

## 6. Internal Operation

### Bounded Request Admission & Fail-Closed Overload
When a protected command arrives:
1. It attempts to acquire a permit from `request_admission_waiters` with a timeout of 10 seconds.
2. If the 8-slot waiting room is full, it immediately returns an SFWP error (`code: "server_busy"`) without creating a correlation record or executing any side effect.
3. Once admitted to the waiting room, it acquires one of the `max_concurrent_runs` active permits.
4. It checks `RequestCorrelationStore` inside the `correlation_admission` lock:
   * If the `request_id` has already completed, it returns the cached response.
   * If the `request_id` is currently in-flight, it returns a pending status.
   * If new, it marks the request as started and dispatches the task.

### Cell Root Resolution Contract
The server and CLI resolve the cell root according to `docs/CELL_CONTRACT.md`:
1. `SEA_FORGE_SOCKET`: Absolute path to the socket (wins outright).
2. `SEA_FORGE_ROOT`: The cell directory (socket is `<root>/server.sock`).
3. Default: `.sea-forge` relative to the current working directory.

---

## 7. Failure Modes & Invariants

* **Invariant OPS-01 (Configuration & Startup Integrity):** An invalid `server.yaml` causes the server process to abort immediately during startup preflight. Live reload updates future dispatches via atomic `Arc` swap; in-flight tasks retain their existing configuration.
* **Invariant DATA-03 (Idempotency Key Enforcement):** Retrying a `case.commit` with the same `request_id` after a network disconnection guarantees that only one case is created, returning the identical case ID.
* **Dual-Server Collision:** If a second server attempts to start on an existing active cell root, it fails to acquire `<root>/server.sock.lock` and exits with an `AddrInUse` error, preventing split-brain ledger corruption.

---

## 8. Source Trail

* [`crates/sea-forge-server/src/lib.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-server/src/lib.rs) — `ServerState`, Unix listener loop, admission semaphore, and case entry maps.
* [`crates/sea-forge-server/src/sfwp/mod.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-server/src/sfwp/mod.rs) — SFWP method catalog, interaction classes, and byte caps.
* [`crates/sea-forge-server/src/sfwp/correlation.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-server/src/sfwp/correlation.rs) — `RequestCorrelationStore` idempotency implementation.
* [`crates/sea-forge-server/src/config.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-server/src/config.rs) — Cell root resolution and configuration loading.
* [`tests/conformance_sfwp.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-server/tests/conformance_sfwp.rs) — SFWP protocol negotiation, schema drift, and admission tests.
