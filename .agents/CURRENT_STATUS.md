# Current status

**Status revision:** 177

**Stage:** b08-domainforge-engine-fail-closed

**Summary:** B-08/SUP-09h: a policy bundle that enables engine: domainforge is refused at load with authority_engine_unavailable, required or optional, before any other bundle check. The filename-stem DomainForge candidate evaluator, its composition into authority decisions, and the bundle-level model load were removed from sea-forge-authority, sea-forge-domainforge, sea-forge-case-runner, sea-forge-cli and sea-forge-server. Revision 176 (the B-12 replay) replaced the limits list with the B-12 branch's stale copy and dropped 14 open limits; this revision restores them, keeps B-12's four, adds B-08's, and drops only the filename-stem DomainForge limit that B-08 resolves.

**Verified:**

- cargo test -p sea-forge-authority --no-fail-fast: 16 test binaries ok, including domainforge_engine_is_refused_before_composition, policy_yaml_strictness domainforge_engine_is_refused_required_or_optional, and yaml_corpus expecting the refusal for doc_011
- cargo test -p sea-forge-domainforge --no-fail-fast: ok
- cargo test -p sea-forge-case-runner --no-fail-fast: ok
- cargo test -p sea-forge-cli --test lifecycle: 12 passed (run_ingress_refuses_domainforge_engine_at_load)
- cargo test -p sea-forge-server --lib: ok
- cargo check --all-targets for authority, domainforge, case-runner, cli, server, runtime, realitytrace, extension, artifact-ip: ok
- cargo clippy -D warnings --all-targets for domainforge, authority, case-runner, cli, server: clean; cargo fmt applied and checked

**Limits:**

- The live release PR content is produced by the Release run on main after this merges; it has been checked here by the updater run and the locked resolution, not yet by CI on the regenerated PR
- Until RELEASE_PLEASE_TOKEN exists, a release PR opened by GITHUB_TOKEN gets its pull_request runs in an approval-required state; a maintainer approves them (carried from 167)
- Creating RELEASE_PLEASE_TOKEN is a maintainer credential action, not done here (carried from 167)
- Only the obligation modality settles; prohibition and permission are refused as unsupported (carried from 164)
- A crate added later with versioned path dependencies must be added to the extra-files list, or the next release PR fails the same way
- The release-binaries job has not run on GitHub; it is unverified until a release run or a workflow_dispatch with tag v0.2.0 completes
- Binaries are built for x86_64-unknown-linux-gnu only; no other platform is shipped
- SHA256SUMS is produced in the same job as the tarball and proves integrity against corruption, not against a compromised release; no signing is in place
- Only sea-forge and sea-forge-server ship; sandbox helper binaries sea-forge-workload-exec and sea-forge-mcp-connect are internal
- obligation.rs record_closure 'window is still open; a closure is recorded once it closes' still returns cep_evidence_refused by decision (it concerns recording a closure, not settlement)
- ledger repair is offline by contract but does not check that the server is stopped; the operator must stop the server first
- startup now refuses a ledger whose chain is broken as well as mid-ledger corruption (fail closed); a crash-torn final record is still repaired silently at startup
- a complete final record missing only its newline is quarantined as one torn record
- The seeded cell's server.yaml makes the script's uid the gateway principal; operator_a and operator_b act only through on_behalf_of (the supported delegation path), not through their own uid
- The escalation policy's argv0 link lives in <target>/demo-tools/sea-forge, outside the cell; the recorded plan argv therefore names that path, which survives only while the target dir does
- The seed script's closing hint 'sea-forge case list' is stale (no such subcommand); not changed here
- The backup check runs under the cell lock after the live-server check; it does not scan the cell for other special file types (fifos, devices) beyond what restore already refuses
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
