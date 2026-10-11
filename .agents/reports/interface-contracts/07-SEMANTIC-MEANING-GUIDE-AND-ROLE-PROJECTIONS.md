# 07 - Semantic Meaning Guide and Role Projections

## 1. Cognitive UI Translation Dictionary

The fundamental mandate of the Cognitive Environment is to eliminate CMMN, case engine, SFWP, and Gauntlet mechanics from the human and agent experience. The user is operating their **purposeful work and world**, not navigating an engine.

### Core Terminology Translation Table

| Backend / Engine Concept | Internal Identifier / State | Human Cognitive Translation | UX Presentation / Placement |
| :--- | :--- | :--- | :--- |
| **Case Instance** | `CaseInstance{id, definition}` | **Workspace** or **Project Context** | Top-level context header, environment breadcrumb |
| **Case Definition** | `CaseDefinition{model_ref}` | **Operating Playbook** or **Blueprint** | Context settings, template metadata |
| **Plan Item (Task)** | `PlanItem{kind: Task, state: WAITING}` | **Upcoming Work** | Grayed out card with prerequisite tooltip |
| **Plan Item (Enabled)** | `PlanItem{kind: Task, state: ENABLED}` | **Ready to Begin** | High-contrast actionable card with primary button |
| **Plan Item (Active)** | `PlanItem{kind: Task, state: ACTIVE}` | **In Progress** | Pulsing border, active worker pill, progress bar |
| **Plan Item (Completed)**| `PlanItem{kind: Task, state: COMPLETED}`| **Done & Verified** | Muted card with green checkmark and evidence link |
| **Plan Item (Failed)** | `PlanItem{kind: Task, state: FAILED}` | **Needs Attention / Blocked** | Amber alert badge, failure summary, retry affordance |
| **Stage** | `PlanItem{kind: Stage}` | **Milestone Phase** or **Work Track** | Horizontal phase band, collapsible container |
| **Milestone** | `PlanItem{kind: Milestone}` | **Target Achievement** | Milestone flag icon, completion badge |
| **Sentry (Entry Criterion)**| `Sentry{if_part, on_part}` | **Prerequisites** | *"Requires: Security Review & Test Pass"* |
| **Sentry (Exit Criterion)** | `Sentry{exit_criteria}` | **Completion Gates** | *"Gates to finish phase: 100% tests green"* |
| **Discretionary Item** | `DiscretionaryItem{planning_table}` | **Optional Work / Available to Add** | Dashed border card: *"Add this work if needed"* |
| **Human Task (Approval)** | `HumanTask{type: Approval}` | **Decision Needed / Needs Your Approval** | Prominent banner with [Approve] and [Reject] buttons |
| **Settlement (Accepted)** | `SettlementEvent{ACCEPTED}` | **Accepted & Proven** | Cryptographic verification badge |
| **Settlement (Rejected)** | `SettlementEvent{REJECTED}` | **Verification Rejected** | Discrepancy report drawer with exact diff |
| **Gauntlet Run** | `GauntletRun{run_id, units}` | **Automated Execution** | Expandable execution transcript card |
| **RealityTrace Discrepancy**| `SXR_Residual{locus, delta}` | **Unresolved Finding** | Interactive discrepancy card with diff viewer |
| **Execution Lease** | `Lease{expires_at, worker_id}` | **Claimed Work** | *"Being handled by Worker-01 (Expires in 3m)"* |
| **Case Reopening** | `CaseReopen{new_episode}` | **Reopen Work Context** | Context menu action: *"Reopen for remediation"* |

---

## 2. Role-Aware Projection Architecture

SEA-Forge enforces strict Role-Based Authority (RBAC) and Separation of Duties (SoD). Rather than dumping raw permissions or failing actions with `403 Forbidden`, the Go System Front End projects an **actor-tailored cognitive world**.

```text
                               SEA-Forge Case
                  (Full authoritative state & permission graph)
                                      │
                                      ▼
                             Go System Front End
                          (Role Projection Filter)
                                      │
          ┌───────────────────────────┼───────────────────────────┐
          ▼                           ▼                           ▼
   Developer Role             Security Officer             Auditor Role
 ┌──────────────────────┐   ┌──────────────────────┐   ┌──────────────────────┐
 │ - "Ready to begin"   │   │ - "Needs your review"│   │ - "Read-only view"   │
 │ - Can claim work     │   │ - [Approve] [Reject] │   │ - Cryptographic trail│
 │ - Cannot approve PR  │   │ - Security gates     │   │ - No mutate buttons  │
 │ - Discretionary tasks│   │ - Policy overrides   │   │ - Audit export tool  │
 └──────────────────────┘   └──────────────────────┘   └──────────────────────┘
```

---

## 3. Comprehensive Role Projection Matrix

| Cognitive Capability / Surface | Developer / Agent Operator | Security Officer | Case Architect / Admin | Auditor / Observer | Customer / Stakeholder |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **View Active Work Items** | Full visibility | Full visibility | Full visibility | Full visibility | High-level summary only |
| **Claim & Execute Work** | **Enabled** (via Lease) | Disabled | **Enabled** | Disabled (Read-only) | Disabled |
| **Approve Human Tasks** | Disabled (SoD) | **Enabled** (for security tasks) | **Enabled** (for admin gates) | Disabled | **Enabled** (for signoffs) |
| **Add Discretionary Tasks**| **Enabled** (within stage) | **Enabled** (security scope) | **Enabled** (all planning tables) | Disabled | Disabled |
| **Override Sentry Gates** | Disabled | **Enabled** (with justification) | **Enabled** (full override) | Disabled | Disabled |
| **Reopen Completed Work** | Disabled | **Enabled** (if defect found) | **Enabled** | Disabled | Disabled |
| **Inspect RealityTrace Diffs**| Full diff & test logs | Security audit diffs | Full diff & test logs | Cryptographic proofs only | High-level pass/fail summary |
| **Manage Case Blueprint** | Disabled | Disabled | **Enabled** (import/export model)| Disabled | Disabled |
| **Terminate / Cancel Case**| Disabled | Disabled | **Enabled** (admin only) | Disabled | Disabled |
| **Export Audit Bundle** | Disabled | Security evidence bundle | Full compliance ledger | **Full cryptographic export** | Status report PDF |

---

## 4. Side-by-Side Projection Comparison

To illustrate how the exact same underlying case state projects differently to different actors, consider Case `case-auth-v2-001` at Step 4: *A PR has been built and tested by Gauntlet, and now awaits security approval and discretionary penetration testing before the milestone can be closed.*

### 4.1 Developer View (`actor_role = "developer"`)

```json
{
  "world_id": "ws-case-auth-v2-001-42.0000000001",
  "case_id": "case-auth-v2-001",
  "cursor": "1.0000000042",
  "timestamp": "2026-09-20T10:15:00Z",
  "perspective": {
    "actor_id": "usr-dev-alice",
    "role": "developer"
  },
  "summary": {
    "headline": "Authentication Gateway Refactor",
    "phase": "Review & Hardening",
    "status_phrase": "Waiting on security officer review"
  },
  "visible_objects": [
    {
      "id": "obj-task-ed25519-impl",
      "name": "Verify Ed25519 Bridge Implementation",
      "status": "COMPLETED",
      "badge": "Done & Verified (Gauntlet pass)",
      "salience": 0.2,
      "actions": [
        { "id": "view_logs", "label": "Inspect Test Logs", "intent": "OPEN_ARTIFACT", "consequential": false }
      ]
    },
    {
      "id": "obj-gate-security",
      "name": "Security Officer Architecture Signoff",
      "status": "WAITING_ON_OTHERS",
      "badge": "Needs Security Review",
      "explanation": "Bob (Security Officer) must approve this gate before next phase unlocks.",
      "salience": 0.7,
      "actions": []
    },
    {
      "id": "obj-disc-pen-test",
      "name": "Run Boundary Fuzzing Test Suite",
      "status": "AVAILABLE_TO_ADD",
      "badge": "Optional Work",
      "salience": 0.5,
      "explanation": "You can add this fuzzing task to the plan if extra validation is desired.",
      "actions": [
        { "id": "act_add_work", "label": "Add this work", "intent": "ADD_DISCRETIONARY_WORK", "consequential": true }
      ]
    }
  ],
  "available_actions": [
    { "id": "act_add_disc", "label": "Add Optional Fuzzing Work", "intent": "ADD_DISCRETIONARY_WORK", "consequential": true }
  ],
  "attention_focus": { "primary_object_id": "obj-gate-security" }
}
```

### 4.2 Security Officer View (`actor_role = "security_officer"`)

```json
{
  "world_id": "ws-case-auth-v2-001-sec-42.0000000042",
  "case_id": "case-auth-v2-001",
  "cursor": "1.0000000042",
  "timestamp": "2026-09-20T10:15:00Z",
  "perspective": {
    "actor_id": "usr-sec-bob",
    "role": "security_officer"
  },
  "summary": {
    "headline": "Authentication Gateway Refactor",
    "phase": "Review & Hardening",
    "status_phrase": "Action required: Security signoff pending"
  },
  "visible_objects": [
    {
      "id": "obj-task-ed25519-impl",
      "name": "Verify Ed25519 Bridge Implementation",
      "status": "COMPLETED",
      "badge": "Ready for Audit",
      "salience": 0.2,
      "actions": [
        { "id": "view_sec_diff", "label": "Inspect Cryptographic Diffs", "intent": "OPEN_ARTIFACT", "consequential": false }
      ]
    },
    {
      "id": "obj-gate-security",
      "name": "Security Officer Architecture Signoff",
      "status": "ACTION_REQUIRED",
      "badge": "Needs Your Decision",
      "salience": 1.0,
      "explanation": "Review SXR evidence bundle and approve or reject milestone advancement.",
      "actions": [
        { "id": "act_sec_approve", "label": "Approve Architecture", "intent": "APPROVE_HUMAN_TASK", "variant": "PRIMARY", "consequential": true },
        { "id": "act_sec_reject", "label": "Reject & Request Remediation", "intent": "REJECT_HUMAN_TASK", "variant": "DANGER", "consequential": true }
      ]
    },
    {
      "id": "obj-override-panel",
      "name": "Security Policy Controls",
      "status": "ACTIVE",
      "badge": "Administrative Authority",
      "salience": 0.8,
      "actions": [
        { "id": "act_sec_override", "label": "Emergency Sentry Override", "intent": "ESCALATE_OR_OVERRIDE", "variant": "WARNING", "consequential": true }
      ]
    }
  ],
  "available_actions": [
    { "id": "act_sec_approve", "label": "Approve Architecture", "intent": "APPROVE_HUMAN_TASK", "consequential": true },
    { "id": "act_sec_reject", "label": "Reject & Request Remediation", "intent": "REJECT_HUMAN_TASK", "consequential": true }
  ],
  "attention_focus": { "primary_object_id": "obj-gate-security" }
}
```

### 4.3 Auditor View (`actor_role = "auditor"`)

```json
{
  "world_id": "ws-case-auth-v2-001-aud-42.0000000042",
  "case_id": "case-auth-v2-001",
  "cursor": "1.0000000042",
  "timestamp": "2026-09-20T10:15:00Z",
  "perspective": {
    "actor_id": "usr-aud-charlie",
    "role": "auditor"
  },
  "summary": {
    "headline": "Authentication Gateway Refactor",
    "phase": "Review & Hardening",
    "status_phrase": "Read-only audit stream"
  },
  "visible_objects": [
    {
      "id": "obj-audit-trail",
      "name": "Append-Only Governance Ledger",
      "status": "COMPLETED",
      "badge": "42 Immutable Records",
      "salience": 0.3,
      "actions": [
        { "id": "view_merkle_tree", "label": "Verify Merkle Signatures", "intent": "RESOLVE_SOURCE", "consequential": false },
        { "id": "export_bundle", "label": "Download Compliance Zip", "intent": "EXPORT_AUDIT_BUNDLE", "consequential": false }
      ]
    },
    {
      "id": "obj-task-ed25519-impl",
      "name": "Verify Ed25519 Bridge Implementation",
      "status": "COMPLETED",
      "badge": "SHA-256 Verified",
      "salience": 0.2,
      "actions": [
        { "id": "view_provenance", "label": "Inspect Provenance Chain", "intent": "RESOLVE_SOURCE", "consequential": false }
      ]
    }
  ],
  "available_actions": [
    { "id": "export_bundle", "label": "Export Signed Audit Bundle", "intent": "EXPORT_AUDIT_BUNDLE", "consequential": false }
  ],
  "attention_focus": { "primary_object_id": "obj-audit-trail" }
}
```

---

## 5. Interaction Intent Resolution Protocol

When a user clicks an action in the UI, the React Cognitive Environment generates an `InteractionIntent`:

```typescript
export interface InteractionIntent {
  intent_id: string; // uuid
  kind: 'CONSEQUENTIAL_CASE' | 'BACKEND_INFORMATION' | 'REACT_LOCAL';
  action_name: InteractionActionName; // e.g., "APPROVE_HUMAN_TASK"
  target_object_id: string;
  case_id: string;
  client_cursor: string;
  actor: {
    actor_id: string;
    role: string;
  };
  parameters: Record<string, unknown>;
}
```

### Evaluation Pipeline in Go System Front End:
1. **Local Intents**: Handled entirely within React (camera movement, node selection, panel expansion). Zero network traffic.
2. **Information Intents**: Go queries the read cache or RealityTrace without acquiring a write lock (e.g., retrieving artifact markdown).
3. **Consequential Intents**:
   - Go matches the intent against the active `ActionDescriptor` emitted in the current world snapshot.
   - Go verifies the client's monotonic cursor is not stale.
   - Go submits the signed command to SEA-Forge.
   - SEA-Forge admits the command into the append-only ledger and emits a state transition event.
   - Go streams the updated projection to all connected clients.
