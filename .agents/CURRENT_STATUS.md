# Current status

**Status revision:** 207

**Stage:** Security: fixtures rebuilt on production behaviour (local strength; authorised external capability fixture)

**Summary:** Test fixtures now match production. The local-behaviour reconciliation tests build local-strength, non-qualifying declarations, and still assert local declarations are not quarantined. The Thoth capability fixture is an authorised external declaration: it is gated, approved through CepAuthorityService::resolve_approval, claimed, and then appended bound to that pending record. is_external, the unclaimed refusal and the quarantine reasons are unchanged.

**Verified:**
- cargo test -p sea-forge-server --lib swe_seed_reconciliation: 14 passed (was 2 failing)
- cargo test -p sea-forge-thoth --test conformance_m11_service: 9 passed (t14a demonstrated now passes)

**Limits:**
- The approved request hash still binds only the gated ingress, not the declared content (Task B, not yet done).
- Live round-trip against a running gateway not performed.

**Next:** Bind the approved request hash to the declaration content that is appended (delegation.rs, resume.rs).
