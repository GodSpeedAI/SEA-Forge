# 05 - Identity Correlation and Temporal Specification

## 1. Executive Summary & Foundational Invariants

In the GodSpeed Casework and Cognitive Environment, five independent subsystems interact:
1. **SEA-Forge (`sea-rs`)**: Governed case lifecycle, sentry logic, plan item activation, authority, append-only ledgers, operational settlement.
2. **Gauntlet (`gauntlet`)**: Bounded execution runtime, role orchestration, transformation, verification, and execution observation.
3. **RealityTrace (`sxr`)**: Declared vs observed world graph, questions ($q$), claims ($clm$), evidence ($evi$), 9-locus diagnostic attribution, cryptographic audit.
4. **Go System Front End (`apps/godspeed-casework-go`)**: Lease management, preflight checks, event streaming, reconciliation, cognitive projection assembly.
5. **React Cognitive Environment (`apps/godspeed-cognitive-ui`)**: Human/agent 2.5D spatial projection, timeline scrubbing, semantic action execution without CMMN jargon.

To make these subsystems operate as a unified, truthful whole without semantic collapse, **identity correlation and temporal reconstruction must be deterministic, mathematically rigorous, and auditable.**

### Core Identity Axioms
1. **One Subsystem Owns Each Identifier**: No layer may invent or redefine another layer's primary key.
2. **Deterministic Correlation Chains**: Every execution event, observation, and settlement must be traversable back to its originating governed plan item and case definition.
3. **No Uncorrelated Side Effects**: No Gauntlet run or RealityTrace observation may enter case ledgers without a cryptographic invocation proof (`inv-<id>`) issued under an active case lease (`lease-<id>`).
4. **Temporal Monotonicity**: Case events, world projections, and event streams carry strictly monotonic sequence numbers (`cursor = <epoch>.<seq>`). Historical views never mutate prior recorded states.
5. **Truthful Temporal Absence**: If historical state at cursor $T$ was not recorded or is incomplete, the system projects an explicit `HistoricalGapIndicator` rather than interpolating or fabricating state.

---

## 2. Canonical Identity Grammar & Ownership Matrix

| Entity | Canonical ID Grammar / Pattern | Owning Subsystem | Generation Mechanism | Immutability |
| :--- | :--- | :--- | :--- | :--- |
| **Case** | `case-[a-z0-9_.-]{4,64}` | SEA-Forge | UUIDv4 or domain slug upon `case_create` | Immutable |
| **Case Episode** | `ep-<case-id>-[0-9]{3}` | SEA-Forge | Incremented on case reopen / new cycle | Append-only |
| **Plan Item** | `pi-<case-id>-[a-z0-9_]{3,32}-[0-9]{3}` | SEA-Forge | Case definition / Dynamic expansion | Fixed in Plan |
| **Execution Opportunity**| `opp-<plan-item-id>-[a-f0-9]{8}`| Go Front End | Synthesized when Plan Item is `ENABLED` | Ephemeral |
| **Execution Lease** | `lease-<opp-id>-[a-f0-9]{8}` | Go Front End | Issued on `AcquireLease` with TTL | Leased / Expired |
| **Authorized Invocation**| `inv-<lease-id>-[a-f0-9]{16}` | SEA-Forge (E5A) | Signed authority token for execution | Single-use |
| **Gauntlet Run** | `run-[a-z0-9_.-]+-[0-9]{14}` | Gauntlet | Timestamped run slug upon start | Immutable |
| **Gauntlet Unit / Task**| `unit-[a-z0-9_]+-[0-9]{3}` | Gauntlet | Task decomposition within plan | Immutable |
| **RealityTrace Question**| `q-[a-f0-9]{12}` | RealityTrace | Hash of question predicate & target entity | Immutable |
| **RealityTrace Claim** | `clm-[a-f0-9]{12}` | RealityTrace | Hash of declared proposition | Immutable |
| **Evidence Record** | `evi-[a-f0-9]{16}` | RealityTrace / SEA | SHA-256 digest of artifact payload | Immutable |
| **Observation Record** | `obs-[a-f0-9]{16}` | RealityTrace | Recorded observation from execution | Append-only |
| **Settlement Record** | `set-<case-id>-[a-f0-9]{12}` | SEA-Forge (E6) | Ledger event on case acceptance/rejection | Append-only |
| **GitHub Reference** | `gh-[a-z0-9_.-]+-[a-z0-9_.-]+-(pr\|issue\|commit)-[a-zA-Z0-9]+` | GitHub | Remote repository identifier | External Source |
| **World Snapshot** | `ws-<case-id>-<epoch>.<seq>` | Go Front End | Assembly of current projection state | Point-in-time |

---

## 3. Deterministic Identity Correlation Chains

### 3.1 The Forward Governance-to-Settlement Chain

When governed work progresses from inception to completion and settlement, the identity chain binds each tier deterministically:

```text
SEA-Forge Case: `case-auth-v2-001`
  │
  ├── Episode: `ep-case-auth-v2-001-001`
  │     │
  │     └── Plan Item: `pi-case-auth-v2-001-stage_impl-004` (Task: "Verify Ed25519 Bridge")
  │           │
  │           ▼ (Go detects status: ENABLED)
  │         Opportunity: `opp-pi-case-auth-v2-001-stage_impl-004-9a8b7c6d`
  │           │
  │           ▼ (Worker claims opportunity with 300s TTL)
  │         Lease: `lease-opp-pi-case-auth-v2-001-stage_impl-004-9a8b7c6d-f1e2d3c4`
  │           │
  │           ▼ (SEA-Forge validates role authority & emits E5A)
  │         Authorized Invocation: `inv-lease-opp-pi-...-7b8c9d0e1f2a3b4c`
  │           │
  │           ▼ (Gauntlet initiates bounded run)
  │         Gauntlet Run: `run-ed25519-bridge-20260920101500`
  │           │
  │           ├── Unit: `unit-critic-001`
  │           ├── RealityTrace Question: `q-ed25519-sig-valid`
  │           ├── RealityTrace Claim: `clm-sig-verifies-with-fixture`
  │           └── RealityTrace Evidence: `evi-5a1b2c3d4e5f6a7b` (Artifact SHA: `sha256:5a1b...`)
  │                 │
  │                 ▼ (Gauntlet finishes execution & emits E5B ExecutionObservation)
  │               Observation Bundle: `obs-8c9d0e1f2a3b4c5d`
  │                 │
  │                 ▼ (Go submits bundle to SEA-Forge for Sentry & Policy evaluation)
  │               Operational Settlement: `set-case-auth-v2-001-4b5c6d7e8f9a` (E6 Event)
  │                 │
  │                 ▼ (SEA-Forge transitions Plan Item to COMPLETED, Stage advances)
  │               World Projection: `ws-case-auth-v2-001-1.042`
```

### 3.2 Backward Provenance Resolution

Given any settled claim or artifact in the Cognitive Environment, a user or auditing agent can deterministically resolve its entire ancestry without guesswork:

```json
{
  "settlement_id": "set-case-auth-v2-001-4b5c6d7e8f9a",
  "settlement_decision": "ACCEPTED",
  "provenance": {
    "case_id": "case-auth-v2-001",
    "episode_id": "ep-case-auth-v2-001-001",
    "plan_item_id": "pi-case-auth-v2-001-stage_impl-004",
    "lease_id": "lease-opp-pi-case-auth-v2-001-stage_impl-004-9a8b7c6d-f1e2d3c4",
    "invocation_id": "inv-lease-opp-pi-...-7b8c9d0e1f2a3b4c",
    "execution": {
      "subsystem": "gauntlet",
      "run_id": "run-ed25519-bridge-20260920101500",
      "exit_code": 0,
      "role": "verifier",
      "unit_id": "unit-critic-001"
    },
    "reality_trace": {
      "question_id": "q-ed25519-sig-valid",
      "claim_id": "clm-sig-verifies-with-fixture",
      "evidence_id": "evi-5a1b2c3d4e5f6a7b",
      "artifact_digest": "sha256:5a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b",
      "discrepancy_score": 0.0,
      "attribution_locus": "Locus0_TargetArtifact"
    },
    "github_correlation": {
      "commit_sha": "7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b",
      "pr_number": 42,
      "repo": "GodSpeedAI/sea-rs"
    }
  }
}
```

---

## 4. Temporal & Versioning Grammar

### 4.1 Monotonic Cursor Grammar
Event streaming and snapshot generation use a two-tuple string cursor:
$$\text{cursor} = \langle\text{epoch}\rangle.\langle\text{seq}\rangle$$
- **`epoch`** (`uint32`): Monotonically incremented whenever the case engine restarts, completes a major migration, or undergoes recovery. In normal steady state, `epoch = 1`.
- **`seq`** (`uint64`, zero-padded to 10 digits): Monotonically incremented for every single committed event in the case's append-only ledger.
- **Example**: `"1.0000000042"`

### 4.2 Comparison Operator Semantics
Given two cursors $C_1 = \langle e_1, s_1 \rangle$ and $C_2 = \langle e_2, s_2 \rangle$:
$$C_1 < C_2 \iff (e_1 < e_2) \lor (e_1 = e_2 \land s_1 < s_2)$$
$$C_1 = C_2 \iff (e_1 = e_2 \land s_1 = s_2)$$

### 4.3 Temporal Query API Specification

```typescript
export interface TemporalQuery {
  case_id: string;
  // Specific point in time
  at_cursor?: string;
  at_timestamp?: string; // RFC3339

  // Interval query for history scrubbing
  from_cursor?: string;
  to_cursor?: string;

  // Maximum events or snapshots to return in scrub trajectory
  limit?: number;
}

export interface TemporalTrajectoryResponse {
  case_id: string;
  base_cursor: string;
  head_cursor: string;
  points: TemporalCheckpoint[];
}

export interface TemporalCheckpoint {
  cursor: string;
  timestamp: string;
  event_type: string;
  summary: string;
  actor_id: string;
  actor_role: string;
  consequential: boolean;
  active_stage_id?: string;
  completed_plan_items_count: number;
  total_plan_items_count: number;
}
```

---

## 5. Truthful Historical Reconstruction Rules

When the user drags the timeline slider in the Cognitive Environment to scrub through past states, the Go System Front End executes deterministic reconstruction:

1. **Exact Checkpoint Match**: If an immutable snapshot exists at exactly `at_cursor`, it is served directly.
2. **Replay from Baseline**: If no snapshot exists at `at_cursor`, the Go layer loads the nearest preceding snapshot $S_{\le C}$ and replays the append-only ledger events up to $C$.
3. **External System Traversal (RealityTrace & Gauntlet)**:
   - For any plan item active at cursor $C$, the Go layer queries RealityTrace using the immutable `evidence_id` and `question_id` bound to that cursor.
   - External data that was generated *after* cursor $C$ is **strictly masked** out of the projection. The UI sees only what was known to the case at cursor $C$.
4. **Historical Gap Handling**:
   If an external dependency (e.g., historical Gauntlet task stdout log or temporary sandbox volume) has been pruned or purged according to retention policy:
   ```typescript
   export interface PrunedHistoricalEntity {
     id: string;
     kind: 'artifact' | 'log_stream' | 'sandbox_diff';
     status: 'PURGED_RETENTION_POLICY';
     digest_at_settlement: string; // The cryptographic proof is retained forever
     pruned_at: string;
     retrieval_ref?: string; // Cold storage pointer if available
   }
   ```
   **Under no circumstances will the system fabricate or extrapolate missing logs or intermediate state.**

---

## 6. SXR 9-Locus Attribution to Case Impact Mapping

When RealityTrace identifies an execution discrepancy between expectation $G_{\text{declared}}$ and observation $G_{\text{observed}}$, it categorizes the defect into one of 9 diagnostic loci (`Locus0` through `Locus8`). The Go Front End and Cognitive Environment translate these loci into immediate case affordances:

| SXR Locus | Technical Meaning | Cognitive Environment Translation | Case Impact / Action |
| :--- | :--- | :--- | :--- |
| **L0: TargetArtifact** | The generated file, code, or binary failed functional assertion. | *"Work produced an incorrect output."* | Plan Item `FAILED`. Re-execute with corrected parameters or add discretionary defect remediation task. |
| **L1: TestHarness** | The test runner or test fixture itself was broken or invalid. | *"Evaluation fixture is flawed."* | Escalate to Case Architect. Plan Item suspended pending harness repair. |
| **L2: RuntimeEnv** | Missing system dependency, OOM, timeout, or OS error. | *"Execution environment error (system failure)."* | Infrastructure error. Lease reset; retry permitted without consuming case retry budget. |
| **L3: ModelInference** | LLM hallucination, syntax error, or refusal. | *"Execution agent failed to follow prompt instruction."* | Critic residual generated. Re-dispatch to alternative model or human fallback. |
| **L4: RoleProtocol** | Orchestrator/Builder/Critic protocol violation in Gauntlet. | *"Agent coordination protocol error."* | Gauntlet run marked `INVALID`. Trace recorded in audit log. |
| **L5: SpecBoundary** | Specification conflict or missing prerequisite contract. | *"Requirements contradiction discovered."* | Case paused; triggers `AMBIGUITY_DETECTED` event. Requires human authority intervention. |
| **L6: SentryCondition**| SEA-Forge sentry evaluation failed due to missing prior milestone. | *"Prerequisites were not satisfied."* | Sentry blocks activation. UI indicates: *"Waiting on prior stage evidence."* |
| **L7: AuthorityPolicy**| Actor attempted action outside granted policy boundaries. | *"Action blocked by security policy."* | Authority violation event recorded. Security review notification dispatched. |
| **L8: ExternalWorld** | GitHub API rate limit, network partition, upstream outage. | *"External service unavailable."* | Case item placed in `BACKOFF_WAITING`. Automatic retry scheduled. |

---

## 7. Verification and Test Vectors

The integrity of this identity and temporal model is verified by automated test suites validating:
- **Lexical conformity**: Regex validation of all ID patterns across generated payloads.
- **Chain completeness**: Verifying that any `set-*` settlement record contains complete non-empty references to `case`, `plan_item`, `lease`, `invocation`, `run`, `evidence`, and `observation`.
- **Monotonic progression**: Replaying synthetic event streams to ensure cursors are strictly increasing and that scrubbing backwards produces bit-identical world projections.
- **Locus mapping**: Asserting that SXR error outputs deterministically yield the exact prescribed cognitive statuses and action affordances.
