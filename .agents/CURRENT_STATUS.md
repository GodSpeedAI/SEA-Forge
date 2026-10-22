# Current status

**Status revision:** 153

**Stage:** compiler-upgrade-transition

**Summary:** A DomainForge producer upgrade becomes a routine, auditable move. Hosts list each world's superseded identities as cep_authority.worlds[].prior_identities (strict, bounded, digest-checked, registered as prior). A prior is a transition source only, never a place to act: governed actions in it are refused with cep_world_refused, and it is refused as a target. compiler_upgrade is strengthened in place (no new wire kind): SEA-Forge allows it only when semantic_closure_hash, source_set_hash, semantic_pack_set_hash, registry_content_hash, identity_scheme_version, language_schema_version, compiler_interpretation_version, canonicalization_version and producer are all equal and producer_version differs, naming the first field that differs. source_edit_only is refused across a producer change. The allowed record stays cep world-transition.v1 valid and carries both producer versions in evidence_refs. Operator procedure: docs/how-to/upgrade-domainforge-producer.md.

**Verified:**

- cargo test -p sea-forge-domainforge: 23 + 4 + 5 + 11 passed, 0 failed (4 new prior_identity tests; fixture refs c2d3978a... prior and a71ff6... current asserted)
- cargo test -p sea-forge-authority: all binaries pass, 0 failed; cep_compiler_upgrade 9 passed (allowed 0.21.0->0.21.1 recorded; refused per each meaning-bearing field, producer_version equal, source_edit_only across the upgrade, semantic change, prior as target, governed action in prior, lineage only through the licence); cep_transition 17 passed unchanged
- cargo test -p sea-forge-server --no-fail-fast (after building sea-forge-cli, which conformance_m16 requires): 519 passed, 0 failed, 2 ignored; conformance_cep_authority 13 passed over the socket (3 new), config prior_identities validation 6 passed
- cargo clippy -p sea-forge-authority -p sea-forge-server --all-targets -- -D warnings: clean
- domainforge-core 0.21.1 CLI (installed to a scratch root, not the PATH binary) emits extensions["domainforge.identity"].domain_model_identity under --emit cep --world-name; its scope.world_ref is world:demo@sha256:a71ff6...; against the 0.21.0 fixture only producer_version and content_hash differ
- a compiler_upgrade record with the new evidence_refs validates against cep world-transition.v1 (validate_world_transition returned no errors)
- governed-work ingress (verify_world) now refuses a prior identity, as the CEP path does; convergence_t11 covers it and fails without the guard (review finding on PR #42)

**Limits:**

- a malformed or unreadable prior identity is a per-request refusal (cep_world_config), following how bad world sources are reported; startup does not read world files
- identity_scheme_version mutations never register (DomainForge supports one scheme); the check is defensive and covered at registration
- content_hash is not compared by compiler_upgrade: it is a digest of the document, which carries producer_version
- source_edit_only is refused on any producer or producer_version change, so an edit and an upgrade cannot be declared as one move
- PR #42: GitHub CI green (gate, lint, test, MSRV, workbench, macOS, authority-opa, live ladder); L01 35/35 and C01 PASS in the readiness workspace

**Next move:** Merge PR #42 once review threads are resolved; export the 0.21.0 identities per docs/how-to/upgrade-domainforge-producer.md before the next DomainForge upgrade.

**Evidence:**

- crates/sea-forge-authority/tests/fixtures/identities/demo-tank.domainforge-0.21.0.json
- docs/how-to/upgrade-domainforge-producer.md

**Spec:** .agents/specs/cep-world-ref-migration.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
