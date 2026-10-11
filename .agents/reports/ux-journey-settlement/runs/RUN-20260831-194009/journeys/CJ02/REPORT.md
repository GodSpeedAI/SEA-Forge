# Conformance Report: CJ02 — Discover Lawful Affordances

## Status: FAIL

### Gates Evaluation
| Gate | Status |
| --- | --- |
| ENTRY | FAIL |
| VISIBILITY | FAIL |
| REACHABILITY | FAIL |
| BINDING | FAIL |
| AUTHORITY | FAIL |
| EXECUTION | FAIL |
| EVIDENCE | FAIL |
| SETTLEMENT | FAIL |
| CONTINUITY | FAIL |
| RECOVERY | FAIL |

### Failure Diagnosis
- **Code**: `ENTRY_GAP`
- **Locus**: ENTRY
- **Expected**: PASS / **Observed**: FAIL

### Canonical Intention & Settlement
- **Intention**: Understand what exists, what is usable, what is allowed, and why a path is unavailable in the current actor and cell context.
- **Settlement Condition**: The user receives a source-backed set of currently visible, reachable, permitted, and settleable actions, including explicit blockers, uncertainty, freshness, and limitations.
- **Oracle Status**: ERROR
- **Oracle Rationale**: AttributeError: 'str' object has no attribute 'get'
Traceback (most recent call last):
  File "/home/sprime01/projects/sea-rs/.agents/reports/ux-journey-settlement/harness/gauntlet_runner.py", line 289, in _run_journey_safely
    fn()
    ~~^^
  File "/home/sprime01/projects/sea-rs/.agents/reports/ux-journey-settlement/harness/gauntlet_runner.py", line 542, in execute_cj02
    query_verbs = [q.get("verb") for q in state.get("queryLog", [])]
                                          ^^^^^^^^^
AttributeError: 'str' object has no attribute 'get'


### Consequential Visual Evidence
