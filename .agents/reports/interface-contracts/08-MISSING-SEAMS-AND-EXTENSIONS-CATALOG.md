# 08 - Missing Seams and Extensions Catalog

## 1. Executive Summary

To achieve complete frontend and agent usability without inventing phantom backend capabilities or redesigning existing crates, this catalog records every required interface seam. For each seam, we document:
1. **Owning Subsystem**
2. **Current Implemented Surface** (Evidence-backed)
3. **Missing Capability / Seam**
4. **Smallest Proposed Backend Extension** (Rust/Go/CLI contract)
5. **Current Mock Representation** (How the mock adapter implements it right now)
6. **Unblocking Status** (Confirms UI builders and agents can proceed immediately)

---

## 2. Comprehensive Seams Inventory

### Seam 1: Dynamic Discretionary Work Addition
- **Owning Subsystem**: SEA-Forge (`sea-rs`)
- **Current Implemented Surface**: Case definitions support discretionary items defined in planning tables; case state tracks plan items.
- **Missing Operation**: Dedicated SFWP JSON-RPC method `case_add_discretionary_work` to instantiate a discretionary item into the active execution plan at runtime.
- **Smallest Proposed Extension**:
  ```rust
  // In crates/sea-forge-server/src/sfwp/methods/case.rs
  pub async fn handle_case_add_discretionary_work(
      state: Arc<ServerState>,
      params: AddDiscretionaryWorkParams,
  ) -> Result<PlanItemSnapshot, SfwpError> {
      // 1. Verify caller role against planning table authorized_roles
      // 2. Instantiate PlanItemInstance in active Stage
      // 3. Emit CasePlanItemAddedEvent to append-only ledger
  }
  ```
  **JSON-RPC Wire Contract**:
  ```json
  {
    "method": "case_add_discretionary_work",
    "params": {
      "case_id": "case-001",
      "stage_id": "stage_impl",
      "discretionary_item_id": "disc-pen-test",
      "actor_id": "usr-dev-alice",
      "actor_role": "developer"
    }
  }
  ```
- **Current Mock Representation**: Implemented in TypeScript `MockCaseworkAdapter.addDiscretionaryWork()`:
  - Dynamically adds the task to `visible_objects` with status `ENABLED`.
  - Increments the world projection monotonic cursor.
  - Emits `PlanItemAddedEvent` over the SSE stream.
- **Unblocking Status**: **Fully Unblocked**. UI builds against the typed client method `client.addDiscretionaryWork()`.

---

### Seam 2: Human Task Decision & Approval Inbox
- **Owning Subsystem**: SEA-Forge (`sea-rs`)
- **Current Implemented Surface**: Plan items can have kind `HumanTask` and wait for external completion events.
- **Missing Operation**: Dedicated SFWP JSON-RPC method `case_decide_human_task` with structured approval/rejection payloads and Separation of Duties validation.
- **Smallest Proposed Extension**:
  ```rust
  // In crates/sea-forge-server/src/sfwp/methods/case.rs
  pub async fn handle_case_decide_human_task(
      state: Arc<ServerState>,
      params: DecideHumanTaskParams,
  ) -> Result<DecisionResult, SfwpError> {
      // 1. Assert actor is NOT the same identity that submitted the prior task (SoD)
      // 2. Validate actor has required approval role
      // 3. Transition HumanTask to COMPLETED (if APPROVED) or FAILED/REJECTED
      // 4. Fire dependent sentries
  }
  ```
  **JSON-RPC Wire Contract**:
  ```json
  {
    "method": "case_decide_human_task",
    "params": {
      "case_id": "case-001",
      "task_id": "obj-gate-security",
      "decision": "APPROVED", // or "REJECTED"
      "justification": "Cryptographic Ed25519 signature checks passed; no regression.",
      "actor_id": "usr-sec-bob",
      "actor_role": "security_officer"
    }
  }
  ```
- **Current Mock Representation**: Implemented in TypeScript `MockCaseworkAdapter.decideHumanTask()`:
  - Verifies actor role is `security_officer` or `case_architect`.
  - Transitions gate to `COMPLETED` on approval, advances stage to next phase.
  - On rejection, marks task `REJECTED`, sets stage back to `ACTIVE`, and opens a remediation task.
- **Unblocking Status**: **Fully Unblocked**. UI builds against `client.decideHumanTask()`.

---

### Seam 3: Administrative Sentry Override
- **Owning Subsystem**: SEA-Forge (`sea-rs`)
- **Current Implemented Surface**: Sentry logic evaluates entry/exit rules based on predicates.
- **Missing Operation**: SFWP method `case_override_sentry` allowing authorized administrators or security officers to bypass a blocking sentry with mandatory recorded justification.
- **Smallest Proposed Extension**:
  ```rust
  // In crates/sea-forge-server/src/sfwp/methods/case.rs
  pub async fn handle_case_override_sentry(
      state: Arc<ServerState>,
      params: OverrideSentryParams,
  ) -> Result<SentryOverrideResult, SfwpError> {
      // 1. Verify caller has admin/security_officer role
      // 2. Require non-empty justification string (min 20 chars)
      // 3. Emit immutable SentryOverriddenEvent to ledger
      // 4. Force sentry evaluated state to TRUE
  }
  ```
- **Current Mock Representation**: Implemented in `MockCaseworkAdapter.overrideSentry()`. Emits `SentryOverriddenEvent` with red audit badge in the UI.
- **Unblocking Status**: **Fully Unblocked**. UI builds against `client.overrideSentry()`.

---

### Seam 4: Gauntlet IPC / Daemon Execution Wrapper
- **Owning Subsystem**: Gauntlet (`gauntlet`)
- **Current Implemented Surface**: Headless CLI `gauntlet execute --invocation-id <inv> --ruler <path>`. Emits exit codes, report files, and test output.
- **Missing Operation**: Long-running JSON-RPC daemon or Go sub-process manager streaming real-time phase updates (Orchestrator $\to$ Builder $\to$ Critic $\to$ Verifier) over IPC.
- **Smallest Proposed Extension**:
  In `apps/godspeed-casework-go/internal/executor/gauntlet_runner.go`:
  - Wrap `os/exec.CommandContext("gauntlet", ...)` with stdout/stderr line-by-line scanners.
  - Parse JSON log events (`{"phase": "critic", "progress": 0.5}`) and forward directly to the Go event stream.
- **Current Mock Representation**: Implemented in `MockCaseworkAdapter.executeGovernedWork()`:
  - Simulates 4 discrete phases over a 4-second interval with SSE progress events:
    - $t+1s$: `Orchestrator: Bounding task units against frozen reference`
    - $t+2s$: `Builder: Generating file transformations in isolated worktree`
    - $t+3s$: `Critic: Comparing transformations against answer key`
    - $t+4s$: `Verifier: Executing deterministic proof suite`
  - Yields final `ExecutionObservation` and SHA-256 evidence digest.
- **Unblocking Status**: **Fully Unblocked**. UI experience is identical to live daemon streaming.

---

### Seam 5: RealityTrace Real-Time Observation Streaming
- **Owning Subsystem**: RealityTrace (`sxr`)
- **Current Implemented Surface**: Python CLI tools (`sxr-resolve.py`, `scripts/validate-plugin.py`), on-disk manifests, and question-claim ledgers.
- **Missing Operation**: Real-time websocket or SSE endpoint emitting discrepancy updates as file system modifications occur.
- **Smallest Proposed Extension**:
  In `apps/godspeed-casework-go/internal/trace/sxr_watcher.go`:
  - Run `fsnotify` file watcher on `.sxr/observations/` directory.
  - When a new observation file is committed, read the JSON and emit an `ObservationRecordedEvent`.
- **Current Mock Representation**: Implemented in `MockCaseworkAdapter` using pre-compiled synthetic SXR manifests and discrepancy matrices.
- **Unblocking Status**: **Fully Unblocked**.

---

### Seam 6: Go Front End Live SSE Stream with Resync
- **Owning Subsystem**: Go System Front End (`apps/godspeed-casework-go`)
- **Current Implemented Surface**: Static HTTP fixture server (`cmd/godspeed-casework/main.go`, `internal/server/server.go`).
- **Missing Operation**: Complete HTTP SSE handler (`GET /api/v1/cases/:id/events`) implementing:
  - `Last-Event-ID` header parsing.
  - Monotonic cursor replay from in-memory ring buffer.
  - Resync fallback (`RESYNC_REQUIRED`) when client cursor is too old.
- **Smallest Proposed Extension**:
  - Implemented in `internal/server/sse_handler.go` (described in detail in Document 03).
- **Current Mock Representation**: Implemented in TypeScript `MockCaseworkAdapter.subscribeEvents()`:
  - Supports `since_cursor` replay.
  - Emits heartbeats every 15 seconds.
  - Simulates network disconnects and resynchronization in automated tests.
- **Unblocking Status**: **Fully Unblocked**.

---

## 3. Summary of Real vs. Mocked Surfaces

| Subsystem | Feature Area | Status Today | Mocked Today | Blocks UI? |
| :--- | :--- | :--- | :--- | :--- |
| **SEA-Forge** | 24 Core SFWP Methods | **Production Rust** | Available in Mock | **NO** |
| **SEA-Forge** | Discretionary Work Addition | Proposed Extension | Simulated in Mock | **NO** |
| **SEA-Forge** | Human Task Decision Inbox | Proposed Extension | Simulated in Mock | **NO** |
| **SEA-Forge** | Sentry Override | Proposed Extension | Simulated in Mock | **NO** |
| **Gauntlet** | Headless CLI Execution | **Production CLI** | Wrapped in Mock | **NO** |
| **Gauntlet** | Real-time Daemon IPC | Proposed Extension | Emulated in Mock | **NO** |
| **RealityTrace** | Manifest & Evidence Hashing | **Production Python/CLI** | Emulated in Mock | **NO** |
| **Go Front End** | Type Definitions & DTOs | **Complete Specification** | Fully Implemented | **NO** |
| **Go Front End** | In-Memory Lease Manager | Specified | Implemented in Mock | **NO** |
| **TypeScript** | Universal Client & Mock Adapter | **Fully Implemented** | **100% Ready** | **NO** |
