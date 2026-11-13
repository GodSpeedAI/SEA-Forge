# Current status

**Status revision:** 233

**Stage:** e2e-world-loop-fix

**Summary:** scripts/e2e-world-loop.sh is fixed in three places: the Context Kernel hop builds package context-kernel-mcp (renamed upstream from ck-mcp, so hop 0 failed and every later hop had no input); quiet_cargo now returns cargo's real exit status (the old '|| true' reset PIPESTATUS, so a failed hop never stopped the script); and the GodSpeed-Agent hops use 'uv run --quiet --extra dev pytest' so pytest exists on a fresh checkout. Script logic is otherwise unchanged. E5B execution remains simulated in this loop.

**Verified:**
- bash -n scripts/e2e-world-loop.sh: ok
- scripts/check-agent-context.sh: ok
- 2026-11-13 patched script run against pinned origin heads (SEA-Forge main, Context Kernel 36bd8c8, SWE_SEED, RealityTrace main, cep 5111ec3, GodSpeed-Agent 4bfed12) with DomainForge 0.21.2: hops 1,0,2,3,4,5,6 passed, cep context_bundle valid, three refusals refused, final line 'e2e-world-loop: OK. One world (world:demo@sha256:6e00ae2d...) from E1 to the evidence the agent records.'
- before the patch the same run failed at hop 0 (package ID specification `ck-mcp` did not match any packages) yet continued through every later hop

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

**Next:** Review and merge the e2e-world-loop fix; then re-run just e2e-world-loop after any pin change.
