# Current status

**Status revision:** 200

**Stage:** Security: adversarial review F3 fixed (claim refuses content that differs from the approved request)

**Summary:** A settlement claim now compares the content hash the caller is about to declare with the approved request hash. A mismatch spends the approval as failed(content_mismatch) and no declaration is made. Callers compute the hash with settlement_request_sha256 over the same ingress they declare under. F5, F6 and F1 remain open on this branch.

**Verified:**
- F3 red-to-green: a claim whose declared content differs from the approved request fails as content_mismatch with no declaration; matching content resumes (cep_settlement: 47 passed)
- cargo test -p sea-forge-server --lib swe_seed: 13 passed
- cargo check --all-targets for sea-forge-cli, sea-forge-artifact-ip, sea-forge-thoth, sea-forge-case-runner: clean

**Limits:**
- F5, F6, F1 and CLI approve not yet addressed on this branch.
- Declared-content check compares the hash the caller presents with the approved request hash; callers derive it from the same ingress, so the binding is to the gated ingress content, not every declaration field.

**Next:** Next: F5/F6 case scoping in the reconciliation verifier, F1 capability dependency and promotion verification, then CLI approve for settlement approvals.
