# Current status

**Status revision:** 149

**Stage:** domainforge-0.21.1-lockstep

**Summary:** DomainForge moves to 0.21.1 coherently (library pin, EXPECTED_DOMAINFORGE_VERSION, devbox CLI, fixture provenance and READMEs), reversing the deliberate 0.21.0 hold kept by PR #39. Reason: DomainForge's v2-full-preimage world identity includes producer_version, so SEA-Forge on 0.21.0 derived world:demo@sha256:c2d397... for the same source the 0.21.1 CLI, Cognate and Horizon name world:demo@sha256:a71ff6...; Cognate's live CEP suite was refused as unknown world in 15 of 35 tests. On 0.21.1 both sides derive a71ff6....

**Verified:**

- probe test (removed before commit) printed SEA-Forge world_ref c2d397... on 0.21.0 and a71ff6... on 0.21.1; the released 0.21.1 CLI prints a71ff6... for the same source
- identity diff 0.21.0 vs 0.21.1: semantic_closure_hash, source_set_hash and semantic_pack_set_hash equal; producer_version and content_hash differ
- fixtures regenerated with the released 0.21.1 CLI (cargo install domainforge-core 0.21.1 --features cli --locked): byte-identical apart from provenance generator; OpenShell lock unchanged
- cargo test -p sea-forge-domainforge -p sea-forge-authority -p sea-forge-case-runner -p sea-forge-changeset and sea-forge-server --test conformance_cep_authority: 21 test binaries, 0 failed
- Cargo.lock changes beyond domainforge-core (toml 1.x, serde_spanned, toml_datetime, toml_writer, uuid 1.23.4, winnow removed) come with domainforge-core 0.21.1; no SEA-Forge crate depends on toml directly

**Limits:**

- Cognate live suite (just sea-forge-live) not yet re-run against this branch's server binary
- full just ci not run on this branch
- the stack-level question of whether world_ref should change on every DomainForge release is open with DomainForge (godspeed-readiness P01); until then all products must pin the same release

**Next move:** Re-run Cognate's live CEP suite against this branch (godspeed-readiness L01), then just ci; open a PR only on operator go.

**Evidence:**

- ~/projects/godspeed-readiness/evidence/runs (L01, C01)
- ~/projects/godspeed-readiness/docs/proposals/P01-world-identity-and-producer-version.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
