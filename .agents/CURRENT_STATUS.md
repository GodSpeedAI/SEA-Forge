# Current status

**Status revision:** 205

**Stage:** Security: adversarial review F-set on settlement-approval-gate (F1, F2, F3, F5, F6, CLI approve)

**Summary:** Settlement pending records and approvals are bound to case and operation, the claim checks declared content against the approved hash, reconciliation and the verifier are case-scoped, capability promotion admits only authorised external declarations, and the CLI approve resolves settlement pending approvals. Pushing and PR creation were not done.

**Verified:**
- Corrects revision 205: no separate red run was recorded for the CLI approve settlement branch; the m8 CLI settlement flow was the test
- cargo test -p sea-forge-authority, -p sea-forge-capability, -p sea-forge-server -p sea-forge-cli: all pass except the pre-existing sea-forge-server doctest (sea_forge_agent not linked; agent_probe.rs untouched on this branch)
- cargo clippy --all-targets -- -D warnings: exit 0; cargo fmt --all -- --check: clean

**Limits:**
- No 300x race stress test was found in this worktree; the closest concurrency tests (concurrent_resumes_of_one_approval_declare_exactly_once, approval_consumption rounds) pass.
- Live round-trip against a running gateway not performed; needs an owner runbook step.
- Not pushed; no PR opened. PR body must call out the new sea-forge-capability to sea-forge-authority dependency edge.

**Next:** Next: owner review, live round-trip against a running gateway, and PR creation with the sea-forge-capability dependency edge called out.
