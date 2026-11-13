# Current status

**Status revision:** 234

**Stage:** FDE integration branch: main plus Horizon desk support (PR #91)

**Summary:** fde/integration is origin/main (09b628ec) merged with feat/horizon-desk-support (6e8b6aff, PR #91: plan digest, case.add_item dry_run and preconditions, external_ref, digests on case reads). The other 33 unmerged local branches were each checked against main and are recorded in the branch ledger; none was ported. This branch is an integration check only and is not proposed for merge.

**Verified:**
- See the branch ledger for the gate commands and results run on this merge

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
- The case.add_item precondition is checked before the write, not under the same lock as the write
- contracts generated TypeScript under workbench/ is not regenerated here (needs bun)
- Integration branch only: not merged to main, no PR; PR #91 remains the route for the Horizon work
- Full workspace tests and just ci not run (host memory); see the branch ledger for what was run

**Next:** Review PR #91 on its own; decide the fate of feat/hardening-capability-harvest and feat/arm64-jetson-packaging per the branch ledger.
