# Current status

**Status revision:** 198

**Stage:** Security: post-review fixes for quarantine class mismatch, thoth view filter, and clippy

**Summary:** Post-review fix pass. An External-classed declaration relabelled as Local is now quarantined (ClassMismatch) rather than correlated. The thoth capability view filters settlement/declarations.jsonl through verify_case_declarations. Clippy -D warnings is clean across the workspace (artifact-ip error borrows removed, resume_pending_transition marked too_many_arguments). Full server and CLI suites: 646 pass, 0 fail.

**Verified:**
- cargo clippy --all-targets -- -D warnings: exit 0 (workspace)
- cargo fmt --all -- --check: clean
- cargo test -p sea-forge-authority: all suites pass
- cargo test -p sea-forge-server -p sea-forge-cli --no-fail-fast: 646 passed, 0 failed
- earlier: authority+thoth+artifact-ip+settlement+capability 427 pass

**Limits:**
- Adversarial review findings F2, F3, F5, F6 not fixed: need case_id binding in the canonical hash, content-hash binding at claim time, authority-ledger-wide scope.
- Capability-side F1 only partly fixed (thoth view filtered; capability build still needs sea-forge-capability to depend on sea-forge-authority, which AGENTS.md says to ask about first).
- CLI approve cannot resolve settlement approvals (case-level only); m8 tests resolve through CepAuthorityService::resolve_approval.
- Live round-trip against a running gateway not performed; needs an owner runbook step.
- Not pushed; no PR opened.

**Next:** Owner decision on F2/F3/F5/F6 scope and the capability dependency; then CLI approve support for settlement approvals, live round-trip, and PR creation.
