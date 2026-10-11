# C2 projected-facts writer enumeration: independent source review

Date: 2026-10-08  
Reviewed artifact: `c2-projected-facts-in-tree-writer-enumeration-recon-oct08.md`  
Reviewed SHA-256: `6f2ab684d0b46ed39267c3dc62c6d725001839a67098be2ce6570860558d51f0` (10,831 bytes)

## Result

The recon is appropriately bounded: it names the five readers and their on-disk inputs, distinguishes known in-tree writers from external filesystem writers, records the current lock boundaries, and explicitly does not claim exhaustive participation. I found one concrete artifact-to-writer mapping omission. It does not invalidate the recon's stated caveat, but the omission should be carried into any corrected inventory.

`commands::manager::escalate` is listed in the `approvals.jsonl` row (recon §Known in-tree producers, row 20), but its write to projected case state is missing from the `case.json` producer row (row 16). In `crates/sea-forge-cli/src/commands/manager.rs:247-290`, escalation commits an approval request, appends it to the approval view, sets `case.state = CaseState::AwaitingApproval`, and calls `save_case`. `crates/sea-forge-cli/src/commands/case.rs:53-55` forwards that call to `case_ops::save_case`; `crates/sea-forge-case-runner/src/case_ops/mod.rs:443-446` resolves the canonical case path and writes the case record. This affects the projected case state read by CaseList/CaseOverview and should be named in the case-artifact producer mapping, not only in the approval-file mapping.

## Cross-checks

Graft queries covered `save_case` callers; `append_event` in CaseRunner and server; `write_json`, `case.json`, `fs::write`, `write_all`, `commit_typed`, `run_ids.push`, `create_dir_all`, and server `OpenOptions::new`. The indexed results agree with the recon's principal writer families: CaseRunner/case_ops, CLI plan and resume flows, server submit/advance flows, run trace/evidence/settlement recorders, approvals appenders, and migration. The exact `run_ids.push` results were limited to plan pipeline, CLI resume, CaseRunner episode completion, and the horizon reader's event-derived IDs. Dynamic artifact-transition plan output is written under `artifact-transition-plans/` and then passed into the ordinary plan pipeline; it is not itself one of the five readers' files.

The inspected source supports the recon's lock distinction: `case_ops` and server case mutation paths have their stated local checks/serialization, while no single shared CLI/server/migration projection transaction is shown. The searches do not prove the absence of arbitrary dynamic writes, downstream CaseRunner consumers, or out-of-tree writers. No runtime participation or complete-writer-coverage claim follows from this review.

## Scope and provenance

Read-only source review only. No source, tests, configuration, or Git state changed. The source anchors above were checked against the current working tree after Graft retrieval. The review is not an approval of the C2 journal design or a claim that the inventory is complete.

Graft retrieval this turn: approximately 759,780 tokens saved; reported value approximately $0.59 plus four calls individually reported as less than $0.01 each (total less than $0.63).
