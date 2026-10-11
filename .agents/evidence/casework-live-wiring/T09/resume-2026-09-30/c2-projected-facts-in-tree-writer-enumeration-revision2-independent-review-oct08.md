# Independent review: C2 writer enumeration revision 2

Date: 2026-10-08  
Reviewed overlay: `c2-projected-facts-in-tree-writer-enumeration-revision2-oct08.md`  
SHA-256: `efd395cd4c5479d901b67bf875d9a3076984a338f276a576e285d5e26846259e`  
Base inventory SHA-256: `6f2ab684d0b46ed39267c3dc62c6d725001839a67098be2ce6570860558d51f0`

## Decision

**Approve this documentation correction.** It fixes the omission identified in the prior review: `commands::manager::escalate` mutates the approval record and the projected `case.json` state. The overlay correctly adds the latter to the case-artifact producer mapping while retaining the original inventory's explicit limits on completeness, dynamic paths, external writers, and runtime participation.

## Source verification

* `crates/sea-forge-cli/src/commands/manager.rs:78-88` performs `authorize_read_with_grant` for the reserved `manager_iteration` action before the iteration loads case data. `manager.rs:103-112` reaches `escalate` on the iteration cap; `graft callers escalate` confirms `iterate` is its caller. This supports the overlay's narrower characterization: the helper itself does not perform a separate case-mutation authorization.
* `manager.rs:281-289` calls `LedgerStream::commit_typed("approval_request", ...)`, appends the approval view, changes `case.state` to `AwaitingApproval`, then calls `save_case`.
* `crates/sea-forge-cli/src/commands/case.rs:53-55` delegates to `sea_forge_case_runner::case_ops::save_case`; `crates/sea-forge-case-runner/src/case_ops/mod.rs:443-446` resolves the canonical case path and calls `write_json`. The helper at `mod.rs:63-68` writes a temporary JSON file and renames it into place without an authorization check or shared projection lock.
* `crates/sea-forge-ledger/src/types.rs:992-1003` wraps ledger append in `with_exclusive_lock`; `types.rs:694-723` shows that lock is per ledger stream. `crates/sea-forge-core/src/approvals.rs:31-49` separately appends `approvals.jsonl` through an `O_APPEND` file handle. Neither operation creates a transaction spanning the approval projection and `case.json`. The overlay accurately identifies the ordering and non-atomicity.

The corrected artifact mapping, caller authorization description, and lock distinction are source-supported. No material discrepancy remains in this overlay. This approval is limited to the documentation correction; it does not approve the complete-writer audit, journal design, migration readiness, or runtime participation.

No source, test, configuration, or Git state was changed. Graft retrieval this turn saved approximately 99,545 tokens; reported value approximately $0.05 plus four calls individually reported under $0.01 each (total under $0.09).
