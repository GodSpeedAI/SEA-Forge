# Current status

**Status revision:** 202

**Stage:** Security: adversarial review F1 fixed (capability promotion verifies external declarations)

**Summary:** Capability promotion now routes every settlement declaration through the shared verifier, verify_case_declarations. An external declaration with no approved, claimed authorisation is quarantined and cannot promote a capability. This adds the owner-approved dependency edge sea-forge-capability to sea-forge-authority. CLI approve for settlement approvals remains open.

**Verified:**
- F1 red-to-green: unauthorised external declaration refused at promotion (conformance_m4a: 22 passed, includes f1_unauthorised_external_declarations_are_refused_at_promotion)
- sea-forge-capability depends on sea-forge-authority; rebuild_capability routes every declaration through verify_case_declarations (owner-approved edge)
- just no-async-kernel: ok

**Limits:**
- CLI approve cannot resolve settlement approvals yet (case-level only); m8 CLI tests resolve through CepAuthorityService::resolve_approval.
- Live round-trip against a running gateway not performed; needs an owner runbook step.
- Not pushed; no PR opened. PR body must call out the new sea-forge-capability to sea-forge-authority dependency edge.

**Next:** Next: CLI approve resolves settlement pending approvals through CepAuthorityService::resolve_approval, with a CLI approve-then-resume test.
