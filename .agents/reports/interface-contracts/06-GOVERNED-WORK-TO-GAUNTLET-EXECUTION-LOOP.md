# 06 - Governed Work to Gauntlet Execution Loop

## 1. Architecture and State Machine Overview

The heart of the GodSpeed casework operating system is the **Governed Work Execution Loop**. It connects four independent subsystems across formal convergence edges:

```text
       ┌─────────────────────────────────────────────────────────┐
       │                SEA-Forge Case Subsystem                 │
       │  - Governs plan, sentries, authority, settlement       │
       └──────┬──────────────────────────────────────────▲───────┘
              │                                          │
              │ E5A: AuthorizedInvocation                │ E6: OperationalSettlement
              │ (Signed capability token)                │ (Accepted / Rejected / Inconclusive)
              ▼                                          │
       ┌──────────────────────────────┐                  │
       │    Go System Front End       ├──────────────────┘
       │  - Lease, preflight, stream  │
       └──────┬────────────────▲──────┘
              │                │
              │ Gauntlet Run   │ E5B: ExecutionObservation
              │ Contract       │ (Artifacts, exit code, residuals)
              ▼                │
       ┌───────────────────────┴──────┐
       │     Gauntlet Runtime         │
       │  - Orchestrator/Builder/     │
       │    Critic/Verifier loop      │
       └──────────────┬───────────────┘
                      │
                      │ Observations, Expectations, Artifacts
                      ▼
       ┌──────────────────────────────┐
       │     RealityTrace (sxr)       │
       │  - G_declared vs G_observed  │
       │  - Evidence digest, 9-locus  │
       └──────────────────────────────┘
```

---

## 2. Step-by-Step Execution Lifecycle

The lifecycle proceeds through 9 deterministic, auditable phases:

### Phase 1: Work Becomes Available in SEA-Forge
1. Sentry criteria evaluate to `TRUE` (e.g., prior milestone satisfied, customer evidence received).
2. Plan item transitions from `WAITING` $\to$ `ENABLED`.
3. SEA-Forge emits SFWP event: `PlanItemEnabledEvent{case_id, plan_item_id, stage_id}`.

### Phase 2: Opportunity Synthesis & Discovery
1. Go System Front End ingests the event stream.
2. Go synthesizes an ephemeral `ExecutionOpportunity`:
   ```go
   type ExecutionOpportunity struct {
       OpportunityID string    // opp-<plan-item-id>-<nonce>
       CaseID        string    // case-001
       PlanItemID    string    // pi-stage_impl-004
       RequiredRole  string    // "developer" or "agent_operator"
       ContractSlug  string    // "run-ed25519-bridge"
       Status        string    // "OPEN"
       CreatedAt     time.Time
   }
   ```
3. Cognitive UI reflects: *"Ready to begin: Verify Ed25519 Bridge"*.

### Phase 3: Lease Acquisition & Concurrency Control
1. An actor (human or automated agent worker) selects the opportunity and calls Go Front End: `AcquireLease(opp_id, actor_id, role, requested_ttl)`.
2. Go verifies no active non-expired lease exists.
3. Go issues `ExecutionLease`:
   - `LeaseID`: `lease-opp-...-f1e2d3c4`
   - `ExpiresAt`: `now() + 300s`
   - `WorkerID`: `worker-node-04`
4. UI updates to other actors: *"Work in progress by worker-node-04 (Expires in 4m 58s)"*.

### Phase 4: Preflight & Authority Issuance (Edge E5A)
1. Go submits preflight check to SEA-Forge via SFWP method `case_authorize_work`:
   ```json
   {
     "jsonrpc": "2.0",
     "id": 101,
     "method": "case_authorize_work",
     "params": {
       "case_id": "case-001",
       "plan_item_id": "pi-stage_impl-004",
       "lease_id": "lease-opp-...-f1e2d3c4",
       "actor_id": "worker-node-04",
       "actor_role": "agent_operator"
     }
   }
   ```
2. SEA-Forge verifies:
   - Case is `ACTIVE` (not `SUSPENDED` or `TERMINATED`).
   - Plan item is `ENABLED` or `ACTIVE`.
   - Actor role is authorized under governing policy.
   - Idempotency check: no conflicting invocation token exists.
3. SEA-Forge returns signed **`AuthorizedInvocation`** token (Edge E5A):
   ```json
   {
     "invocation_id": "inv-lease-opp-pi-7b8c9d0e1f2a3b4c",
     "case_id": "case-001",
     "plan_item_id": "pi-stage_impl-004",
     "max_duration_seconds": 600,
     "sandbox_profile": "bounded_container_net_deny",
     "permitted_capabilities": ["fs_read", "fs_write_workspace", "cargo_test"],
     "signature": "ed25519:3a1b2c..."
   }
   ```

### Phase 5: Gauntlet Run Initiation
1. Go launches Gauntlet execution runtime via CLI or IPC:
   ```bash
   gauntlet execute \
     --invocation-id inv-lease-opp-pi-7b8c9d0e1f2a3b4c \
     --plan-item-id pi-stage_impl-004 \
     --sandbox-profile bounded_container_net_deny \
     --ruler .wayfinder/answer-key.md
   ```
2. Gauntlet generates `run_id`: `run-ed25519-bridge-20260920101500`.
3. Gauntlet initiates role orchestration:
   - **Orchestrator**: Decomposes task into units against the frozen reference.
   - **Builder**: Executes transformations in the isolated worktree.
   - **Critic**: Evaluates builder changes against answer key in fresh context.
   - **Verifier**: Runs deterministic proof commands (`cargo test -p sea-bridge`).

### Phase 6: Continuous Observation & RealityTrace Ingestion
1. During execution, Gauntlet writes traces, tool logs, and test assertions to disk.
2. RealityTrace (`sxr`) observes each change:
   - Evaluates questions: $q_{\text{functional\_pass}}$, $q_{\text{no\_secrets\_leaked}}$.
   - Calculates discrepancies between declared and observed behavior:
     $$\Delta = G_{\text{declared}} \oplus G_{\text{observed}}$$
   - If $\Delta = \emptyset$, evidence digest is signed:
     $$\text{evidence\_id} = \text{SHA256}(\text{test\_output} \parallel \text{diff} \parallel \text{ruler\_digest})$$
3. Go streams progress events to the Cognitive UI:
   - `ExecutionProgressEvent`: *"Builder modified 3 files; Verifier running test suite..."*

### Phase 7: Gauntlet Completion & Observation Emission (Edge E5B)
1. Gauntlet process terminates with exit code 0.
2. Gauntlet outputs structured **`ExecutionObservation`** bundle (Edge E5B):
   ```json
   {
     "run_id": "run-ed25519-bridge-20260920101500",
     "invocation_id": "inv-lease-opp-pi-7b8c9d0e1f2a3b4c",
     "exit_code": 0,
     "status": "SUCCEEDED",
     "duration_ms": 34500,
     "artifacts": [
       {
         "name": "cargo_test_report.json",
         "digest": "sha256:5a1b2c3d4e5f6a7b...",
         "size_bytes": 1420
       }
     ],
     "reality_trace": {
       "question_id": "q-ed25519-sig-valid",
       "evidence_id": "evi-5a1b2c3d4e5f6a7b",
       "discrepancy_count": 0,
       "attribution_locus": null
     }
   }
   ```

### Phase 8: Operational Settlement in SEA-Forge (Edge E6)
1. Go receives the execution observation and submits it to SEA-Forge via SFWP method `case_settle_work`:
   ```json
   {
     "jsonrpc": "2.0",
     "id": 102,
     "method": "case_settle_work",
     "params": {
       "case_id": "case-001",
       "plan_item_id": "pi-stage_impl-004",
       "invocation_id": "inv-lease-opp-pi-7b8c9d0e1f2a3b4c",
       "observation": { ... }
     }
   }
   ```
2. **SEA-Forge Settlement Logic**:
   - Evaluates the evidence against the plan item's declared **Evidence Requirements**.
   - Validates the cryptographic signatures on the SXR evidence bundle.
   - Confirms that Gauntlet exit code 0 was accompanied by a verified answer-key match.
   - **Records immutable Settlement Event**: `SettlementEvent{status: ACCEPTED, settlement_id: "set-001"}`.
   - Releases the execution lease.
   - Transitions Plan Item: `ACTIVE` $\to$ `COMPLETED`.

### Phase 9: Case Consequence & Projection Update
1. Dependent sentries fire (e.g., exit criteria for current stage).
2. Stage transitions from `ACTIVE` $\to$ `COMPLETED`.
3. Next stage (`stage_security_review`) becomes `ACTIVE`.
4. Go Front End publishes a new monotonic snapshot: `ws-case-001-1.043`.
5. Cognitive UI plays subtle completion animation, advances the stage track, and highlights the newly enabled action: *"Review Security Evidence"*.

---

## 3. Failure Classification: Infrastructure vs. Consequential

A critical failure in naive agent architectures is treating all nonzero exit codes or test failures identically. The GodSpeed System Front End strictly enforces the distinction between **Infrastructure Failures** and **Consequential Failures**:

| Category | Typical Symptoms | SXR Locus | Settlement Decision | Case Effect | Retry Behavior |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Infrastructure Failure** | Sandbox SIGKILL (OOM), host disk full, network timeout to provider, lease expired mid-run. | `Locus2_RuntimeEnv` or `Locus8_ExternalWorld` | `ABORTED` (Not Settled) | Plan item remains `ENABLED`. State is rolled back to pre-lease. | Immediate retry allowed with fresh lease; does NOT decrement case retry limit. |
| **Consequential Failure** | Tests compiled and executed, but assertions failed; Gauntlet Critic found regression against answer key. | `Locus0_TargetArtifact` | `REJECTED` (Settled Defect) | Plan item transitions to `FAILED`. Emits `SettlementEvent{REJECTED}`. | Increments retry counter. May trigger sentry fallback or require manual intervention. |
| **Protocol Violation** | Agent output unparseable JSON; Gauntlet Orchestrator breached role boundary. | `Locus4_RoleProtocol` | `INVALID` | Run halted immediately. Security log generated. | Requires agent prompt/role contract adjustment before retry. |
| **Ambiguity / Conflict** | Conflicting requirements discovered between two specs. | `Locus5_SpecBoundary` | `SUSPENDED` | Case pauses; raises human escalation alert in UI. | Awaits human decision or specification amendment. |

---

## 4. Anti-Collapse Invariant: Why Exit Code 0 $\ne$ Settlement

```text
       ┌────────────────────────────────────────────────────────┐
       │                  Gauntlet Exit 0                       │
       │  Means: Process completed without uncaught exception. │
       └──────────────────────────┬─────────────────────────────┘
                                  │
                                  ▼
               Is Gauntlet exit code 0 proof of success?
                                  │
         ┌────────────────────────┴────────────────────────┐
         │ NO!                                             │
         │ - A test suite might have 0 tests executed.     │
         │ - A mock might have masked a real failure.      │
         │ - A prompt might have produced empty output.    │
         │ - An unauthorized change might have occurred.   │
         └────────────────────────┬────────────────────────┘
                                  │
                                  ▼
       ┌────────────────────────────────────────────────────────┐
       │              SEA-Forge Case Settlement                 │
       │  Evaluates:                                            │
       │  1. Cryptographic SXR Evidence Digest matches claim.   │
       │  2. Output satisfies defined Acceptance Gates.         │
       │  3. Actor possessed legitimate authority.             │
       │  4. Invariant checks (no secrets, zero regress) pass.  │
       └────────────────────────────────────────────────────────┘
```

**Anti-Collapse Axiom 1**: Gauntlet execution completion is an **observation of fact**, not an authoritative settlement. Settlement is the exclusive prerogative of SEA-Forge's governed policy engine.

---

## 5. Lease Recovery, Heartbeats, and Stale Worker Mitigation

To ensure high availability and prevent orphaned locks when workers crash:

```text
Worker                               Go Front End                        SEA-Forge
  │                                       │                                  │
  │─── 1. AcquireLease(TTL: 300s) ───────►│                                  │
  │    (Lease granted: expires at T+300)  │                                  │
  │                                       │                                  │
  │─── 2. Heartbeat(every 60s) ──────────►│ (Extends lease: expires T+360)   │
  │                                       │                                  │
  ▼ [Worker Crashes / Hard Power Loss]    │                                  │
                                          ▼                                  │
                                   [Clock reaches T+360]                     │
                                   Lease marked EXPIRED                      │
                                          │                                  │
                                          │─── 3. InvalidateInvocation ─────►│
                                          │       (inv-... marked CANCELLED) │
                                          │                                  │
                                          │─── 4. Re-enable Plan Item ──────►│
                                          │       (pi-... back to ENABLED)   │
                                          │                                  │
                                          ▼                                  │
                            Emits LeaseExpiredEvent to UI                    │
                            "Worker timed out; work is available again"       │
```

1. **Heartbeat Requirement**: Active workers must send a heartbeat every $\frac{1}{3}\text{TTL}$.
2. **Reconciliation Loop**: The Go Front End runs a background reconciliation ticker (default: 10s) scanning for expired leases.
3. **Fencing Tokens**: Every invocation token is bound to a specific lease version. If a stale worker attempts to submit results after lease expiration, SEA-Forge rejects the submission with `STALE_LEASE_REJECTED`.
