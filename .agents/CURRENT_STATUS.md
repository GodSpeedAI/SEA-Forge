# Current status

**Status revision:** 232

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
- A detached grandchild that survives the process-group kill could append after the operator attests NotDeclared; lease safety assumes the authority call is bounded
- --declared <ref> is not checked against a ledger declaration
- A failure inside declare before the transport spawns is held for an operator, which is conservative
- The CLI builds an Operator from --actor, the same local trust model as approve
- The Horizon policy renderer and the sea-rs fixture are not tied by a test (follow-up)
- The server test covers a nonzero exit only; the timeout variant is covered at the CLI
- The world-load log is INFO level; SFWP has no way to list loaded worlds
- The liveness guard sees the recorded process group only: a process that left it (setsid/setpgid), a resolver in another pid namespace on the same boot, and the instant between fork and the ledger write are not seen
- An Unknown liveness result or a malformed launch record can be overridden only by the operator's audited --attest-command-gone; a group that is positively alive is always refused
- Exactly-once across an operator NotDeclared and a late declaration still relies on the external authority treating authorization_request_sha256 as an idempotency key (contract documented in sea-forge-settlement declaration.rs)
- Server-crash containment comes from the Horizon systemd unit (KillMode=control-group, Horizon#26); it takes effect once the owner reinstalls the unit
- An attempt with no launch record keeps the lease-only rule: NotDeclared is accepted after the lease lapses without a liveness check or --attest-command-gone. This covers claims made before launch recording existed, a transport without launch evidence, and a request with no authorization_ref. A failed launch-record write stops the command, so it does not produce such an attempt.
- Only sea-forge-capability was tested (the sole rusqlite user); full workspace tests, CI and MSRV 1.92 job not run locally; rusqlite 0.41.1 declares rust-version 1.89

**Next:** Merge with the other stack products pinned to 0.21.2; re-run the readiness gates C01, SW01 and SW02.
