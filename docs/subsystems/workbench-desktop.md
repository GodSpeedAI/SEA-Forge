# Subsystem: Workbench Desktop & Tauri Host

> **Tauri 2 native host, React 19 desktop interface, Astryx design system, and G1–G9 route guards.**  
> Governed workspaces: `workbench/` (Bun workspace), `workbench/apps/desktop/src-tauri` (Standalone Cargo workspace) · Governing ADR: `ADR-004`

---

## 1. Purpose

The **Workbench Desktop & Tauri Host** subsystem provides the primary graphical operator, approver, and auditor experience for SEA Forge. It renders real-time case horizons, approval inboxes, readiness dashboards, and evidence inspectors while strictly adhering to **ADR-004**: the webview renderer remains a pure presentation layer, completely isolated from direct operating system sockets, filesystem paths, and kernel data structures.

---

## 2. Responsibilities

* **Tauri 2 Native Host (`src-tauri`):**
  * Serves as the native trust boundary between the operating system and the web renderer.
  * Manages the Unix domain socket client (`SocketHandle`) connecting to `sea-forge-server`.
  * Supervises or adopts the local cell server process (`CellSupervisor`).
  * Runs a background reconnect and gap-recovery loop maintaining an on-disk event cursor (`app_data_dir/sfwp-cursor.json`).
  * Exposes a strictly closed, typed command bridge (`bridge.rs`) to the renderer.
* **React 19 Presentation Layer (`apps/desktop`):**
  * Implements 17 typed routes via TanStack Router.
  * Renders UI primitives using the Astryx design system and SEA Forge design tokens.
  * Manages frontend interactive workflows using XState v5 state machines.
  * Evaluates G1–G9 route guards against source-backed cell readiness data.
* **Generated Schema Contracts (`packages/contracts`):**
  * Generates TypeScript types and AJV runtime validators from Rust `schemars` definitions.

---

## 3. Non-Responsibilities

* **Policy Decisions & Settlement:** The frontend never decides authority or outcome settlement; it displays server-attested state.
* **Direct Socket or Database Access:** The webview has zero access to raw Unix sockets, SQLite databases, or the `.sea-forge/` directory.

---

## 4. Position in the System

```mermaid
flowchart TB
    subgraph WEBVIEW ["React 19 Renderer (apps/desktop)"]
        ROUTER["TanStack Router (17 Routes)"]
        GUARDS["G1-G9 Guard Evaluator"]
        XSTATE["XState v5 Machines"]
        AJV["AJV Schema Validators"]
        ROUTER --> GUARDS
        GUARDS -->|Guard Failed| DENIAL["GovernedDenialSurface"]
        GUARDS -->|Guard Passed| PAGES["Page Components"]
    end

    subgraph TAURI_HOST ["Tauri 2 Host (src-tauri - Standalone Cargo Workspace)"]
        BRIDGE["bridge.rs (Closed SfwpQuery / SfwpCommand)"]
        SUPER["CellSupervisor (Process Lifecycle)"]
        CURSOR["EventCursor (sfwp-cursor.json)"]
        SOCKET["socket.rs (NDJSON Client)"]
        WEBVIEW <-->|Tauri IPC (invoke / emit)| BRIDGE
        BRIDGE <--> SOCKET
        SOCKET <--> CURSOR
    end

    subgraph SERVER ["sea-forge-server"]
        SOCK["server.sock (Unix Domain Socket, 0600)"]
        SOCKET <-->|SFWP v1 Protocol| SOCK
    end
```

---

## 5. Core Abstractions

### Standalone Cargo Workspace (`workbench/apps/desktop/src-tauri/Cargo.toml`)
To enforce invariant **BUILD-01** (keeping the 19 kernel crates free of async runtimes), `src-tauri` declares its own empty `[workspace]` table. This prevents Cargo from adopting the root workspace, ensuring Tauri's substantial asynchronous dependency tree never pollutes the synchronous kernel.

### Closed Host Bridge (`bridge.rs`)
The renderer reaches the backend exclusively through two closed enums:
* `SfwpQuery`: `Hello`, `Describe`, `GetSchema`, `ReadinessGet`, `IdentityGet`, `CaseList`, `CaseGetOverview`, `CaseGetHorizon`, `ApprovalList`, `RunList`, `RunGet`, `AssetList`, etc.
* `SfwpCommand`: `CaseCommit`, `ApprovalDecide`, `ThothAsk`, etc.
Generic shell execution escapes (`invoke("execute_command")`) are strictly forbidden.

### G1–G9 Route Guards (`guards/guards.ts`)
Every route evaluates typed guard predicates over `GuardContext`:
* `G1`: Cell root resolved and socket connected.
* `G2`: Server version and SFWP protocol negotiated.
* `G3`: Local actor identity bound and verified.
* `G4`: Policy engine active and verifiable.
* `G5`: Genesis self-model validated.
* `G6`: OS jail sandbox available and operational.
* `G7`: Integrity ledgers verified without drift.
* `G8`: Active case selected and valid.
* `G9`: Required human role satisfied.
If any blocking guard evaluates to `failed`, the router renders a `GovernedDenialSurface` displaying the exact failed invariant and necessary remediation.

---

## 6. Internal Operation: Reconnect & Gap Recovery

1. On startup, `CellSupervisor` inspects `docs/CELL_CONTRACT.md` resolution ranks and attaches to or launches `sea-forge-server`.
2. `EventCursor::load()` reads the last acknowledged `entry_ulid` from `sfwp-cursor.json`.
3. `run_event_loop()` opens a socket connection and sends `events.subscribe { from_cursor: Some(cursor) }`.
4. If a network blip occurs, the socket reconnects automatically, requests missed events via `events.get_range`, and resumes live streaming without loss of causal order.

---

## 7. Failure Modes & Invariants

* **Invariant API-02 (Closed Renderer Seam):** The renderer cannot open filesystem files or connect to remote hosts. All data flows through closed Tauri IPC commands validated by AJV.
* **Schema Drift:** If a Rust contract type in `sea-forge-server` is updated without regenerating `@sea-forge/contracts`, `just workbench-contracts-gate` and CI fail immediately.

---

## 8. Source Trail

* [`workbench/apps/desktop/src-tauri/src/lib.rs`](file:///c:/Users/sprim/projects/sea-rs/workbench/apps/desktop/src-tauri/src/lib.rs) — Tauri host entry point and event loop.
* [`workbench/apps/desktop/src-tauri/src/supervisor.rs`](file:///c:/Users/sprim/projects/sea-rs/workbench/apps/desktop/src-tauri/src/supervisor.rs) — Cell process supervisor.
* [`workbench/apps/desktop/src-tauri/src/bridge.rs`](file:///c:/Users/sprim/projects/sea-rs/workbench/apps/desktop/src-tauri/src/bridge.rs) — Closed Tauri IPC query/command router.
* [`workbench/apps/desktop/src/router.tsx`](file:///c:/Users/sprim/projects/sea-rs/workbench/apps/desktop/src/router.tsx) — TanStack Router configuration and route guard integration.
* [`workbench/apps/desktop/src/guards/guards.ts`](file:///c:/Users/sprim/projects/sea-rs/workbench/apps/desktop/src/guards/guards.ts) — G1–G9 guard predicate definitions.
* [`docs/decisions/ADR-004-workbench-stack.md`](file:///c:/Users/sprim/projects/sea-rs/docs/decisions/ADR-004-workbench-stack.md) — Architectural decision record governing the workbench stack.
