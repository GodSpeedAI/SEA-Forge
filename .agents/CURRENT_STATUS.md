# Current status

**Status revision:** 289

**Stage:** Chrono migration step 1: golden timestamp tests extended

**Summary:** Extend the timestamp golden tests to the PostgreSQL session expiry and the ACP approval stamps, pin stamp formats rather than comparison semantics, and record the leap-second rule the swap will implement.

**Verified:**

- PostgreSQL session expiry: a new golden test against a real pg0 cluster (SEA_FORGE_SECURITY_PG_INSTALL=~/.pg0/installation/18.1.0, SEA_FORGE_REQUIRE_POSTGRES=1) passes: the session expires_at is whole seconds with a literal Z, PostgreSQL's rolvaliduntil renders back to the identical text, and renewal moves it in the same shape; it fails rather than skips when SEA_FORGE_REQUIRE_POSTGRES is set.
- ACP permission approvals (delegation.rs): conformance_m16 now pins requested_at and expires_at as +00:00 automatic-fraction stamps whose difference is exactly the endpoint timeout (3 s), and the escalated-permission test passes.
- The approval inbox golden no longer pins the text comparison (a bug, fixed in a separate PR): it pins that stored requested_at and expires_at are returned verbatim.
- Leap-second rule for the swap: comparison-only sites accept :60 and fold it into the next second (pinned: the discrimination baseline completed_at); values that are stored or serialized reject :60 explicitly so their bytes never change silently (so the stored window_closes_at :60 rendering is deliberately no longer pinned).
- authority 8, server 4 plus the postgres test and conformance_m16 (18 passed; its t16_8 case needs a built sea-forge CLI binary in the target dir and was not run here), ledger 4, core 1, trace 1; clippy -D warnings and fmt clean.

**Limits:**

- Still not pinned through a public seam: case_dispatch.rs sandbox-episode approval requested_at/expires_at (no existing test drives the flow over the wire; the delegation.rs ACP path is pinned instead), the approval-window parse at postgres.rs:403 (covered only by the existing broker suite, which needs workload isolation that this host cannot provide), authority lib.rs approval-grant expiry parsing, and the case-runner, CLI, thoth, settlement, sandbox and cell stamps.
- Now-derived stamps can only be pinned by shape and arithmetic, not by fixed instants; fixed instants are pinned only where the instant is an input (obligation claims and runs).
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
- cargo test --workspace and just ci not run (host memory); Cognate/Horizon live suites not re-run
- FDE planning baseline is documents only (S1 not started); the hardening work changes product code
- Open: 24.04 fixture defect (non-gating); admission control; F17 allow-control (upstream); Milestone C on-device
- macOS ProcessGroup claim rests on a best-effort census; live egress allow-control policy shape: owner decisions pending on PR #71

**Next:** Merge the instant-comparison fix, then migrate to the time crate with these tests unedited.

**Evidence:**

- crates/sea-forge-server/tests/timestamp_golden_postgres.rs
- crates/sea-forge-server/tests/conformance_m16.rs
- crates/sea-forge-server/tests/timestamp_golden.rs
- crates/sea-forge-authority/tests/timestamp_golden.rs
