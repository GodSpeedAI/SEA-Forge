# Current status

**Status revision:** 235

**Stage:** domainforge-0.22.0-lockstep

**Summary:** SEA-Forge moves its DomainForge producer pin from 0.21.2 to 0.22.0 for stack lockstep. Library pin, EXPECTED_DOMAINFORGE_VERSION, devbox CLI install, Cargo.lock (targeted update), fixture provenance and READMEs now say 0.22.0. The demo world_ref changes with producer_version: 7ddedb... becomes 469e93..., minted by the linked compiler and confirmed with the 0.22.0 CLI. The Rego lib.rego fixtures changed (exists_unique is three-valued) so provenance digest 3b25cf21... becomes bff56dc0....

**Verified:**
- scripts/check-agent-context.sh: ok
- cargo update -p domainforge-core --precise 0.22.0: only domainforge-core changed in Cargo.lock
- cargo test -p sea-forge-domainforge: all suites ok (22, 4, 5, 11 passed; 0 failed)
- cargo test -p sea-forge-authority --test cep_compiler_upgrade: 9 passed; opa_candidate 22 passed + 7 ignored passed with opa
- cargo test -p sea-forge-server --test conformance_cep_authority: 13 passed
- cargo test -p sea-forge-case-runner --test cell_episode: 24 passed + 6 ignored passed with opa
- fixtures regenerated with the 0.22.0 CLI; cargo fmt --check on touched crates: clean

**Limits:**
- cargo test --workspace and just ci not run (host memory)
- 0.21.0 prior-identity fixtures kept deliberately: they model the older producer
- docs/how-to/upgrade-domainforge-producer.md still records a 0.21.1 check and its a71ff6 digest as history

**Next:** Merge after CI; keep Cognate, Horizon, Workbench and RealityTrace on the same DomainForge version.
