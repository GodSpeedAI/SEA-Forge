# Current status

**Status revision:** 176

**Stage:** b08-domainforge-engine-fail-closed

**Summary:** B-08/SUP-09h: a policy bundle that enables engine: domainforge is refused at load with authority_engine_unavailable, required or optional, before any other bundle check. The filename-stem DomainForge candidate evaluator, its composition into authority decisions, and the bundle-level model load were removed from sea-forge-authority, sea-forge-domainforge, sea-forge-case-runner, sea-forge-cli and sea-forge-server.

**Verified:**

- cargo test -p sea-forge-authority --no-fail-fast: 16 test binaries ok, including domainforge_engine_is_refused_before_composition, policy_yaml_strictness domainforge_engine_is_refused_required_or_optional, and yaml_corpus expecting the refusal for doc_011
- cargo test -p sea-forge-domainforge --no-fail-fast: ok
- cargo test -p sea-forge-case-runner --no-fail-fast: ok
- cargo test -p sea-forge-cli --test lifecycle: 12 passed (run_ingress_refuses_domainforge_engine_at_load)
- cargo test -p sea-forge-server --lib: ok
- cargo check --all-targets for authority, domainforge, case-runner, cli, server, runtime, realitytrace, extension, artifact-ip: ok
- cargo clippy -D warnings --all-targets for domainforge, authority, case-runner, cli, server: clean; cargo fmt applied and checked

**Limits:**

- AuthorityEvaluation no longer has a domainforge_candidate field; the authority context json keeps domain_model_ref as null
- sea_forge_domainforge keeps load_validate, DomainModel and DomainModelRef (used by self-model, cli project and artifact-ip); CandidateDisposition, DomainForgeTrace, normalize_authority and evaluate_authority were removed
- fixtures/yaml-corpus/reference.json still records the old doc_011 outcome (schema_error); the corpus test carries the new expectation in REFUSED_DOMAINFORGE_ENGINE
- docs/subsystems/domainforge-boundary.md section on DomainForgeCandidate wording not fully rewritten

**Next move:** Merge after CI; the domain_model source surface remains required for v0.2 bundles and can be revisited separately.

**Evidence:**

- `crates/sea-forge-authority/src/lib.rs`
- `crates/sea-forge-authority/tests/policy_yaml_strictness.rs`
- `crates/sea-forge-authority/tests/yaml_corpus.rs`
- `crates/sea-forge-authority/tests/fixtures/yaml-corpus/real/doc_011_8f23190964a9.yaml`
- `docs/subsystems/domainforge-boundary.md`
- `docs/workflows/cli-run-lifecycle.md`
- `.agents/specs/spec-full.md`
