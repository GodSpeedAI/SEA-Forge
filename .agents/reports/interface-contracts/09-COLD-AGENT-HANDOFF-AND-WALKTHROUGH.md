# 09 - Cold-Agent Handoff and Walkthrough Manual

## 1. Welcome to the GodSpeed Cognitive Environment

If you are a frontend developer, UI agent, or client implementer picking up this repository cold:
**You do not need to open Rust crates, inspect CMMN XML models, reverse-engineer Gauntlet harnesses, or parse RealityTrace Python scripts.**

The entire application operating boundary has been modeled, typed, and packaged into a single directory:
`/.agents/reports/interface-contracts/`

This document provides clear, concise, and definitive answers to the 10 canonical operating questions, followed by a 5-minute fast start guide.

---

## 2. The 10 Canonical Operating Questions

### Q1: What world can I query?
**Answer**: You query the **Cognitive World Snapshot** (`CognitiveWorldSnapshot`) for a specific case. The world represents a unified, human-understandable state of purposeful work at a specific point in time (identified by a monotonic `cursor` like `"1.0000000042"`). It contains the workspace headline, active phase, visible work objects, spatial/salience layouts, and permitted action descriptors.

### Q2: What objects can appear?
**Answer**: The world contains typed **Cognitive Objects** (`CognitiveObject`). Each object belongs to one of seven semantic kinds:
1. `work_item`: A task ready to be executed, in progress, or completed.
2. `stage`: A phase container grouping related work.
3. `milestone`: A critical goal achievement.
4. `decision_gate`: A human signoff or policy approval.
5. `discretionary_opportunity`: Optional work available to add to the plan.
6. `execution_trace`: An active or past Gauntlet automated run.
7. `evidence_record`: A cryptographic proof or verification artifact.

### Q3: What actions can a user take?
**Answer**: A user can take only the actions explicitly listed in the object's `actions` array (`ActionDescriptor`). Actions include:
- `BEGIN_WORK`: Claim an enabled task and initiate execution.
- `APPROVE_HUMAN_TASK`: Grant signoff on an approval gate.
- `REJECT_HUMAN_TASK`: Deny approval and require remediation.
- `ADD_DISCRETIONARY_WORK`: Instantiate optional work from a planning table.
- `REOPEN_WORK`: Re-activate a completed task following new observations.
- `ESCALATE_OR_OVERRIDE`: Administrative sentry bypass or policy escalation.
- `OPEN_ARTIFACT`: Inspect logs, diffs, or evidence without mutating state.

### Q4: How do role differences appear?
**Answer**: Role filtering is performed **before** the snapshot reaches the frontend.
- When connected as a **Developer**, you see tasks marked *"Ready to begin"*, can acquire leases, and see optional tasks to add. Approval buttons on security gates are hidden or disabled.
- When connected as a **Security Officer**, you receive prominent *"Needs your decision"* banners with `[Approve]` and `[Reject]` buttons, alongside sentry override tools.
- When connected as an **Auditor**, all mutation buttons are stripped; you receive cryptographic export tools and immutable Merkle ledger viewers.
The UI never needs to calculate RBAC rules; it simply renders the `actions` array present on each object.

### Q5: How does governed work become execution?
**Answer**: Through the 4-edge convergence loop:
1. When a task is `ENABLED`, an actor dispatches a `BEGIN_WORK` intent.
2. The Go Front End acquires an `ExecutionLease` (`lease-...`).
3. SEA-Forge validates policy and issues an `AuthorizedInvocation` token (`inv-...`, Edge E5A).
4. Gauntlet receives the token and launches an isolated execution run (`run-...`).
5. Upon completion, Gauntlet emits an `ExecutionObservation` (Edge E5B).
6. SEA-Forge settles the work (`set-...`, Edge E6) and advances the case.

### Q6: How do I observe a Gauntlet run?
**Answer**: Gauntlet runs are projected as `CognitiveObject{kind: "execution_trace"}`. The standard object carries its name, status, badge, salience, and permitted actions. Live execution detail is carried by the typed progress payload:
- `phase`: `"orchestrator" | "builder" | "critic" | "verifier" | "settling"`
- `progress_percent`: `0.0` to `1.0`
- `log_line`: One-line plain English summary of current activity
Progress events stream continuously over the SSE connection without polling.

### Q7: How do I inspect evidence?
**Answer**: Click an action with intent `OPEN_ARTIFACT`. Call `client.resolveArtifact(evidence_id)`.
You receive an `ArtifactPayload` containing:
- `digest`: The SHA-256 cryptographic hash (e.g., `sha256:5a1b...`).
- `content_type`: `"text/markdown" | "application/json" | "text/x-diff"`.
- `content`: The raw text or data.
- `provenance`: The full chain linking the evidence back to the originating case, plan item, Gauntlet run, and SXR question.

### Q8: How do I move through history?
**Answer**: Call `client.queryTemporalTrajectory(case_id)` to retrieve timeline checkpoints, then call `client.getSnapshotAt(case_id, target_cursor)`.
The Go Front End reconstructs the exact world state as it existed at that cursor.
- The UI renders in *"Historical Inspection Mode"* (banner: *"Viewing state as of 2026-09-20 10:15 UTC"*).
- All consequential mutation actions are disabled.
- An action *"Return to Live Head"* restores real-time streaming.

### Q9: How do I distinguish completion from settlement?
**Answer**:
- **Completion** (`status: "EXECUTION_COMPLETE"`): Gauntlet finished running and exited with code 0. The output files exist, but have **not yet been verified** by the governing case.
- **Settlement** (`status: "SETTLED_ACCEPTED"` or `"SETTLED_REJECTED"`): SEA-Forge has cryptographically verified the evidence against declared criteria and formally recorded the result in the append-only ledger.
The UI represents completion with a blue spinning gear (*"Evaluating results..."*) and settlement with a solid green shield (*"Verified & Settled"*).

### Q10: How do I invoke a consequential action?
**Answer**: Call `client.dispatchIntent(intent)`.
```typescript
const result = await client.dispatchIntent({
  intent_id: crypto.randomUUID(),
  kind: 'CONSEQUENTIAL_CASE',
  action_name: 'BEGIN_WORK',
  target_object_id: 'obj-task-ed25519-impl',
  case_id: 'case-auth-v2-001',
  client_cursor: '1.0000000042',
  actor: { actor_id: 'usr-dev-alice', role: 'developer' },
  parameters: { requested_ttl_seconds: 300 }
});
```
The server validates your cursor, checks authority, executes the state transition, and broadcasts the updated snapshot to all connected clients.

---

## 3. Five-Minute Fast Start Guide

### Step 1: Install or Import the Client
```bash
# In apps/godspeed-cognitive-ui/
import { createCaseworkClient, MockCaseworkAdapter } from './contracts/typescript';
```

### Step 2: Initialize with Mock Adapter (Instant Local Development)
```typescript
// Uses the zero-dependency rich mock that simulates full case lifecycles
const adapter = new MockCaseworkAdapter();
const client = createCaseworkClient(adapter);
```

### Step 3: Subscribe to the Live World Stream
```typescript
const unsubscribe = client.subscribeWorld(
  'case-auth-v2-001',
  'usr-dev-alice',
  'developer',
  (snapshot) => {
    console.log(`World Cursor: ${snapshot.cursor}`);
    console.log(`Headline: ${snapshot.headline}`);
    renderSpatialStage(snapshot.visible_objects);
  },
  (error) => {
    console.error('Stream error:', error);
  }
);
```

### Step 4: Dispatch an Interaction Intent
```typescript
async function handleCardClick(
  object: CognitiveObject,
  action: ActionDescriptor,
  snapshot: CognitiveWorldSnapshot,
) {
  const response = await client.dispatchIntent({
    intent_id: crypto.randomUUID(),
    kind: action.consequential ? 'CONSEQUENTIAL_CASE' : 'BACKEND_INFORMATION',
    action_name: action.intent,
    target_object_id: object.id,
    case_id: 'case-auth-v2-001',
    client_cursor: snapshot.cursor,
    actor: { actor_id: 'usr-dev-alice', role: 'developer' },
    parameters: {}
  });

  if (!response.success) {
    alert(`Action rejected: ${response.error_message}`);
  }
}
```

---

## 4. Architectural Boundaries: What You Must NOT Do

1. **NEVER compute authorization in React**: Do not write `if (role === 'admin') showButton()`. If an action is permitted, it will be in `object.actions`.
2. **NEVER treat exit code 0 as success**: Wait for `SettlementEvent` before marking a task done.
3. **NEVER parse internal ID strings**: Treat IDs as opaque tokens. Use typed relationships (`parent_id`, `depends_on`).
4. **NEVER mutate history**: When scrubbed back in time, all API calls are read-only.
