# Current status

**Status revision:** 231

**Stage:** domainforge-0.21.2-lockstep

**Summary:** SEA-Forge moves its DomainForge producer pin from 0.21.1 to 0.21.2 for stack lockstep (readiness gate C01: versions disagree 0.21.1 vs 0.21.2; SW01/SW02: CLI reports 0.21.2). The library pin, EXPECTED_DOMAINFORGE_VERSION, devbox CLI install, Cargo.lock (targeted update), fixture provenance and READMEs now say 0.21.2. The v2-full-preimage world_ref changes with producer_version: demo world a71ff6... becomes 7ddedb..., confirmed by both the linked compiler and the 0.21.2 CLI.

**Verified:**
- cargo update -p domainforge-core --precise 0.21.2: only domainforge-core changed in Cargo.lock
- cargo check -p sea-forge-domainforge: ok
- cargo test -p sea-forge-domainforge: 4 suites with tests, 0 failed (prior_identity digest updated to the compiler-minted 7ddedb...)
- cargo test -p sea-forge-authority --test cep_compiler_upgrade: 9 passed, 0 failed
- cargo test -p sea-forge-server --test conformance_cep_authority: 13 passed, 0 failed
- fixtures regenerated with the 0.21.2 CLI: byte-identical apart from the provenance generator field; OpenShell lock unchanged
- cargo fmt --check on the three touched crates: clean

**Limits:**
- cargo test --workspace and just ci not run (host memory); Cognate/Horizon live suites not re-run
- 0.21.0 prior-identity fixtures kept deliberately: they model the older producer
- docs/how-to/upgrade-domainforge-producer.md still records a 0.21.1 check and its a71ff6 digest as history

**Next:** Merge with the other stack products pinned to 0.21.2; re-run the readiness gates C01, SW01 and SW02.
