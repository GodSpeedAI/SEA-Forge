# Current status

**Status revision:** 204

**Stage:** Security: CLI approve resolves settlement pending approvals (F-set complete)

**Summary:** The approve command resolves an apr-settlement-pending approval through CepAuthorityService::resolve_approval with an operator resolver. The requester cannot resolve it, and the approval must name a pending record of the case given on the command line. The m8 CLI settlement flow now approves and resumes through the CLI.

**Verified:**
- Red test first: an approval naming a settlement pending record of another case is refused by the CLI approve path (cli approve: settlement branch covered by the m8 CLI test)
- cargo test -p sea-forge-cli --test conformance_m8_artifact: 6 passed (settlement approve by requester refused, approve by settlement_approver succeeds, resume completes)

**Limits:**
- The CLI approve test is the m8 settlement flow, which now approves through the CLI instead of the library; no separate CLI-only test was added.
- Live round-trip against a running gateway not performed; needs an owner runbook step.
- Not pushed; no PR opened. PR body must call out the new sea-forge-capability to sea-forge-authority dependency edge.

**Next:** Next: owner review of the PR scope, live round-trip against a running gateway, and PR creation (not done here).
