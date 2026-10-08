# Conformance Report: CJ05 — Navigate and Adapt a Live Case

## Status: FAIL

### Gates Evaluation
| Gate | Status |
| --- | --- |
| ENTRY | PASS |
| VISIBILITY | FAIL |
| REACHABILITY | PASS |
| BINDING | PASS |
| AUTHORITY | PASS |
| EXECUTION | PASS |
| EVIDENCE | PASS |
| SETTLEMENT | PASS |
| CONTINUITY | PASS |
| RECOVERY | PASS |

### Failure Diagnosis
- **Code**: `AFFORDANCE_VISIBILITY_GAP`
- **Locus**: VISIBILITY
- **Expected**: PASS / **Observed**: FAIL

### Canonical Intention & Settlement
- **Intention**: Understand a committed case's purpose, current horizon, history, and lawful adaptation paths without rewriting prior truth.
- **Settlement Condition**: The user can identify the case's governing outcome, current spendable and blocked work, the evidence for every state, and each presently lawful action.
- **Oracle Status**: ACCEPTED
- **Oracle Rationale**: Case horizon projected 1 item(s), each with distinct execution/settlement standing folded from trace events.

### Consequential Visual Evidence
- **CJ05-001-entry-case-horizon.png** (entry): Live case horizon, continued from CJ04's committed case
  ![CJ05-001-entry-case-horizon.png](../../screenshots/CJ05/CJ05-001-entry-case-horizon.png)
- **CJ05-002-affordance-item-standing.png** (affordance_discovery): Active, waiting, and completed plan items
  ![CJ05-002-affordance-item-standing.png](../../screenshots/CJ05/CJ05-002-affordance-item-standing.png)
- **CJ05-003-next-affordance-execution.png** (resulting_next_affordance): Enabled item offers lawful next action
  ![CJ05-003-next-affordance-execution.png](../../screenshots/CJ05/CJ05-003-next-affordance-execution.png)
