# C2 projected-facts writer enumeration — revision 2 correction

Date: 2026-10-08  
Base inventory: `c2-projected-facts-in-tree-writer-enumeration-recon-oct08.md`  
Base SHA-256: `6f2ab684d0b46ed39267c3dc62c6d725001839a67098be2ce6570860558d51f0`  
Review finding: `c2-projected-facts-in-tree-writer-enumeration-independent-review-oct08.md`

This is an additive correction overlay. The base inventory remains immutable;
all its sections and classifications remain in force except for the
`case.json` producer mapping below. Read this overlay together with the base
inventory as revision 2. The correction adds one identified path and narrows
the guard description for that path; it does not claim exhaustive writer
coverage or migration readiness.

## Replacement: `case.json` producer mapping

Add the following writer to the base inventory’s first producer row, which
lists the case directory, `case.json`, `plan.json`, `case-events.jsonl`, and
`run_ids` producers:

* CLI manager escalation: `crates/sea-forge-cli/src/commands/manager.rs:247–290`
  constructs and commits an `approval_request` (lines 261–286), appends the
  approval projection (line 287), sets `case.state` to
  `CaseState::AwaitingApproval` (line 288), then calls `save_case` (line 289).
  This changes the `case.json` state read by the case list and overview
  projections, in addition to changing `approvals.jsonl` already covered by
  the base inventory’s approval producer row.

The call chain is `commands::manager::escalate` →
`crates/sea-forge-cli/src/commands/case.rs:53–55` →
`crates/sea-forge-case-runner/src/case_ops/mod.rs:443–446`, where `save_case`
resolves the canonical case path and calls `write_json`. The manager command’s
caller authorizes `manager_iteration` through
`commands/manager.rs:78–88`; the escalation helper is reached from that
iteration flow. The shared `case_ops::save_case` helper itself performs no
additional policy/authority check, and this path does not participate in a
shared C2 projection-journal transaction. The approval ledger and
`approvals.jsonl` writes precede the `case.json` save, so the artifacts are
not atomic together; a failure at the final save can leave the approval
request visible without the new case state.

## Guard and classification correction

For this manager path, classify the guard as the caller’s manager-iteration
authorization. The approval record uses the ledger’s stream file lock, but the
subsequent `case.json` save has no shared projection lock. Do not
classify the `save_case` helper as independently enforcing case mutation
policy, or as using a C2 lock/journal. Other `case_ops` mutation functions
retain their existing path-specific checks as described by the base inventory;
that statement does not extend to this direct save helper.

No other base inventory row, search-coverage statement, federation/external
writer classification, lock description, limitation, or migration boundary is
changed by this correction. In particular, this one added producer does not
close the original inventory’s stated uncertainty about dynamic paths,
downstream consumers, indirect writers, or runtime participation. No source,
test, configuration, or Git state was changed; this revision changes only the
documented inventory mapping.
