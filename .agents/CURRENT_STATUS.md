# Current status

**Status revision:** 235

**Stage:** FDE integration branch: planner test fix after the Horizon merge

**Summary:** fde/integration: sea-forge-planner/tests/conformance_m2.rs now sets the new ItemMarkers.review field in its two struct literals; the merge of feat/horizon-desk-support (PR #91) left the test target unable to compile. PR #91 itself has the same defect. Gate results are in the branch ledger.

**Verified:**
- cargo fmt --check, cargo check --workspace --locked: ok
- cargo test -p sea-forge-core, case-runner, cli, server --locked: 0 failures; sea-forge-planner 69 passed 0 failed after the fix

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

**Next:** Apply the same one-line test fix to PR #91 (feat/horizon-desk-support) before it merges.
