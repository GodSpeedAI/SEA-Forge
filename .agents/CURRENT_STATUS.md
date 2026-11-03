# Current status

**Status revision:** 199

**Stage:** Security: adversarial review F2 fixed (case_id bound into the settlement request hash)

**Summary:** Settlement requests now hash case_id with the operation, so the same operation in two cases gets two pending records and two approvals. Pending records written before the binding carry no case_id and are quarantined: no lookup matches them and a declaration naming one is refused as authorization_unscoped. F3, F5, F6 and F1 remain open on this branch.

**Verified:**
- cargo test -p sea-forge-authority --test cep_settlement: 45 passed (includes case-binding and legacy non-reuse tests)
- cargo check --all-targets for sea-forge-server, sea-forge-cli, sea-forge-artifact-ip, sea-forge-thoth, sea-forge-case-runner: clean
- cargo fmt --all -- --check: clean

**Limits:**
- Adversarial finding F2 fixed: case_id is in the canonical request hash and the pending record. F3, F5, F6 and F1 not yet addressed on this branch.
- Legacy pending records (no case_id) are quarantined: never matched by any lookup, refused as authorization_unscoped when a declaration names them. Their open claims, if any, are left for operator review.

**Next:** Next: F3 claim-time content hash check, then F5/F6 case scoping, F1 capability dependency and promotion verification, then CLI approve for settlement approvals.
