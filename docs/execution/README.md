# Execution Campaign Documents (docs/execution)

## Overview
Working documents of the SEA Forge execution campaign: the mission definition
(`ULTRACODE_MISSION.md`), the execution plan and DAG (`MASTER_EXECUTION_PLAN.md`,
`EXECUTION_DAG.md`/`EXECUTION_DAG.json`), pass handoffs (`PASS_1_HANDOFF.md`,
`PASS_2_HANDOFF.md`), architectural truth and invariants
(`ARCHITECTURAL_TRUTH.md`, `ARCHITECTURAL_INVARIANTS.md`,
`REPOSITORY_TRUTH.md`, `FINAL_ARCHITECTURE_STATE.md`), decision registers
(`DECISION_REGISTER.md`, `CONTRADICTIONS_AND_DECISIONS.md`), verification
artifacts (`VERIFICATION_BASELINE.md`, `VERIFICATION_MATRIX.md`,
`FINAL_VERIFICATION_RESULTS.md`, `USER_JOURNEY_EVIDENCE.md`), completeness
tracking (`PRODUCT_COMPLETION_DEFINITION.md`, `INCOMPLETENESS_INVENTORY.md`,
`REMAINING_BLOCKERS.md`, `FINAL_COMPLETION_REPORT.md`), operations
(`OPERATIONS_AND_STARTUP.md`), traceability (`SPEC_CODE_TRACEABILITY.md`,
`SUBSTRATE_MANIFEST.json`), and workbench drafts
(`ULTRACODE_CONTEXT_INDEX.md`, `ULTRACODE_READINESS_REPORT.md`,
`WORKBENCH_DRAFTS.md`). These are Markdown/JSON campaign artifacts, not code;
the executable part of this subtree is the journey driver scripts.

## Subdirectories & Boundaries
- journey: [./journey/README.md] - Live SFWP-socket journey drivers
  (`drive.py`, `drive_identity.py`) that exercise a real booted cell and
  produce PASS/FAIL journey evidence for identity, separation of duty, and
  default-deny authorization.
- work-packets: (no contract written) - Work packet specifications
  `SF-001.md` through `SF-013.md`; Markdown-only campaign task definitions
  with no executable code.
