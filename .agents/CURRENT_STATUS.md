# Current status

**Status revision:** 214

**Stage:** T1 stage D2.2: case-task PendingApproval renamed to CaseTaskApproval; approval-standing golden fixture

**Summary:** The sfwp case-task PendingApproval is now CaseTaskApproval (Rust type, SCHEMA_TYPES entry, schema title and file, ApprovalListResult defs by regeneration, TS contracts, ApprovalInboxPage). ApprovalStanding is unchanged. A golden fixture contracts/fixtures/approval-standing.json holds two ApprovalStanding::to_json objects, checked by crates/sea-forge-authority/tests/approval_standing_fixture.rs as parsed JSON with exactly the 11 wire keys.

**Verified:**
- cargo test -p sea-forge-server --test conformance_sfwp: 20 passed
- cargo test -p sea-forge-authority: 297 passed, 0 failed (295 before, plus 2 fixture tests)
- fixture mutation (resource_id changed) fails the fixture test
- just workbench-contracts-gate and workbench bun run check: clean

**Limits:**
- Casework-go Go identifiers named PendingApproval are not renamed (Non-Goal; no Go toolchain here)
- The fixture is the to_string_pretty output, so its keys are alphabetical; it is value-equal to the Cognate copy but not byte-identical

**Next:** Review and merge D2.1 then D2.2; sync the Cognate fixture copy to the sea-rs fixture bytes.
