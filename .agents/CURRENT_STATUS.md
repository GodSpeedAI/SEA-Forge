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
- Cognate/Horizon live suites not re-run
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
- The FDE planning baseline on main is documents only; S1 implementation is on the fde/integration branches, not on main
- Owner decisions OD-1 to OD-6 are open; see .agents/reports/fde-convergence-2026-11-12/H-egm-review-record.md
- The planning baseline is draft 3 and the SOP is a v0.4.1 draft; neither is approved

**Next:** Merge after CI; keep Cognate, Horizon, Workbench and RealityTrace on the same DomainForge version.
