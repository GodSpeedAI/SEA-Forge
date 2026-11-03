# Current status

**Status revision:** 201

**Stage:** Security: adversarial review F5 and F6 fixed (case-scoped verification and reconciliation)

**Summary:** The declaration verifier now refuses a pending record that belongs to another case, and reconciliation settles only the claims of its own case and operation. A declaration in one case cannot complete a claim in another. F1 (capability promotion) and CLI approve remain open on this branch.

**Verified:**
- F5/F6 red-to-green: a declaration in another case cannot complete a claim (server swe_seed_reconciliation: 14 passed)
- Declaration verification refuses a pending record whose case differs from the declaration's case (authorization_mismatch)
- Reconciliation settles only pending records of the same case and operation

**Limits:**
- F1 not yet addressed on this branch: capability promotion does not yet verify external declarations.
- CLI approve cannot resolve settlement approvals yet (case-level only).

**Next:** Next: F1 capability dependency on sea-forge-authority and promotion through verify_case_declarations, then CLI approve for settlement approvals.
